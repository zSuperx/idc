use std::hash::Hash;

use registry::*;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum TirType {
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

impl TirType {
    pub fn is_integral(&self) -> bool {
        match self {
            TirType::I8
            | TirType::U8
            | TirType::I16
            | TirType::U16
            | TirType::I32
            | TirType::U32
            | TirType::I64
            | TirType::U64 => true,
            _ => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            TirType::I8 | TirType::I16 | TirType::I32 | TirType::I64 => true,
            _ => false,
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            TirType::Base(_) => todo!(),
            // Function types are coerced to pointers
            x => x.size(),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            TirType::I8 | TirType::U8 => 1,
            TirType::I16 | TirType::U16 => 2,
            TirType::I32 | TirType::U32 => 4,
            TirType::I64 | TirType::U64 => 8,
            TirType::Bool => 1,
            TirType::Void => 0,
            TirType::Base(_) => todo!(),
            TirType::Function { args, returns } => 8,
            TirType::Pointer(_) => 8,
        }
    }
}

impl std::fmt::Display for TirType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TirType::Base(s) => format_args!("{}", *s),
            TirType::Function { args, returns } => {
                format_args!(
                    "Fn({}) -> {}",
                    args.iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    *returns
                )
            }
            TirType::Pointer(type_id) => format_args!("*{}", *type_id),
            x => format_args!("{}", format!("{x:?}").to_lowercase()),
        };
        f.write_fmt(s)
    }
}

pub type TypeId = Id<TirType>;
