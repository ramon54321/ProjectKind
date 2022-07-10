use project_kind::{deserialize, serialize, HasLayout, Layout, LayoutKind};

#[repr(C)]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Person {
    age: u8,
    name: String,
    height: u16,
}
impl HasLayout for Person {
    fn get_layout() -> Layout {
        Layout {
            name: String::from("Person"),
            kind: LayoutKind::Struct,
            fields: Some(vec![
                Layout {
                    name: String::from("age"),
                    kind: LayoutKind::U8,
                    fields: None,
                },
                Layout {
                    name: String::from("name"),
                    kind: LayoutKind::String,
                    fields: None,
                },
                Layout {
                    name: String::from("height"),
                    kind: LayoutKind::U16,
                    fields: None,
                },
            ]),
        }
    }
}

fn main() {
    //
    // Typed --Bincode--> Bytes --Custom--> String
    // String --Custom--> Bytes --Bincode--> Typed
    //

    println!();
    println!("--Single--");

    let person_layout = Layout {
        name: String::from("Person"),
        kind: LayoutKind::Struct,
        fields: Some(vec![
            Layout {
                name: String::from("age"),
                kind: LayoutKind::U8,
                fields: None,
            },
            Layout {
                name: String::from("name"),
                kind: LayoutKind::String,
                fields: None,
            },
            Layout {
                name: String::from("height"),
                kind: LayoutKind::U16,
                fields: None,
            },
        ]),
    };

    let person_typed = Person {
        age: 27,
        height: 180,
        name: String::from("Bob is a cool kid"),
    };
    let person_two_typed = Person {
        age: 50,
        height: 150,
        name: String::from("James D"),
    };
    println!("Person Typed: {:?}", person_typed);
    println!("Person Two Typed: {:?}", person_two_typed);

    let person_bytes = bincode::serialize(&person_typed).unwrap();
    let person_two_bytes = bincode::serialize(&person_two_typed).unwrap();
    println!("Person Bytes: {:?}", person_bytes);
    println!("Person Two Bytes: {:?}", person_two_bytes);

    let person_string = serialize(&person_layout, &person_bytes);
    let person_two_string = serialize(&person_layout, &person_two_bytes);
    println!("Person String: {}", person_string);
    println!("Person Two String: {}", person_two_string);

    let person_bytes = deserialize(&person_layout, &person_string);
    let person_two_bytes = deserialize(&person_layout, &person_two_string);
    println!("Person Bytes: {:?}", person_bytes);
    println!("Person Two Bytes: {:?}", person_two_bytes);

    let person_typed = bincode::deserialize::<Person>(&person_bytes).unwrap();
    let person_two_typed = bincode::deserialize::<Person>(&person_two_bytes).unwrap();
    println!("Person Typed: {:?}", person_typed);
    println!("Person Two Typed: {:?}", person_two_typed);

    println!();
    println!("--Array--");

    let person_array_layout = Layout {
        name: String::from("None"),
        kind: LayoutKind::Array,
        fields: Some(vec![person_layout.clone()]),
    };

    let person_array_typed = vec![person_typed, person_two_typed];
    println!("Person Array Typed: {:?}", person_array_typed);

    let person_array_bytes = bincode::serialize(&person_array_typed).unwrap();
    println!("Person Array Bytes: {:?}", person_array_bytes);

    let person_array_string = serialize(&person_array_layout, &person_array_bytes);
    println!("Person Array String: {}", person_array_string);

    let person_array_bytes = deserialize(&person_array_layout, &person_array_string);
    println!("Person Array Bytes: {:?}", person_array_bytes);

    let person_array_typed = bincode::deserialize::<Vec<Person>>(&person_array_bytes).unwrap();
    println!("Person Array Typed: {:?}", person_array_typed);
}
