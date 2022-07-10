use project_kind::{HasLayout, Layout, LayoutKind};
use project_kind_macros::component;

#[component]
struct Person {
    age: u8,
}

#[test]
fn simple_u8() {
    let actual_layout = Person::get_layout();
    let expected_layout = Layout {
        name: String::from("Person"),
        kind: LayoutKind::Struct,
        fields: Some(vec![Layout {
            name: String::from("age"),
            kind: LayoutKind::U8,
            fields: None,
        }]),
    };
    assert_eq!(actual_layout, expected_layout);
}
