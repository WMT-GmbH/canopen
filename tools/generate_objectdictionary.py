import sys
import textwrap
from pathlib import Path

import eds
from objectdictionary import Variable, Array


def generate(eds_file, destination_path: Path, node_id):
    assert destination_path.parent.exists(), 'Path does not exist'
    od = eds.import_eds(eds_file, node_id)
    try:
        with open(destination_path, 'w') as f:
            sys.stdout = f
            print('use canopen::objectdictionary::{Array, ObjectDictionary, Variable};')
            print()
            variable_names = [generate_variable(variable) for variable in od.variables]
            array_members = {array: generate_array_variables(array) for array in od.arrays}
            print('pub fn get_od() -> ObjectDictionary {')
            print('    let mut od = ObjectDictionary::default();')
            for variable in variable_names:
                add_variable(variable)
            for array, members in array_members.items():
                add_array(array, members)
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
    string = (
        f'pub const {name}: Variable = Variable {{\n'
        f'    index: 0x{variable.index:04x},\n'
        f'    subindex: 0x{variable.subindex:02x},\n'
        f'}};\n'
    )
    print(string)
    return name


def generate_array_variables(array: Array):
    members = []
    for variable in array.members:
        variable.name = array.name + '_' + variable.name
        members.append(generate_variable(variable))
    return members


def add_variable(variable_name: str, indent=4):
    indent = ' ' * indent
    print(indent + f'od.add_variable({variable_name});')


def add_array(array: Array, member_names: list, indent=4):
    indent = ' ' * indent

    if member_names:
        member_string = '\n'
        for member_name in member_names:
            member_string += indent + ' ' * 4 + member_name + ',\n'
        member_string += indent
    else:
        member_string = ''

    string = (
        f'od.add_array(Array {{\n'
        f'    index: 0x{array.index:04x},\n'
        f'    members: vec![{member_string}]\n'
        f'}});'
    )
    print(textwrap.indent(string, indent))


if __name__ == '__main__':
    EDS_PATH = Path(__file__).parent / 'sample.eds'
    OUT_PATH = Path(__file__).parent.parent / 'tests' / 'my_od.rs'
    generate(EDS_PATH, OUT_PATH, node_id=2)
