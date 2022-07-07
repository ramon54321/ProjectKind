use std::mem::size_of;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
struct Person {
    age: u8,
    height: u16,
    name: String,
}

fn get_in_slice<'a, T>(slice: &'a [u8], offset: usize) -> Option<&'a T> {
    let size = size_of::<T>();
    if offset + size > slice.len() {
        return None;
    }
    let subslice = &slice[offset..(offset + size)];
    let value = unsafe { &*(subslice as *const _ as *const T) };
    Some(value)
}

struct Layout {
    name: String,
    kind: LayoutKind,
    offset: u32,
    fields: Vec<Layout>,
}

enum LayoutKind {
    Struct,
    String,
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

impl Layout {
    fn size_in_bytes(&self) -> u32 {
        match self.kind {
            LayoutKind::Struct => {
                let mut total_size = 0;
                for field in self.fields.iter() {
                    total_size = total_size + field.size_in_bytes();
                }
                total_size
            }
            LayoutKind::String => 0,
            LayoutKind::Bool => 1,
            LayoutKind::U8 => 1,
            LayoutKind::U16 => 2,
            LayoutKind::U32 => 4,
            LayoutKind::U64 => 8,
            LayoutKind::I8 => 1,
            LayoutKind::I16 => 2,
            LayoutKind::I32 => 4,
            LayoutKind::I64 => 8,
            LayoutKind::F32 => 4,
            LayoutKind::F64 => 8,
        }
    }
}

fn build_value_from_layout(layout: &Layout, bytes: &[u8]) -> Value {
    match layout.kind {
        LayoutKind::Struct => {
            let mut value = Map::new();
            for field in layout.fields.iter() {
                value.insert(field.name.clone(), build_value_from_layout(&field, &bytes));
            }
            Value::from(value)
        }
        LayoutKind::String => {
            let start = layout.offset as usize;
            let length_bytes: [u8; 8] = bytes[start..(start + 8)].try_into().unwrap();
            let length = usize::from_le_bytes(length_bytes);
            let start = start + 8;
            let end = start + length;
            Value::from(String::from_utf8(bytes[start..end].to_vec()).unwrap())
        }
        LayoutKind::Bool => {
            Value::from(*get_in_slice::<u8>(&bytes[(layout.offset as usize)..], 0).unwrap())
        }
        LayoutKind::U8 => {
            Value::from(*get_in_slice::<u8>(&bytes[(layout.offset as usize)..], 0).unwrap())
        }
        LayoutKind::U16 => {
            Value::from(*get_in_slice::<u16>(&bytes[(layout.offset as usize)..], 0).unwrap())
        }
        LayoutKind::U32 => {
            Value::from(*get_in_slice::<u32>(&bytes[(layout.offset as usize)..], 0).unwrap())
        }
        LayoutKind::U64 => {
            Value::from(*get_in_slice::<u64>(&bytes[(layout.offset as usize)..], 0).unwrap())
        }
        LayoutKind::I8 => {
            Value::from(*get_in_slice::<i8>(&bytes[(layout.offset as usize)..], 0).unwrap())
        }
        LayoutKind::I16 => {
            Value::from(*get_in_slice::<i16>(&bytes[(layout.offset as usize)..], 0).unwrap())
        }
        LayoutKind::I32 => {
            Value::from(*get_in_slice::<i32>(&bytes[(layout.offset as usize)..], 0).unwrap())
        }
        LayoutKind::I64 => {
            Value::from(*get_in_slice::<i64>(&bytes[(layout.offset as usize)..], 0).unwrap())
        }
        LayoutKind::F32 => {
            Value::from(*get_in_slice::<f32>(&bytes[(layout.offset as usize)..], 0).unwrap())
        }
        LayoutKind::F64 => {
            Value::from(*get_in_slice::<f64>(&bytes[(layout.offset as usize)..], 0).unwrap())
        }
    }
}

fn build_bytes_from_layout(layout: &Layout, value: &Value) -> Vec<u8> {
    let mut bytes = Vec::new();
    for field in layout.fields.iter() {
        let field_value = value.as_object().unwrap().get(&field.name).unwrap();
        let field_bytes = match field.kind {
            LayoutKind::Struct => build_bytes_from_layout(&field, field_value),
            LayoutKind::String => {
                let field_value_string = field_value.as_str().unwrap();
                let mut bytes = Vec::new();
                let length_bytes = field_value_string.len().to_le_bytes();
                for byte in length_bytes.iter() {
                    bytes.push(*byte);
                }
                let string_bytes = field_value_string.as_bytes();
                for byte in string_bytes.iter() {
                    bytes.push(*byte);
                }
                bytes
            }
            LayoutKind::Bool => {
                if field_value.as_bool().unwrap() {
                    vec![1u8]
                } else {
                    vec![0u8]
                }
            }
            LayoutKind::U8 => (field_value.as_u64().unwrap() as u8).to_be_bytes().to_vec(),
            LayoutKind::U16 => (field_value.as_u64().unwrap() as u16)
                .to_le_bytes()
                .to_vec(),
            LayoutKind::U32 => (field_value.as_u64().unwrap() as u32)
                .to_le_bytes()
                .to_vec(),
            LayoutKind::U64 => (field_value.as_u64().unwrap() as u64)
                .to_le_bytes()
                .to_vec(),
            LayoutKind::I8 => (field_value.as_i64().unwrap() as i8).to_be_bytes().to_vec(),
            LayoutKind::I16 => (field_value.as_i64().unwrap() as i16)
                .to_le_bytes()
                .to_vec(),
            LayoutKind::I32 => (field_value.as_i64().unwrap() as i32)
                .to_le_bytes()
                .to_vec(),
            LayoutKind::I64 => (field_value.as_i64().unwrap() as i64)
                .to_le_bytes()
                .to_vec(),
            LayoutKind::F32 => (field_value.as_f64().unwrap() as f32)
                .to_le_bytes()
                .to_vec(),
            LayoutKind::F64 => (field_value.as_f64().unwrap() as f64)
                .to_le_bytes()
                .to_vec(),
        };
        for byte in field_bytes.iter() {
            bytes.push(*byte);
        }
    }
    bytes
}

fn serialize(layout: &Layout, bytes: &Vec<u8>) -> String {
    //let object_size = layout.size_in_bytes() as usize;
    //let value = if bytes.len() > object_size {
    //let object_count = bytes.len() / object_size;
    //let mut value_array = Vec::new();
    //for i in 0..object_count {
    //let value =
    //build_value_from_layout(layout, &bytes[(i * layout.size_in_bytes() as usize)..]);
    //value_array.push(value);
    //}
    //Value::from(value_array)
    //} else {
    //build_value_from_layout(layout, &bytes)
    //};
    let value = build_value_from_layout(layout, &bytes);
    serde_json::to_string(&value).unwrap()
}

fn deserialize(layout: &Layout, serial: &String) -> Vec<u8> {
    let value = serde_json::from_str::<Value>(serial).unwrap();
    if value.is_array()
    /* && layout.kind != LayoutKind::Array */
    {
        let value_array = value.as_array().unwrap();
        let mut bytes = Vec::new();
        for value in value_array.iter() {
            let value_bytes = build_bytes_from_layout(&layout, &value);
            for byte in value_bytes.iter() {
                bytes.push(*byte);
            }
        }
        return bytes;
    }
    build_bytes_from_layout(layout, &value)
}

fn main() {
    //
    // Typed --Bincode--> Bytes --Custom--> String
    // String --Custom--> Bytes --Bincode--> Typed
    //

    let person_layout = Layout {
        name: String::from("Person"),
        kind: LayoutKind::Struct,
        offset: 0,
        fields: vec![
            Layout {
                name: String::from("age"),
                kind: LayoutKind::U8,
                offset: 0,
                fields: Vec::new(),
            },
            Layout {
                name: String::from("height"),
                kind: LayoutKind::U16,
                offset: 1,
                fields: Vec::new(),
            },
            Layout {
                name: String::from("name"),
                kind: LayoutKind::String,
                offset: 3,
                fields: Vec::new(),
            },
        ],
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
    println!("Person String: {:?}", person_string);
    println!("Person Two String: {:?}", person_two_string);

    let person_bytes = deserialize(&person_layout, &person_string);
    let person_two_bytes = deserialize(&person_layout, &person_two_string);
    println!("Person Bytes: {:?}", person_bytes);
    println!("Person Two Bytes: {:?}", person_two_bytes);

    let person_typed = bincode::deserialize::<Person>(&person_bytes).unwrap();
    let person_two_typed = bincode::deserialize::<Person>(&person_two_bytes).unwrap();
    println!("Person Typed: {:?}", person_typed);
    println!("Person Two Typed: {:?}", person_two_typed);

    //let mut multiple_person_bytes = person_bytes_after
    //.iter()
    //.cloned()
    //.chain(person_bytes_after.iter().cloned())
    //.chain(person_two_bytes.iter().cloned())
    //.chain(person_bytes_after.iter().cloned())
    //.chain(person_bytes_after.iter().cloned())
    //.collect::<Vec<u8>>();

    //println!("Persons is {} bytes.", multiple_person_bytes.len());

    //let mut multiple_person_typed =
    //bincode::deserialize::<[Person; 5]>(&multiple_person_bytes).unwrap();

    //multiple_person_typed[0].age = 10;
    //multiple_person_typed[1].age = 30;
    //multiple_person_typed[3].height = 120;

    //println!("{:?}", multiple_person_typed[1]);
    //println!("{:?}", multiple_person_typed[2]);

    //// Serialize and Deserialize array

    //let multiple_person_serialized = serialize(&person_layout, &multiple_person_bytes);

    //println!("Multiple Serialized: {}", multiple_person_serialized);

    //let mut multiple_person_deserialized = deserialize(&person_layout, multiple_person_serialized);

    //let multiple_person_rebuilt =
    //unsafe { from_bytes::<Person>(&mut multiple_person_deserialized) };

    ////println!("{:?}", multiple_person_rebuilt);
}
