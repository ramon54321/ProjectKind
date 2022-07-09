use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, ExprCall, ItemStruct, PathArguments, Type};

//
// Type -> Kind -> Layout
//
// u8 -> Primative("U8") -> ...
// String -> Primative("String") -> ...
// Vec<u8> -> Array(Primative("U8")) -> ...
// Vec<Vec<u8>> -> Array(Array(Primative("U8"))) -> ...
//

#[derive(Debug, Clone)]
enum Kind {
    Primative(String),
    Array(Box<Kind>),
    Struct(String),
}

fn type_string_to_kind_string(type_string: &str) -> String {
    match type_string {
        "Vec" => "Array",
        "String" => "String",
        "bool" => "Bool",
        "u8" => "U8",
        "u16" => "U16",
        "u32" => "U32",
        "u64" => "U64",
        "i8" => "I8",
        "i16" => "I16",
        "i32" => "I32",
        "i64" => "I64",
        "f32" => "F32",
        "f64" => "F64",
        _ => {
            eprintln!(
                "Field type not supported in layout kind conversion: {}",
                type_string.to_string()
            );
            panic!()
        }
    }
    .to_string()
}

fn type_to_kind(field_type: Type) -> Kind {
    let path_segment = match field_type.clone() {
        Type::Path(type_path) => {
            let path = type_path.path.to_owned();
            let last_path_segment = path
                .segments
                .last()
                .expect("Could not get last path segment of field type");
            last_path_segment.clone()
        }
        _ => panic!("Field type is not of AST type Type::Path"),
    };
    let type_ident = path_segment.ident.to_string();
    match type_ident.as_str() {
        "Vec" => {
            let argument_kinds = match path_segment.arguments {
                PathArguments::AngleBracketed(generic_arguments) => {
                    let mut argument_kinds = Vec::new();
                    for argument in generic_arguments.args.iter() {
                        match argument {
                            syn::GenericArgument::Type(argument_type) => {
                                let argument_kind = type_to_kind(argument_type.to_owned());
                                argument_kinds.push(argument_kind);
                            }
                            _ => panic!("Unknown argument type"),
                        }
                    }
                    argument_kinds
                }
                _ => panic!("Non angle bracketed arguments not supported"),
            };
            Kind::Array(Box::new(
                argument_kinds
                    .first()
                    .expect("Could not get first argument kind")
                    .clone(),
            ))
        }
        "String" | "Bool" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32"
        | "f64" => Kind::Primative(type_string_to_kind_string(&type_ident)),
        struct_name => Kind::Struct(String::from(struct_name)),
    }
}

fn kind_to_layout(kind: Kind, layout_name: String) -> proc_macro2::TokenStream {
    match kind {
        Kind::Primative(kind_string) => {
            let field_kind_enum_string =
                syn::parse_str::<syn::ExprPath>(format!("LayoutKind::{}", kind_string).as_str())
                    .expect("Could not parse enum expression from layout kind and kind string");
            quote! {
                Layout {
                    name: String::from(#layout_name),
                    kind: #field_kind_enum_string,
                    fields: None,
                }
            }
        }
        Kind::Array(child_kind) => {
            let child_layout = kind_to_layout(*child_kind, String::from("unnamed"));
            quote! {
                Layout {
                    name: String::from(#layout_name),
                    kind: LayoutKind::Array,
                    fields: Some(vec![
                        #child_layout
                    ]),
                }
            }
        }
        Kind::Struct(struct_name_string) => {
            let function_call = syn::parse_str::<ExprCall>(
                format!("{}::get_layout()", struct_name_string).as_str(),
            )
            .expect("Could not parse function call string");
            quote! { #function_call }
        }
    }
}

fn item_struct_to_layout_token_stream(item_struct: ItemStruct) -> proc_macro2::TokenStream {
    let mut field_layout_token_streams = Vec::new();
    for field in item_struct.fields.iter() {
        let field_name = field
            .ident
            .clone()
            .expect("Could not get field ident")
            .to_string();
        let field_kind = type_to_kind(field.ty.clone());
        let field_layout = kind_to_layout(field_kind, field_name);
        field_layout_token_streams.push(field_layout);
    }
    let struct_name = item_struct.ident.clone().to_string();
    let layout_token_stream = quote! {
        Layout {
            name: String::from(#struct_name),
            kind: LayoutKind::Struct,
            fields: Some(
                vec![
                    #(#field_layout_token_streams),*
                ]
            ),
        }
    };
    layout_token_stream
}

#[proc_macro_attribute]
pub fn component(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct =
        parse::<ItemStruct>(TokenStream::from(input)).expect("Could not parse item struct");
    let item_struct_name = item_struct.ident.clone();
    let item_struct_layout_token_stream = item_struct_to_layout_token_stream(item_struct.clone());
    let expanded = quote! {
        #[repr(C)]
        //#[derive(serde::Serialize, serde::Deserialize)]
        #item_struct
        impl project_kind::HasLayout for #item_struct_name {
            fn get_layout() -> Layout {
                #item_struct_layout_token_stream
            }
        }
    };
    TokenStream::from(expanded)
}
