import sys
import textwrap
from pathlib import Path

import eds
from objectdictionary import *


def generate(eds_file, destination_path: Path, node_id):
    assert destination_path.parent.exists(), 'Path does not exist'
    od = eds.import_eds(eds_file, node_id)
    try:
        with open(destination_path, 'w') as f:
            sys.stdout = f
            print('use canopen::objectdictionary::{Object, ObjectDictionary, Variable, Array, Record};')
            print('use canopen::datatypes::*;')
            print()

            for idx in sorted(od.indices):
                obj = od[idx]
                if isinstance(obj, Variable):
                    generate_variable(obj)
                else:
                    for variable in obj.members:
                        variable.name = obj.name + '_' + variable.name
                        generate_variable(variable)

            print()
            print('pub fn get_od() -> ObjectDictionary {')
            print('    ObjectDictionary {')
            print('        objects: [')
            for idx in sorted(od.indices):
                obj = od[idx]
                if isinstance(obj, Variable):
                    variable_name = make_variable_name(obj)
                    add_variable(variable_name, indent=12)
                else:
                    member_names = [make_variable_name(member) for member in obj.members]
                    add_complex_obj(obj, member_names, indent=12)

            print('        ]')
            print('    }')
            print('}')
    finally:
        sys.stdout = sys.__stdout__


def make_variable_name(variable: Variable) -> str:
    return 'OD_' + variable.name.upper() \
        .replace(' ', '_') \
        .replace('(', '') \
        .replace(')', '') \
        .replace('-', '_') \
        .replace('%', '')


def generate_variable(variable: Variable):
    name = make_variable_name(variable)
    default_string = 'None'
    if variable.default is not None:
        if variable.data_type in INTEGER_TYPES:
            default_string = f'Some({DATA_TYPE_NAMES[variable.data_type]}(0x{variable.default:x}))'
        elif variable.data_type in FLOAT_TYPES:
            default_string = f'Some({DATA_TYPE_NAMES[variable.data_type]}({variable.default}))'
        elif variable.data_type != DOMAIN:
            default_string = f'Some({DATA_TYPE_NAMES[variable.data_type]}("{variable.default}"))'

    string = (
        f'pub const {name}: Variable =\n'
        f'    Variable::new(0x{variable.index:04x}, 0x{variable.subindex:02x}, {default_string});'
    )
    print(string)


def generate_members(array: Array):
    for variable in array.members:
        variable.name = array.name + '_' + variable.name
        generate_variable(variable)


def add_variable(variable_name: str, indent=4):
    indent = ' ' * indent
    print(indent + f'Object::Variable({variable_name}),')


def add_complex_obj(complex_obj, member_names: list, indent=4):
    indent = ' ' * indent
    obj_type = 'Array' if isinstance(complex_obj, Array) else 'Record'

    if member_names:
        member_string = '\n'
        for member_name in member_names:
            member_string += indent + member_name + ',\n'
        member_string += indent
    else:
        member_string = ''

    string = (
        f'Object::{obj_type}({obj_type} {{\n'
        f'    index: 0x{complex_obj.index:04x},\n'
        f'    members: vec![{member_string}]\n'
        f'}}),'
    )
    print(textwrap.indent(string, indent))


if __name__ == '__main__':
    EDS_PATH = Path(__file__).parent / 'sample.eds'
    OUT_PATH = Path(__file__).parent.parent / 'tests' / 'my_od.rs'
    generate(EDS_PATH, OUT_PATH, node_id=2)
