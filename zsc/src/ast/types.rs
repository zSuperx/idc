use std::hash::Hash;

use registry::*;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum RawType {
    Base(&'static str),
    Pointer(RawTypeId),
    Function {
        args: Vec<RawTypeId>,
        returns: RawTypeId,
    },
}

pub type RawTypeId = Id<RawType>;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum ResolvedType {
    Unknown,

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
    Function {
        args: Vec<ResolvedTypeId>,
        returns: ResolvedTypeId,
    },
    Pointer(ResolvedTypeId),
}

impl ResolvedType {
    pub fn is_integral(&self) -> bool {
        match self {
            ResolvedType::I8
            | ResolvedType::U8
            | ResolvedType::I16
            | ResolvedType::U16
            | ResolvedType::I32
            | ResolvedType::U32
            | ResolvedType::I64
            | ResolvedType::U64 => true,
            _ => false,
        }
    }

    pub fn is_primitive(&self) -> bool {
        match self {
            ResolvedType::I8
            | ResolvedType::U8
            | ResolvedType::I16
            | ResolvedType::U16
            | ResolvedType::I32
            | ResolvedType::U32
            | ResolvedType::I64
            | ResolvedType::U64
            | ResolvedType::Bool
            | ResolvedType::Void => true,
            _ => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            ResolvedType::I8 | ResolvedType::I16 | ResolvedType::I32 | ResolvedType::I64 => true,
            _ => false,
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            ResolvedType::Base(_) => todo!(),
            // Function types are coerced to pointers
            x => x.size(),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            ResolvedType::I8 | ResolvedType::U8 => 1,
            ResolvedType::I16 | ResolvedType::U16 => 2,
            ResolvedType::I32 | ResolvedType::U32 => 4,
            ResolvedType::I64 | ResolvedType::U64 => 8,
            ResolvedType::Bool => 1,
            ResolvedType::Void => 0,
            ResolvedType::Base(_) => todo!(),
            ResolvedType::Function { args, returns } => 8,
            ResolvedType::Pointer(_) => 8,
            Self::Unknown => panic!(),
        }
    }
}

impl std::fmt::Display for ResolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ResolvedType::Base(s) => format_args!("{}", *s),
            ResolvedType::Function { args, returns } => {
                format_args!(
                    "Fn({}) -> {}",
                    args.iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    *returns
                )
            }
            ResolvedType::Pointer(type_id) => format_args!("*{}", *type_id),
            x => format_args!("{}", format!("{x:?}").to_lowercase()),
        };
        f.write_fmt(s)
    }
}

pub type ResolvedTypeId = Id<ResolvedType>;
