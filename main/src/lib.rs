use std::mem::size_of;

use serde_json::{Map, Value};

fn get_in_slice<'a, T>(slice: &'a [u8], offset: usize) -> Option<&'a T> {
    let size = size_of::<T>();
    if offset + size > slice.len() {
        return None;
    }
    let subslice = &slice[offset..(offset + size)];
    let value = unsafe { &*(subslice as *const _ as *const T) };
    Some(value)
}

pub trait HasLayout {
    fn get_layout() -> Layout;
    fn get_name(&self) -> String;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    pub name: String,
    pub kind: LayoutKind,
    pub fields: Option<Vec<Layout>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayoutKind {
    Array,
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
            LayoutKind::Array => {
                let element_layout = self.fields.as_ref().unwrap().first().unwrap();
                let element_count_bytes: [u8; 8] = bytes[..8].try_into().unwrap();
                let element_count = usize::from_le_bytes(element_count_bytes);
                let mut byte_count = 0;
                for _ in 0..element_count {
                    let size = element_layout.size_in_bytes(&bytes[(8 + byte_count)..]);
                    byte_count = byte_count + size;
                }
                byte_count + 8
            }
            LayoutKind::Struct => {
                let mut byte_count = 0;
                for field in self.fields.as_ref().unwrap().iter() {
                    byte_count = byte_count + field.size_in_bytes(&bytes[byte_count..]);
                }
                byte_count
            }
            LayoutKind::String => {
                let length_bytes: [u8; 8] = bytes[..8].try_into().unwrap();
                let length = usize::from_le_bytes(length_bytes);
                8 + length
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
        LayoutKind::Array => {
            let element_count_bytes: [u8; 8] = bytes[..8].try_into().unwrap();
            let element_count = usize::from_le_bytes(element_count_bytes);
            let start = 8;
            let element_layout = layout.fields.as_ref().unwrap().first().unwrap();
            let mut offset = start;
            let mut value_array = Vec::new();
            for _ in 0..element_count {
                let element_value = build_value_from_layout(&element_layout, &bytes[offset..]);
                value_array.push(element_value);
                let element_size = element_layout.size_in_bytes(&bytes[offset..]);
                offset = offset + element_size;
            }
            Value::from(value_array)
        }
        LayoutKind::Struct => {
            let mut value = Map::new();
            let mut offset = 0;
            for field in layout.fields.as_ref().unwrap().iter() {
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
        LayoutKind::Bool => Value::from(*get_in_slice::<bool>(&bytes, 0).unwrap()),
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
        LayoutKind::Array => {
            let value_array = value.as_array().unwrap();
            let mut bytes = Vec::new();
            let length_bytes = value_array.len().to_le_bytes();
            for byte in length_bytes.iter() {
                bytes.push(*byte);
            }
            let element_layout = layout.fields.as_ref().unwrap().first().unwrap();
            for element in value_array.iter() {
                let element_bytes = build_bytes_from_layout(&element_layout, element);
                for byte in element_bytes.iter() {
                    bytes.push(*byte);
                }
            }
            bytes
        }
        LayoutKind::Struct => {
            let mut bytes = Vec::new();
            let value_object = value.as_object().unwrap();
            for field in layout.fields.as_ref().unwrap().iter() {
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
            if value.as_bool().expect("Could not read value as bool") {
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

pub fn serialize(layout: &Layout, bytes: &Vec<u8>) -> String {
    let value = build_value_from_layout(layout, &bytes);
    serde_json::to_string(&value).unwrap()
}

pub fn deserialize(layout: &Layout, serial: &String) -> Vec<u8> {
    let value = serde_json::from_str::<Value>(serial).unwrap();
    build_bytes_from_layout(layout, &value)
}
