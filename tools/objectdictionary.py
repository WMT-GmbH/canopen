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

DATA_TYPE_NAMES = {0x1: 'BOOLEAN', 0x2: 'INTEGER8', 0x3: 'INTEGER16', 0x4: 'INTEGER32', 0x5: 'UNSIGNED8',
                   0x6: 'UNSIGNED16', 0x7: 'UNSIGNED32', 0x8: 'REAL32', 0x9: 'VISIBLE_STRING',
                   0xA: 'OCTET_STRING', 0xB: 'UNICODE_STRING', 0xF: 'DOMAIN', 0x11: 'REAL64',
                   0x15: 'INTEGER64', 0x1B: 'UNSIGNED64'}


class ObjectDictionary:
    def __init__(self):
        self.indices = {}
        #: Default bitrate if specified by file
        self.bitrate = None
        #: Node ID if specified by file
        self.node_id = None

    def __getitem__(self, index):
        return self.indices[index]

    @property
    def variables(self):
        return [var for var in [self.indices[index] for index in sorted(self.indices)] if isinstance(var, Variable)]

    @property
    def arrays(self):
        return [var for var in [self.indices[index] for index in sorted(self.indices)] if isinstance(var, Array)]

    @property
    def records(self):
        return [var for var in [self.indices[index] for index in sorted(self.indices)] if isinstance(var, Record)]

    def add_object(self, obj):
        self.indices[obj.index] = obj


class Record:
    #: Description for the whole record
    description = ""

    def __init__(self, name, index):
        #: 16-bit address of the record
        self.index = index
        #: Name of record
        self.name = name
        self.subindices = {}

    @property
    def members(self):
        return [self.subindices[index] for index in sorted(self.subindices)]

    def add_member(self, variable):
        variable.parent = self
        self.subindices[variable.subindex] = variable


class Array:
    #: Description for the whole array
    description = ""

    def __init__(self, name, index):
        #: 16-bit address of the array
        self.index = index
        #: Name of array
        self.name = name
        self.subindices = {}

    def __getitem__(self, subindex):
        return self.subindices[subindex]

    @property
    def members(self):
        return [self.subindices[index] for index in sorted(self.subindices)]

    def add_member(self, variable):
        self.subindices[variable.subindex] = variable


class Variable(object):
    def __init__(self, name, index, subindex=0):
        #: 16-bit address of the object in the dictionary
        self.index = index
        #: 8-bit sub-index of the object in the dictionary
        self.subindex = subindex
        #: String representation of the variable
        self.name: str = name
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
