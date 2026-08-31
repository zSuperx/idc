use std::fmt::Display;

use registry::Id;

use crate::common::builder::StructId;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum IRType {
    I1,
    I8,
    I16,
    I32,
    I64,
    Ptr,
    Struct(StructId),
}

impl Display for IRType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{self:?}").to_lowercase();
        f.write_str(&s)
    }
}

impl IRType {
    pub fn is_pointer(&self) -> bool {
        matches!(self, Self::Ptr)
    }

    pub fn bits(&self) -> usize {
        match self {
            IRType::I1 => 8,
            IRType::I8 => 8,
            IRType::I16 => 16,
            IRType::I32 => 32,
            IRType::I64 | IRType::Ptr => 64,
            IRType::Struct(s) => s.size * 8,
        }
    }

    pub fn bytes(&self) -> usize {
        self.bits() / 8
    }
}
