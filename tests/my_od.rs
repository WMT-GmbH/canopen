use canopen::objectdictionary::{Array, ObjectDictionary, Variable};

pub const OD_DUMMY0001: Variable =
    Variable::new(0x0001, 0x00, None);
pub const OD_DUMMY0002: Variable =
    Variable::new(0x0002, 0x00, None);
pub const OD_DEVICE_TYPE: Variable =
    Variable::new(0x1000, 0x00, None);
pub const OD_ERROR_REGISTER: Variable =
    Variable::new(0x1001, 0x00, None);
pub const OD_MANUFACTURER_DEVICE_NAME: Variable =
    Variable::new(0x1008, 0x00, None);
pub const OD_PRODUCER_HEARTBEAT_TIME: Variable =
    Variable::new(0x1017, 0x00, None);
pub const OD_WRITABLE_STRING: Variable =
    Variable::new(0x2000, 0x00, None);
pub const OD_INTEGER16_VALUE: Variable =
    Variable::new(0x2001, 0x00, None);
pub const OD_UNSIGNED8_VALUE: Variable =
    Variable::new(0x2002, 0x00, None);
pub const OD_INTEGER8_VALUE: Variable =
    Variable::new(0x2003, 0x00, None);
pub const OD_INTEGER32_VALUE: Variable =
    Variable::new(0x2004, 0x00, None);
pub const OD_BOOLEAN_VALUE: Variable =
    Variable::new(0x2005, 0x00, None);
pub const OD_BOOLEAN_VALUE_2: Variable =
    Variable::new(0x2006, 0x00, None);
pub const OD_COMPLEX_DATA_TYPE: Variable =
    Variable::new(0x2020, 0x00, None);
pub const OD_SENSOR_SAMPLING_RATE_HZ: Variable =
    Variable::new(0x3002, 0x00, None);
pub const OD_PRE_DEFINED_ERROR_FIELD_NUMBER_OF_ENTRIES: Variable =
    Variable::new(0x1003, 0x00, None);
pub const OD_PRE_DEFINED_ERROR_FIELD_PRE_DEFINED_ERROR_FIELD: Variable =
    Variable::new(0x1003, 0x01, None);
pub const OD_SENSOR_STATUS_NUMBER_OF_ENTRIES: Variable =
    Variable::new(0x3004, 0x00, None);
pub const OD_SENSOR_STATUS_SENSOR_STATUS_1: Variable =
    Variable::new(0x3004, 0x01, None);
pub const OD_SENSOR_STATUS_SENSOR_STATUS_2: Variable =
    Variable::new(0x3004, 0x02, None);
pub const OD_SENSOR_STATUS_SENSOR_STATUS_3: Variable =
    Variable::new(0x3004, 0x03, None);
pub const OD_VALVE_1__OPEN_NUMBER_OF_ENTRIES: Variable =
    Variable::new(0x3006, 0x00, None);
pub const OD_VALVE_1__OPEN_VALVE_1__OPEN: Variable =
    Variable::new(0x3006, 0x01, None);
pub fn get_od() -> ObjectDictionary {
    let mut od = ObjectDictionary::default();
    od.add_variable(OD_DUMMY0001);
    od.add_variable(OD_DUMMY0002);
    od.add_variable(OD_DEVICE_TYPE);
    od.add_variable(OD_ERROR_REGISTER);
    od.add_variable(OD_MANUFACTURER_DEVICE_NAME);
    od.add_variable(OD_PRODUCER_HEARTBEAT_TIME);
    od.add_variable(OD_WRITABLE_STRING);
    od.add_variable(OD_INTEGER16_VALUE);
    od.add_variable(OD_UNSIGNED8_VALUE);
    od.add_variable(OD_INTEGER8_VALUE);
    od.add_variable(OD_INTEGER32_VALUE);
    od.add_variable(OD_BOOLEAN_VALUE);
    od.add_variable(OD_BOOLEAN_VALUE_2);
    od.add_variable(OD_COMPLEX_DATA_TYPE);
    od.add_variable(OD_SENSOR_SAMPLING_RATE_HZ);
    od.add_array(Array {
        index: 0x1003,
        members: vec![
            OD_PRE_DEFINED_ERROR_FIELD_NUMBER_OF_ENTRIES,
            OD_PRE_DEFINED_ERROR_FIELD_PRE_DEFINED_ERROR_FIELD,
        ]
    });
    od.add_array(Array {
        index: 0x3003,
        members: vec![]
    });
    od.add_array(Array {
        index: 0x3004,
        members: vec![
            OD_SENSOR_STATUS_NUMBER_OF_ENTRIES,
            OD_SENSOR_STATUS_SENSOR_STATUS_1,
            OD_SENSOR_STATUS_SENSOR_STATUS_2,
            OD_SENSOR_STATUS_SENSOR_STATUS_3,
        ]
    });
    od.add_array(Array {
        index: 0x3006,
        members: vec![
            OD_VALVE_1__OPEN_NUMBER_OF_ENTRIES,
            OD_VALVE_1__OPEN_VALVE_1__OPEN,
        ]
    });
    od
}
