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
    fn size_in_bytes(&self, bytes: &[u8]) -> usize {
        match self.kind {
            LayoutKind::Struct => {
                let mut total_size = 0;
                let mut offset = 0;
                for field in self.fields.iter() {
                    total_size = total_size + field.size_in_bytes(bytes);
                    offset = offset + field.size_in_bytes(&bytes[offset..]);
                }
                total_size
            }
            LayoutKind::String => {
                let length_bytes: [u8; 8] = bytes[..8].try_into().unwrap();
                let length = usize::from_le_bytes(length_bytes);
                length
            }
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
            let mut offset = 0;
            for field in layout.fields.iter() {
                value.insert(
                    field.name.clone(),
                    build_value_from_layout(&field, &bytes[offset..]),
                );
                offset = offset + field.size_in_bytes(&bytes[offset..]);
            }
            Value::from(value)
        }
        LayoutKind::String => {
            let length_bytes: [u8; 8] = bytes[..8].try_into().unwrap();
            let length = usize::from_le_bytes(length_bytes);
            let start = 8;
            let end = start + length;
            Value::from(String::from_utf8(bytes[start..end].to_vec()).unwrap())
        }
        LayoutKind::Bool => Value::from(*get_in_slice::<u8>(&bytes, 0).unwrap()),
        LayoutKind::U8 => Value::from(*get_in_slice::<u8>(&bytes, 0).unwrap()),
        LayoutKind::U16 => Value::from(*get_in_slice::<u16>(&bytes, 0).unwrap()),
        LayoutKind::U32 => Value::from(*get_in_slice::<u32>(&bytes, 0).unwrap()),
        LayoutKind::U64 => Value::from(*get_in_slice::<u64>(&bytes, 0).unwrap()),
        LayoutKind::I8 => Value::from(*get_in_slice::<i8>(&bytes, 0).unwrap()),
        LayoutKind::I16 => Value::from(*get_in_slice::<i16>(&bytes, 0).unwrap()),
        LayoutKind::I32 => Value::from(*get_in_slice::<i32>(&bytes, 0).unwrap()),
        LayoutKind::I64 => Value::from(*get_in_slice::<i64>(&bytes, 0).unwrap()),
        LayoutKind::F32 => Value::from(*get_in_slice::<f32>(&bytes, 0).unwrap()),
        LayoutKind::F64 => Value::from(*get_in_slice::<f64>(&bytes, 0).unwrap()),
    }
}

fn build_bytes_from_layout(layout: &Layout, value: &Value) -> Vec<u8> {
    match layout.kind {
        LayoutKind::Struct => {
            let mut bytes = Vec::new();
            let value_object = value.as_object().unwrap();
            for field in layout.fields.iter() {
                let field_value = value_object.get(&field.name).unwrap();
                let field_bytes = build_bytes_from_layout(&field, field_value);
                for byte in field_bytes.iter() {
                    bytes.push(*byte);
                }
            }
            bytes
        }
        LayoutKind::String => {
            let value_string = value.as_str().unwrap();
            let mut bytes = Vec::new();
            let length_bytes = value_string.len().to_le_bytes();
            for byte in length_bytes.iter() {
                bytes.push(*byte);
            }
            let string_bytes = value_string.as_bytes();
            for byte in string_bytes.iter() {
                bytes.push(*byte);
            }
            bytes
        }
        LayoutKind::Bool => {
            if value.as_bool().unwrap() {
                vec![1u8]
            } else {
                vec![0u8]
            }
        }
        LayoutKind::U8 => (value.as_u64().unwrap() as u8).to_be_bytes().to_vec(),
        LayoutKind::U16 => (value.as_u64().unwrap() as u16).to_le_bytes().to_vec(),
        LayoutKind::U32 => (value.as_u64().unwrap() as u32).to_le_bytes().to_vec(),
        LayoutKind::U64 => (value.as_u64().unwrap() as u64).to_le_bytes().to_vec(),
        LayoutKind::I8 => (value.as_i64().unwrap() as i8).to_be_bytes().to_vec(),
        LayoutKind::I16 => (value.as_i64().unwrap() as i16).to_le_bytes().to_vec(),
        LayoutKind::I32 => (value.as_i64().unwrap() as i32).to_le_bytes().to_vec(),
        LayoutKind::I64 => (value.as_i64().unwrap() as i64).to_le_bytes().to_vec(),
        LayoutKind::F32 => (value.as_f64().unwrap() as f32).to_le_bytes().to_vec(),
        LayoutKind::F64 => (value.as_f64().unwrap() as f64).to_le_bytes().to_vec(),
    }
}

fn serialize(layout: &Layout, bytes: &Vec<u8>) -> String {
    let value = build_value_from_layout(layout, &bytes);
    serde_json::to_string(&value).unwrap()
}

fn deserialize(layout: &Layout, serial: &String) -> Vec<u8> {
    let value = serde_json::from_str::<Value>(serial).unwrap();
    build_bytes_from_layout(layout, &value)
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
        fields: vec![
            Layout {
                name: String::from("age"),
                kind: LayoutKind::U8,
                fields: Vec::new(),
            },
            Layout {
                name: String::from("height"),
                kind: LayoutKind::U16,
                fields: Vec::new(),
            },
            Layout {
                name: String::from("name"),
                kind: LayoutKind::String,
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

    println!();
    println!("--Array--");

    let person_array_typed = vec![person_typed, person_two_typed];
    println!("Person Array Typed: {:?}", person_array_typed);

    let person_array_bytes = bincode::serialize(&person_array_typed).unwrap();
    println!("Person Array Bytes: {:?}", person_array_bytes);

    let person_array_string = serialize(&person_layout, &person_array_bytes);
    println!("Person Array String: {:?}", person_array_string);
}
