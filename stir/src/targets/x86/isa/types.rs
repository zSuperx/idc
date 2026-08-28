#[derive(Clone, Debug, Copy)]
pub enum LLType {
    I1,
    I8,
    I16,
    I32,
    I64,
}

impl LLType {
    pub fn bytes(&self) -> usize {
        self.bits().div_ceil(8)
    }

    pub fn bits(&self) -> usize {
        match self {
            Self::I1 => 8,
            LLType::I8 => 8,
            LLType::I16 => 16,
            LLType::I32 => 32,
            LLType::I64 => 64,
        }
    }

    pub fn width_str(&self) -> &'static str {
        match self {
            Self::I1 => "FLAG",
            LLType::I8 => "byte ",
            LLType::I16 => "word ",
            LLType::I32 => "dword ",
            LLType::I64 => "qword ",
        }
    }
}
