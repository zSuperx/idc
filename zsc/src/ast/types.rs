use std::hash::Hash;

use registry::*;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum RawType {
    Named(&'static str),
    Pointer(Box<RawType>),
    Function {
        args: Vec<RawType>,
        returns: Box<RawType>,
    },
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum RealType {
    // Primitive types
    I8,
    I16,
    I32,
    I64,

    U8,
    U16,
    U32,
    U64,

    Bool,
    Void,

    // Rest
    Base(&'static str),
    Function { args: Vec<TypeId>, returns: TypeId },
    Pointer(TypeId),
}

impl RealType {
    pub fn is_integral(&self) -> bool {
        match self {
            RealType::I8
            | RealType::U8
            | RealType::I16
            | RealType::U16
            | RealType::I32
            | RealType::U32
            | RealType::I64
            | RealType::U64 => true,
            _ => false,
        }
    }

    pub fn is_primitive(&self) -> bool {
        match self {
            RealType::I8
            | RealType::U8
            | RealType::I16
            | RealType::U16
            | RealType::I32
            | RealType::U32
            | RealType::I64
            | RealType::U64
            | RealType::Bool
            | RealType::Void => true,
            _ => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            RealType::I8 | RealType::I16 | RealType::I32 | RealType::I64 => true,
            _ => false,
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            RealType::Base(_) => todo!(),
            // Function types are coerced to pointers
            x => x.size(),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            RealType::I8 | RealType::U8 => 1,
            RealType::I16 | RealType::U16 => 2,
            RealType::I32 | RealType::U32 => 4,
            RealType::I64 | RealType::U64 => 8,
            RealType::Bool => 1,
            RealType::Void => 0,
            RealType::Base(_) => todo!(),
            RealType::Function { args, returns } => 8,
            RealType::Pointer(_) => 8,
        }
    }
}

impl std::fmt::Display for RealType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RealType::Base(s) => format_args!("{}", *s),
            RealType::Function { args, returns } => {
                format_args!(
                    "Fn({}) -> {}",
                    args.iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    *returns
                )
            }
            RealType::Pointer(type_id) => format_args!("*{}", *type_id),
            x => format_args!("{}", format!("{x:?}").to_lowercase()),
        };
        f.write_fmt(s)
    }
}

pub type TypeId = Id<RealType>;
