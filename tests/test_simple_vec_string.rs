use project_kind::{HasLayout, Layout, LayoutKind};
use project_kind_macros::component;

#[component]
#[derive(Debug, Clone, PartialEq, Eq)]
struct Person {
    favorite_names: Vec<String>,
}

#[test]
fn serialize_deserialize_single() {
    let person_layout = Layout {
        name: String::from("Person"),
        kind: LayoutKind::Struct,
        fields: Some(vec![Layout {
            name: String::from("favorite_names"),
            kind: LayoutKind::Array,
            fields: Some(vec![Layout {
                name: String::from("unnamed"),
                kind: LayoutKind::String,
                fields: None,
            }]),
        }]),
    };

    let person_typed = Person {
        favorite_names: vec![String::from("Andy"), String::from("Bob")],
    };
    let person_two_typed = Person {
        favorite_names: vec![
            String::from("Angela"),
            String::from("Barbara Smith"),
            String::from("Chloe"),
        ],
    };

    let person_bytes = bincode::serialize(&person_typed).unwrap();
    let person_two_bytes = bincode::serialize(&person_two_typed).unwrap();

    let person_string = project_kind::serialize(&person_layout, &person_bytes);
    let person_two_string = project_kind::serialize(&person_layout, &person_two_bytes);

    let person_bytes_after = project_kind::deserialize(&person_layout, &person_string);
    let person_two_bytes_after = project_kind::deserialize(&person_layout, &person_two_string);

    let person_typed_after = bincode::deserialize::<Person>(&person_bytes_after).unwrap();
    let person_two_typed_after = bincode::deserialize::<Person>(&person_two_bytes_after).unwrap();

    assert_eq!(person_typed, person_typed_after);
    assert_eq!(person_two_typed, person_two_typed_after);

    let person_string_expected = r#"{"favorite_names":["Andy","Bob"]}"#;
    let person_two_string_expected = r#"{"favorite_names":["Angela","Barbara Smith","Chloe"]}"#;
    assert_eq!(person_string, person_string_expected);
    assert_eq!(person_two_string, person_two_string_expected);

    assert_eq!(person_bytes, person_bytes_after);
    assert_eq!(person_two_bytes, person_two_bytes_after);
}

#[test]
fn serialize_deserialize_array() {
    let person_layout = Layout {
        name: String::from("Person"),
        kind: LayoutKind::Struct,
        fields: Some(vec![Layout {
            name: String::from("favorite_names"),
            kind: LayoutKind::Array,
            fields: Some(vec![Layout {
                name: String::from("unnamed"),
                kind: LayoutKind::String,
                fields: None,
            }]),
        }]),
    };
    let person_array_layout = Layout {
        name: String::from("Persons"),
        kind: LayoutKind::Array,
        fields: Some(vec![person_layout]),
    };

    let person_typed = Person {
        favorite_names: vec![String::from("Andrew the 2nd")],
    };
    let person_two_typed = Person {
        favorite_names: vec![String::from("Aurora"), String::from("Beth Wilson")],
    };

    let person_array_typed = vec![person_typed.clone(), person_two_typed.clone()];

    let person_array_bytes = bincode::serialize(&person_array_typed).unwrap();
    println!("{:?}", person_array_bytes);

    let person_array_string = project_kind::serialize(&person_array_layout, &person_array_bytes);
    println!("{}", person_array_string);

    let person_array_bytes_after =
        project_kind::deserialize(&person_array_layout, &person_array_string);

    let person_array_typed_after =
        bincode::deserialize::<Vec<Person>>(&person_array_bytes_after).unwrap();

    assert_eq!(person_array_typed, person_array_typed_after);

    let person_array_string_expected =
        r#"[{"favorite_names":["Andrew the 2nd"]},{"favorite_names":["Aurora","Beth Wilson"]}]"#;
    assert_eq!(person_array_string, person_array_string_expected);

    assert_eq!(person_array_bytes, person_array_bytes_after);

    assert_eq!(person_array_typed[0], person_typed);
    assert_eq!(person_array_typed[1], person_two_typed);
}

#[test]
fn auto_implement_layout() {
    let actual_layout = Person::get_layout();
    let expected_layout = Layout {
        name: String::from("Person"),
        kind: LayoutKind::Struct,
        fields: Some(vec![Layout {
            name: String::from("favorite_names"),
            kind: LayoutKind::Array,
            fields: Some(vec![Layout {
                name: String::from("unnamed"),
                kind: LayoutKind::String,
                fields: None,
            }]),
        }]),
    };
    assert_eq!(actual_layout, expected_layout);
}
