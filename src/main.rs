use std::{mem::size_of, slice::from_raw_parts_mut};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[repr(C, packed)]
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
struct Person {
    age: u8,
    height: u16,
}

fn get_in_slice<'a, T>(slice: &'a Vec<u8>, offset: usize) -> Option<&'a T> {
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
}

fn build_value_from_layout(layout: &Layout, bytes: &Vec<u8>) -> Value {
    let bytes_per_object = 3; // From layout
    if bytes_per_object == bytes.len() {
        let mut value = Map::new();
        value.insert(
            String::from("age"),
            Value::from(*get_in_slice::<u8>(&bytes, 0).unwrap()),
        );
        value.insert(
            String::from("height"),
            Value::from(*get_in_slice::<u16>(&bytes, 1).unwrap()),
        );
        return Value::from(value);
    }
    let mut value_array = Vec::new();
    for offset in (0..bytes.len()).step_by(bytes_per_object) {
        let mut value = Map::new();
        value.insert(
            String::from("age"),
            Value::from(*get_in_slice::<u8>(&bytes, offset + 0).unwrap()),
        );
        value.insert(
            String::from("height"),
            Value::from(*get_in_slice::<u16>(&bytes, offset + 1).unwrap()),
        );
        value_array.push(Value::from(value));
    }
    Value::from(value_array)
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
    let value = build_value_from_layout(layout, bytes);
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
