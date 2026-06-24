use crate::ast::*;

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

impl From<&RealType> for LirType {
    fn from(value: &RealType) -> Self {
        match value {
            RealType::I8 | RealType::U8 => LirType::I8,
            RealType::I16 | RealType::U16 => LirType::I16,
            RealType::I32 | RealType::U32 => LirType::I32,
            RealType::I64 | RealType::U64 => LirType::I64,
            RealType::Pointer(id) => LirType::Ptr,
            RealType::Function { args, returns } => LirType::Ptr,
            RealType::Bool => LirType::I8,
            RealType::Void => panic!("Can't actually compile void type"),
            RealType::Base(_) => todo!(),
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
