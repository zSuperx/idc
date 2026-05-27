use crate::lir::*;

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub name: Option<&'static str>,
    pub id: usize,
    pub instructions: Vec<Instr>,
    pub terminator: Instr,
}

impl BasicBlock {
    pub fn new(
        name: Option<&'static str>,
        id: usize,
        buf: Vec<Instr>,
        terminator: Instr,
    ) -> Self {
        Self {
            name,
            id,
            instructions: buf,
            terminator,
        }
    }
}
