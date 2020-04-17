import sys
import logging
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


BOOLEAN = 0x1
INTEGER8 = 0x2
INTEGER16 = 0x3
INTEGER32 = 0x4
UNSIGNED8 = 0x5
UNSIGNED16 = 0x6
UNSIGNED32 = 0x7
REAL32 = 0x8
VISIBLE_STRING = 0x9
OCTET_STRING = 0xA
UNICODE_STRING = 0xB
DOMAIN = 0xF
REAL64 = 0x11
INTEGER64 = 0x15
UNSIGNED64 = 0x1B

SIGNED_TYPES = (INTEGER8, INTEGER16, INTEGER32, INTEGER64)
UNSIGNED_TYPES = (UNSIGNED8, UNSIGNED16, UNSIGNED32, UNSIGNED64)
INTEGER_TYPES = SIGNED_TYPES + UNSIGNED_TYPES
FLOAT_TYPES = (REAL32, REAL64)
NUMBER_TYPES = INTEGER_TYPES + FLOAT_TYPES
DATA_TYPES = (VISIBLE_STRING, OCTET_STRING, UNICODE_STRING, DOMAIN)

logger = logging.getLogger(__name__)


class ObjectDictionary:
    def __init__(self):
        self.indices = {}
        self.names = {}
        #: Default bitrate if specified by file
        self.bitrate = None
        #: Node ID if specified by file
        self.node_id = None

    def __getitem__(self, index):
        item = self.names.get(index) or self.indices.get(index)
        if item is None:
            name = "0x%X" % index if isinstance(index, int) else index
            raise KeyError("%s was not found in Object Dictionary" % name)
        return item

    def add_object(self, obj):
        obj.parent = self
        self.indices[obj.index] = obj
        self.names[obj.name] = obj


class Record:
    #: Description for the whole record
    description = ""

    def __init__(self, name, index):
        #: The :class:`~canopen_lib.ObjectDictionary` owning the record.
        self.parent = None
        #: 16-bit address of the record
        self.index = index
        #: Name of record
        self.name = name
        self.subindices = {}
        self.names = {}

    def add_member(self, variable):
        variable.parent = self
        self.subindices[variable.subindex] = variable
        self.names[variable.name] = variable


class Array:
    #: Description for the whole array
    description = ""

    def __init__(self, name, index):
        #: The :class:`~canopen_lib.ObjectDictionary` owning the record.
        self.parent = None
        #: 16-bit address of the array
        self.index = index
        #: Name of array
        self.name = name
        self.subindices = {}
        self.names = {}

    def __getitem__(self, subindex):
        var = self.names.get(subindex) or self.subindices.get(subindex)
        if var is not None:
            # This subindex is defined
            pass
        elif isinstance(subindex, int) and 0 < subindex < 256:
            # Create a new variable based on first array item
            template = self.subindices[1]
            name = "%s_%x" % (template.name, subindex)
            var = Variable(name, self.index, subindex)
            var.parent = self
            for attr in ("data_type", "unit", "factor", "min", "max", "default",
                         "access_type", "description", "value_descriptions",
                         "bit_definitions"):
                if attr in template.__dict__:
                    var.__dict__[attr] = template.__dict__[attr]
        else:
            raise KeyError("Could not find subindex %r" % subindex)
        return var

    def add_member(self, variable):
        variable.parent = self
        self.subindices[variable.subindex] = variable
        self.names[variable.name] = variable


class Variable(object):
    def __init__(self, name, index, subindex=0):
        #: The :class:`~canopen_lib.ObjectDictionary`,
        #: :class:`~canopen_lib.objectdictionary.Record` or
        #: :class:`~canopen_lib.objectdictionary.Array` owning the variable
        self.parent = None
        #: 16-bit address of the object in the dictionary
        self.index = index
        #: 8-bit sub-index of the object in the dictionary
        self.subindex = subindex
        #: String representation of the variable
        self.name = name
        #: Physical unit
        self.unit = ""
        #: Factor between physical unit and integer value
        self.factor = 1
        #: Minimum allowed value
        self.min = None
        #: Maximum allowed value
        self.max = None
        #: Default value at start-up
        self.default = None
        #: The value of this variable stored in the object dictionary
        self.value = None
        #: Data type according to the standard as an :class:`int`
        self.data_type = None
        #: Access type, should be "rw", "ro", "wo", or "const"
        self.access_type = "rw"
        #: Description of variable
        self.description = ""
        #: Dictionary of value descriptions
        self.value_descriptions = {}
        #: Dictionary of bitfield definitions
        self.bit_definitions = {}
