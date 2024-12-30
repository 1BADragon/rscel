use syn::{GenericArgument, PathArguments, Type};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum DispatchArgType {
    Int,
    Uint,
    Double,
    Boolean,
    String,
    Bytes,
    Vec,
    Map,
    Timestamp,
    Duration,
    CelResult,
}

impl DispatchArgType {
    pub fn from_type(pat: &Type) -> Self {
        match pat {
            Type::Path(path) => {
                if path.qself.is_some() {
                    panic!("Path qualifiers not allowed");
                }

                match path.path.segments.last() {
                    Some(t) => match t.ident.to_string().as_str() {
                        "i64" => DispatchArgType::Int,
                        "u64" => DispatchArgType::Uint,
                        "f64" => DispatchArgType::Double,
                        "bool" => DispatchArgType::Boolean,
                        "String" => DispatchArgType::String,
                        "Vec" => match &t.arguments {
                            PathArguments::AngleBracketed(angle) => match angle.args.last() {
                                Some(GenericArgument::Type(t)) => match t {
                                    Type::Path(path) => match path.path.segments.last() {
                                        Some(t) => match t.ident.to_string().as_str() {
                                            "u8" => DispatchArgType::Bytes,
                                            "CelValue" => DispatchArgType::Vec,
                                            _ => panic!("Vec arg must be either CelValue or u8"),
                                        },
                                        _ => panic!("Empty Vec args"),
                                    },
                                    _ => panic!("Vec arg must be path"),
                                },
                                _ => panic!("Vec arg must be path"),
                            },
                            _ => panic!("Vec arg must be either CelValue or u8"),
                        },
                        "Map" => DispatchArgType::Map,
                        "DateTime" => DispatchArgType::Timestamp,
                        "Duration" => DispatchArgType::Duration,
                        "CelResult" => DispatchArgType::CelResult,
                        other => panic!("Unknown type: {}", other),
                    },
                    None => panic!("No type info"),
                }
            }
            _ => panic!("Only type paths are allowed for argument types"),
        }
    }

    pub fn mangle_sym(&self) -> char {
        match self {
            DispatchArgType::Int => 'i',
            DispatchArgType::Uint => 'u',
            DispatchArgType::Double => 'd',
            DispatchArgType::Boolean => 'b',
            DispatchArgType::String => 's',
            DispatchArgType::Bytes => 'p',
            DispatchArgType::Vec => 'v',
            DispatchArgType::Map => 'm',
            DispatchArgType::Timestamp => 't',
            DispatchArgType::Duration => 'y',
            DispatchArgType::CelResult => 'r',
        }
    }

    pub fn celvalue_enum(&self) -> &'static str {
        match self {
            DispatchArgType::Int => "Int",
            DispatchArgType::Uint => "UInt",
            DispatchArgType::Double => "Float",
            DispatchArgType::Boolean => "Bool",
            DispatchArgType::String => "String",
            DispatchArgType::Bytes => "Bytes",
            DispatchArgType::Vec => "List",
            DispatchArgType::Map => "Map",
            DispatchArgType::Timestamp => "TimeStamp",
            DispatchArgType::Duration => "Duration",
            _ => unreachable!(),
        }
    }
}
