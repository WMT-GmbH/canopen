import re
import logging
import copy
from configparser import RawConfigParser, NoOptionError, NoSectionError
import objectdictionary

logger = logging.getLogger(__name__)

DOMAIN = 2
VAR = 7
ARR = 8
RECORD = 9


def import_eds(source, node_id):
    eds = RawConfigParser()
    if hasattr(source, "read"):
        fp = source
    else:
        fp = open(source)
    eds.read_file(fp)
    fp.close()
    od = objectdictionary.ObjectDictionary()
    if eds.has_section("DeviceComissioning"):
        od.bitrate = int(eds.get("DeviceComissioning", "Baudrate")) * 1000
        od.node_id = int(eds.get("DeviceComissioning", "NodeID"))

    for section in eds.sections():
        # Match dummy definitions
        match = re.match(r"^[Dd]ummy[Uu]sage$", section)
        if match is not None:
            for i in range(1, 8):
                key = "Dummy%04d" % i
                if eds.getint(section, key) == 1:
                    var = objectdictionary.Variable(key, i, 0)
                    var.data_type = i
                    var.access_type = "const"
                    od.add_object(var)

        # Match indexes
        match = re.match(r"^[0-9A-Fa-f]{4}$", section)
        if match is not None:
            index = int(section, 16)
            name = eds.get(section, "ParameterName")
            try:
                object_type = int(eds.get(section, "ObjectType"), 0)
            except NoOptionError:
                # DS306 4.6.3.2 object description
                # If the keyword ObjectType is missing, this is regarded as
                # "ObjectType=0x7" (=VAR).
                object_type = VAR

            if object_type in (VAR, DOMAIN):
                var = build_variable(eds, section, node_id, index)
                od.add_object(var)
            elif object_type == ARR and eds.has_option(section, "CompactSubObj"):
                arr = objectdictionary.Array(name, index)
                last_subindex = objectdictionary.Variable(
                    "Number of entries", index, 0)
                last_subindex.data_type = objectdictionary.UNSIGNED8
                arr.add_member(last_subindex)
                arr.add_member(build_variable(eds, section, node_id, index, 1))
                od.add_object(arr)
            elif object_type == ARR:
                arr = objectdictionary.Array(name, index)
                od.add_object(arr)
            elif object_type == RECORD:
                record = objectdictionary.Record(name, index)
                od.add_object(record)

            continue

        # Match subindexes
        match = re.match(r"^([0-9A-Fa-f]{4})[S|s]ub([0-9A-Fa-f]+)$", section)
        if match is not None:
            index = int(match.group(1), 16)
            subindex = int(match.group(2), 16)
            entry = od[index]
            if isinstance(entry, (objectdictionary.Record,
                                  objectdictionary.Array)):
                var = build_variable(eds, section, node_id, index, subindex)
                entry.add_member(var)

        # Match [index]Name
        match = re.match(r"^([0-9A-Fa-f]{4})Name", section)
        if match is not None:
            index = int(match.group(1), 16)
            num_of_entries = int(eds.get(section, "NrOfEntries"))
            entry = od[index]
            # For CompactSubObj index 1 is were we find the variable
            src_var = od[index][1]
            for subindex in range(1, num_of_entries + 1):
                var = copy_variable(eds, section, subindex, src_var)
                if var is not None:
                    entry.add_member(var)

    return od


def _convert_variable(node_id, var_type, value):
    if var_type in objectdictionary.DATA_TYPES:
        return value
    elif var_type in objectdictionary.FLOAT_TYPES:
        return float(value)
    else:
        # COB-ID can have a suffix of '$NODEID+' so replace this with node_id before converting
        value = value.replace(" ", "").upper()
        if '$NODEID+' in value and node_id is not None:
            return int(value.replace('$NODEID+', ''), 0) + node_id
        else:
            return int(value, 0)


def build_variable(eds, section, node_id, index, subindex=0):
    """Creates a object dictionary entry.
    :param eds: String stream of the eds file
    :param section:
    :param node_id: Node ID
    :param index: Index of the CANOpen object
    :param subindex: Subindex of the CANOpen object (if presente, else 0)
    """
    name = eds.get(section, "ParameterName")
    var = objectdictionary.Variable(name, index, subindex)
    var.data_type = int(eds.get(section, "DataType"), 0)
    var.access_type = eds.get(section, "AccessType").lower()
    if var.data_type > 0x1B:
        # The object dictionary editor from CANFestival creates an optional object if min max values are used
        # This optional object is then placed in the eds under the section [A0] (start point, iterates for more)
        # The eds.get function gives us 0x00A0 now convert to String without hex representation and upper case
        # The sub2 part is then the section where the type parameter stands
        try:
            var.data_type = int(eds.get("%Xsub1" % var.data_type, "DefaultValue"), 0)
        except NoSectionError:
            logger.warning("%s has an unknown or unsupported data type (%X)", name, var.data_type)
            # Assume DOMAIN to force application to interpret the byte data
            var.data_type = objectdictionary.DOMAIN

    if eds.has_option(section, "LowLimit"):
        try:
            var.min = int(eds.get(section, "LowLimit"), 0)
        except ValueError:
            pass
    if eds.has_option(section, "HighLimit"):
        try:
            var.max = int(eds.get(section, "HighLimit"), 0)
        except ValueError:
            pass
    if eds.has_option(section, "DefaultValue"):
        try:
            var.default = _convert_variable(node_id, var.data_type, eds.get(section, "DefaultValue"))
        except ValueError:
            pass
    if eds.has_option(section, "ParameterValue"):
        try:
            var.value = _convert_variable(node_id, var.data_type, eds.get(section, "ParameterValue"))
        except ValueError:
            pass
    return var


def copy_variable(eds, section, subindex, src_var):
    name = eds.get(section, str(subindex))
    var = copy.copy(src_var)
    # It is only the name and subindex that varies
    var.name = name
    var.subindex = subindex
    return var
