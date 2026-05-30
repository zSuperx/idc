use crate::{lir::BasicBlock, tir::VarId};

#[derive(Debug, Clone)]
pub struct LIRFunction {
    pub name: crate::tir::VarId,
    pub bbs: Vec<BasicBlock>,
    pub vreg_count: usize,
}

impl LIRFunction {
    pub fn new(name: VarId, bbs: Vec<BasicBlock>, vreg_count: usize) -> Self {
        Self {
            name,
            bbs,
            vreg_count,
        }
    }
}

impl std::fmt::Display for LIRFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bb in self.bbs.iter() {
            f.write_fmt(format_args!("{bb}"))?;
        }
        Ok(())
    }
}
