use canopen::objectdictionary::{Array, ObjectDictionary, Variable};

fn get_od() -> ObjectDictionary {
    let mut od = ObjectDictionary::default();
    od.add_variable(Variable {
        index: 0x0003,
        subindex: 0x00,
        name: String::from("Dummy0003"),
    });

    od.add_variable(Variable {
        index: 0x1000,
        subindex: 0x00,
        name: String::from("Device type"),
    });

    od.add_variable(Variable {
        index: 0x1001,
        subindex: 0x00,
        name: String::from("Error register"),
    });

    od.add_variable(Variable {
        index: 0x1008,
        subindex: 0x00,
        name: String::from("Manufacturer device name"),
    });

    od.add_variable(Variable {
        index: 0x1017,
        subindex: 0x00,
        name: String::from("Producer heartbeat time"),
    });

    od.add_variable(Variable {
        index: 0x2000,
        subindex: 0x00,
        name: String::from("Writable string"),
    });

    od.add_variable(Variable {
        index: 0x2001,
        subindex: 0x00,
        name: String::from("INTEGER16 value"),
    });

    od.add_variable(Variable {
        index: 0x2002,
        subindex: 0x00,
        name: String::from("UNSIGNED8 value"),
    });

    od.add_variable(Variable {
        index: 0x2003,
        subindex: 0x00,
        name: String::from("INTEGER8 value"),
    });

    od.add_variable(Variable {
        index: 0x2004,
        subindex: 0x00,
        name: String::from("INTEGER32 value"),
    });

    od.add_variable(Variable {
        index: 0x2005,
        subindex: 0x00,
        name: String::from("BOOLEAN value"),
    });

    od.add_variable(Variable {
        index: 0x2006,
        subindex: 0x00,
        name: String::from("BOOLEAN value 2"),
    });

    od.add_variable(Variable {
        index: 0x2020,
        subindex: 0x00,
        name: String::from("Complex data type"),
    });

    od.add_variable(Variable {
        index: 0x3002,
        subindex: 0x00,
        name: String::from("Sensor Sampling Rate (Hz)"),
    });

    let members = vec![
        Variable {
            index: 0x1003,
            subindex: 0x00,
            name: String::from("Number of entries"),
        },
        Variable {
            index: 0x1003,
            subindex: 0x01,
            name: String::from("Pre-defined error field"),
        },
    ];

    od.add_array(Array {
        index: 0x1003,
        name: String::from("Pre-defined error field"),
        members,
    });
    let members = vec![];

    od.add_array(Array {
        index: 0x3003,
        name: String::from("Valve % open"),
        members,
    });
    let members = vec![
        Variable {
            index: 0x3004,
            subindex: 0x00,
            name: String::from("Number of entries"),
        },
        Variable {
            index: 0x3004,
            subindex: 0x01,
            name: String::from("Sensor Status 1"),
        },
        Variable {
            index: 0x3004,
            subindex: 0x02,
            name: String::from("Sensor Status 2"),
        },
        Variable {
            index: 0x3004,
            subindex: 0x03,
            name: String::from("Sensor Status 3"),
        },
    ];

    od.add_array(Array {
        index: 0x3004,
        name: String::from("Sensor Status"),
        members,
    });
    let members = vec![
        Variable {
            index: 0x3006,
            subindex: 0x00,
            name: String::from("Number of entries"),
        },
        Variable {
            index: 0x3006,
            subindex: 0x01,
            name: String::from("Valve 1 % Open"),
        },
    ];

    od.add_array(Array {
        index: 0x3006,
        name: String::from("Valve 1 % Open"),
        members,
    });
    od
}
