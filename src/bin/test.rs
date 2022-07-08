use project_kind_macros::component;

#[component]
struct MyComponent {
    age: u8,
}

fn main() {
    println!("Hello, world!");
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
