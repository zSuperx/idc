use std::hash::Hash;

use registry::*;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum Type {
    Unresolved(&'static str),

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
        args: Vec<TypeId>,
        returns: TypeId,
    },
    Pointer(TypeId),
}

impl Type {
    pub fn is_integral(&self) -> bool {
        match self {
            Type::I8
            | Type::U8
            | Type::I16
            | Type::U16
            | Type::I32
            | Type::U32
            | Type::I64
            | Type::U64 => true,
            _ => false,
        }
    }

    pub fn is_primitive(&self) -> bool {
        match self {
            Type::I8
            | Type::U8
            | Type::I16
            | Type::U16
            | Type::I32
            | Type::U32
            | Type::I64
            | Type::U64
            | Type::Bool
            | Type::Void => true,
            _ => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => true,
            _ => false,
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            Type::Base(_) => todo!(),
            // Function types are coerced to pointers
            x => x.size(),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Type::I8 | Type::U8 => 1,
            Type::I16 | Type::U16 => 2,
            Type::I32 | Type::U32 => 4,
            Type::I64 | Type::U64 => 8,
            Type::Bool => 1,
            Type::Void => 0,
            Type::Base(_) => todo!(),
            Type::Function { args, returns } => 8,
            Type::Pointer(_) => 8,
            Self::Unresolved(..) => panic!(),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Type::Base(s) => format_args!("{}", *s),
            Type::Function { args, returns } => {
                format_args!(
                    "Fn({}) -> {}",
                    args.iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    *returns
                )
            }
            Type::Pointer(type_id) => format_args!("*{}", *type_id),
            x => format_args!("{}", format!("{x:?}").to_lowercase()),
        };
        f.write_fmt(s)
    }
}

pub type TypeId = Id<Type>;
