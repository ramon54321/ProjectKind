use std::{mem::size_of, slice::from_raw_parts_mut};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[repr(C, packed)]
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
struct Person {
    age: u8,
    height: u16,
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

unsafe fn from_bytes<T>(bytes: &mut Vec<u8>) -> &mut [T] {
    from_raw_parts_mut(bytes.as_mut_ptr() as *mut T, bytes.len() / size_of::<T>())
}

struct Layout {
    name: String,
    kind: LayoutKind,
    offset: u32,
    fields: Vec<Layout>,
}

enum LayoutKind {
    Struct,
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

fn build_bytes_from_layout(layout: &Layout, value: Value) -> Vec<u8> {
    let mut bytes = Vec::new();
    if let Some(array_value) = value.as_array() {
        for value in array_value.iter() {
            let field_value = value
                .as_object()
                .unwrap()
                .get("age")
                .unwrap()
                .as_u64()
                .unwrap()
                .to_be_bytes();
            for byte in field_value[7..8].iter().rev() {
                bytes.push(*byte);
            }
            let field_value = value
                .as_object()
                .unwrap()
                .get("height")
                .unwrap()
                .as_u64()
                .unwrap()
                .to_be_bytes();
            for byte in field_value[6..8].iter().rev() {
                bytes.push(*byte);
            }
        }
        return bytes;
    }
    let field_value = value
        .as_object()
        .unwrap()
        .get("age")
        .unwrap()
        .as_u64()
        .unwrap()
        .to_be_bytes();
    for byte in field_value[7..8].iter().rev() {
        bytes.push(*byte);
    }
    let field_value = value
        .as_object()
        .unwrap()
        .get("height")
        .unwrap()
        .as_u64()
        .unwrap()
        .to_be_bytes();
    for byte in field_value[6..8].iter().rev() {
        bytes.push(*byte);
    }
    bytes
}

fn serialize(layout: &Layout, bytes: &Vec<u8>) -> String {
    let object_size = layout.size_in_bytes() as usize;
    let value = if bytes.len() > object_size {
        let object_count = bytes.len() / object_size;
        let mut value_array = Vec::new();
        for i in 0..object_count {
            let value =
                build_value_from_layout(layout, &bytes[(i * layout.size_in_bytes() as usize)..]);
            value_array.push(value);
        }
        Value::from(value_array)
    } else {
        build_value_from_layout(layout, &bytes)
    };
    serde_json::to_string(&value).unwrap()
}

fn deserialize(layout: &Layout, serial: String) -> Vec<u8> {
    let value = serde_json::from_str::<Value>(&serial).unwrap();
    build_bytes_from_layout(layout, value)
}

fn main() {
    let person = Person {
        age: 27,
        height: 180,
    };
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
        ],
    };

    let person_bytes = bincode::serialize(&person).unwrap();
    let person_serialized = serialize(&person_layout, &person_bytes);

    println!("{}", person_serialized);

    let person_bytes_after = deserialize(&person_layout, person_serialized);

    println!("{:?}", get_in_slice::<u8>(&person_bytes, 0));
    println!("{:?}", get_in_slice::<u8>(&person_bytes_after, 0));

    let mut multiple_person_bytes = person_bytes_after
        .iter()
        .cloned()
        .chain(person_bytes_after.iter().cloned())
        .chain(person_bytes_after.iter().cloned())
        .chain(person_bytes_after.iter().cloned())
        .chain(person_bytes_after.iter().cloned())
        .collect::<Vec<u8>>();

    println!("Persons is {} bytes.", multiple_person_bytes.len());

    let mut multiple_person_typed = unsafe { from_bytes::<Person>(&mut multiple_person_bytes) };

    multiple_person_typed[0].age = 10;
    multiple_person_typed[1].age = 30;
    multiple_person_typed[3].height = 120;

    println!("{:?}", multiple_person_typed[1]);

    // Serialize and Deserialize array

    let multiple_person_serialized = serialize(&person_layout, &multiple_person_bytes);

    println!("Multiple Serialized: {}", multiple_person_serialized);

    let mut multiple_person_deserialized = deserialize(&person_layout, multiple_person_serialized);

    let multiple_person_rebuilt =
        unsafe { from_bytes::<Person>(&mut multiple_person_deserialized) };

    println!("{:?}", multiple_person_rebuilt);
}
