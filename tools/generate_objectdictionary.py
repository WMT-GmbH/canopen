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
            print('')
            print('pub fn get_od() -> ObjectDictionary {')
            print('    let mut od = ObjectDictionary::default();')
            for variable in od.variables:
                generate_variable(variable)
            for array in od.arrays:
                generate_array(array)
            print('    od')
            print('}')
    finally:
        sys.stdout = sys.__stdout__


def generate_variable(variable: Variable, indent=4):
    indent = ' ' * indent

    string = (
        f'od.add_variable(Variable {{\n'
        f'    index: 0x{variable.index:04x},\n'
        f'    subindex: 0x{variable.subindex:02x},\n'
        f'    name: String::from("{variable.name}"),\n'
        f'}});\n'
    )
    print(textwrap.indent(string, indent))


def generate_array(array: Array, indent=4):
    indent = ' ' * indent
    inner_indent = indent + '    '

    if len(array.members):
        no_members_string = ''
        print(indent + 'let members = vec![')
        for member in array.members:
            string = (
                f'Variable {{\n'
                f'    index: 0x{member.index:04x},\n'
                f'    subindex: 0x{member.subindex:02x},\n'
                f'    name: String::from("{member.name}"),\n'
                f'}},'
            )
            print(textwrap.indent(string, inner_indent))
        print(indent + '];\n')
    else:
        no_members_string = ': vec![]'

    string = (
        f'od.add_array(Array {{\n'
        f'    index: 0x{array.index:04x},\n'
        f'    name: String::from("{array.name}"),\n'
        f'    members{no_members_string},\n'
        f'}});'
    )
    print(textwrap.indent(string, indent))


if __name__ == '__main__':
    EDS_PATH = Path(__file__).parent / 'sample.eds'
    OUT_PATH = Path(__file__).parent.parent / 'tests' / 'od.rs'
    generate(EDS_PATH, OUT_PATH, node_id=2)

