use canopen::objectdictionary::{Array, ObjectDictionary, Variable};

fn get_od() -> ObjectDictionary {
    let mut od = ObjectDictionary::default();
    od.add_variable(Variable {
        index: 1,
        subindex: 0,
    });
    od.add_variable(Variable {
        index: 2,
        subindex: 0,
    });

    let array_content = vec![
        Variable {
            index: 3,
            subindex: 0,
        },
        Variable {
            index: 3,
            subindex: 1,
        },
    ];
    let array = Array::new(3, array_content);
    od.add_array(array);
    od
}
