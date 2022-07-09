use project_kind::{HasLayout, Layout, LayoutKind};
use project_kind_macros::component;

#[component]
struct Blob {
    age: u8,
}

#[component]
struct MyComponent {
    age: u8,
    name: String,
    height: Vec<u16>,
    blob: Blob,
}

fn main() {
    println!("{:?}", MyComponent::get_layout());
}

/*

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

*/
