use crate::tir::{TirType, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LirType {
    I8,
    I16,
    I32,
    I64,
    Ptr,
}

impl LirType {
    pub fn size(&self) -> usize {
        match self {
            LirType::I8 => 1,
            LirType::I16 => 2,
            LirType::I32 => 4,
            LirType::I64 => 8,
            LirType::Ptr => 8,
        }
    }

    pub fn alignment(&self) -> usize {
        self.size()
    }
}

impl From<&TirType> for LirType {
    fn from(value: &TirType) -> Self {
        match value {
            TirType::I8 | TirType::U8 => LirType::I8,
            TirType::I16 | TirType::U16 => LirType::I16,
            TirType::I32 | TirType::U32 => LirType::I32,
            TirType::I64 | TirType::U64 => LirType::I64,
            TirType::Pointer(id) => LirType::Ptr,
            TirType::Function { args, returns } => LirType::Ptr,
            TirType::Bool => LirType::I8,
            TirType::Void => panic!("Can't actually compile void type"),
            TirType::Base(_) => todo!(),
        }
    }
}

impl From<TypeId> for LirType {
    fn from(value: TypeId) -> Self {
        value.lookup().into()
    }
}

impl std::fmt::Display for LirType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}").to_lowercase())
    }
}
