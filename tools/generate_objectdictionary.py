import sys
import textwrap
from pathlib import Path

import eds
from objectdictionary import Variable, Array, DATA_TYPE_NAMES, NUMBER_TYPES, DOMAIN


def generate(eds_file, destination_path: Path, node_id):
    assert destination_path.parent.exists(), 'Path does not exist'
    od = eds.import_eds(eds_file, node_id)
    try:
        with open(destination_path, 'w') as f:
            sys.stdout = f
            print('use canopen::objectdictionary::{ObjectDictionary, Variable, Array, Record};')
            print('use canopen::datatypes::*;')
            print()
            variable_names = [generate_variable(variable) for variable in od.variables]
            members = {complex_obj: generate_members(complex_obj) for complex_obj in od.arrays + od.records}
            print()
            print('pub fn get_od() -> ObjectDictionary {')
            print('    let mut od = ObjectDictionary::default();')
            for variable in variable_names:
                add_variable(variable)
            for complex_obj, members in members.items():
                add_complex_obj(complex_obj, members)
            print('    od')
            print('}')
    finally:
        sys.stdout = sys.__stdout__


def generate_variable(variable: Variable):
    name = 'OD_' + variable.name.upper()\
        .replace(' ', '_')\
        .replace('(', '')\
        .replace(')', '')\
        .replace('-', '_')\
        .replace('%', '')
    default_string = 'None'
    if variable.default is not None:
        if variable.data_type in NUMBER_TYPES:
            default_string = f'Some({DATA_TYPE_NAMES[variable.data_type]}({variable.default}))'
        elif variable.data_type != DOMAIN:
            default_string = f'Some({DATA_TYPE_NAMES[variable.data_type]}("{variable.default}"))'

    string = (
        f'pub const {name}: Variable =\n'
        f'    Variable::new(0x{variable.index:04x}, 0x{variable.subindex:02x}, {default_string});'
    )
    print(string)
    return name


def generate_members(array: Array):
    members = []
    for variable in array.members:
        variable.name = array.name + '_' + variable.name
        members.append(generate_variable(variable))
    return members


def add_variable(variable_name: str, indent=4):
    indent = ' ' * indent
    print(indent + f'od.add_variable({variable_name});')


def add_complex_obj(complex_obj, member_names: list, indent=4):
    indent = ' ' * indent
    obj_type, func = ('Array', 'add_array') if isinstance(complex_obj, Array) else ('Record', 'add_record')

    if member_names:
        member_string = '\n'
        for member_name in member_names:
            member_string += indent + ' ' * 4 + member_name + ',\n'
        member_string += indent
    else:
        member_string = ''

    string = (
        f'od.{func}({obj_type} {{\n'
        f'    index: 0x{complex_obj.index:04x},\n'
        f'    members: vec![{member_string}]\n'
        f'}});'
    )
    print(textwrap.indent(string, indent))


if __name__ == '__main__':
    EDS_PATH = Path(__file__).parent / 'sample.eds'
    OUT_PATH = Path(__file__).parent.parent / 'tests' / 'my_od.rs'
    generate(EDS_PATH, OUT_PATH, node_id=2)
