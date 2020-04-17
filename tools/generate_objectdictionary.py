import sys
from pathlib import Path

import eds


def generate(eds_file, destination_path: Path, node_id):
    assert destination_path.parent.exists(), 'Path does not exist'
    od = eds.import_eds(eds_file, node_id)
    try:
        with open(destination_path, 'w') as f:
            sys.stdout = f
            print('use canopen::objectdictionary::{Array, ObjectDictionary, Variable};')
            print('')
            print('fn get_od() -> ObjectDictionary {')
            print('    let mut od = ObjectDictionary::default();')
            print('    od')
            print('}')
    finally:
        sys.stdout = sys.__stdout__
