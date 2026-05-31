use crate::{lir::*, tir::VarId};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BB(pub VarId, pub usize, pub &'static str);

impl std::fmt::Display for BB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id: usize = self.0.into();
        f.write_fmt(format_args!(".F{id}.{}", self.1))?;
        if !self.2.is_empty() {
            f.write_str(".")?;
            f.write_str(self.2)?;
        }
        Ok(())
    }
}

pub enum Terminator {
    Fallthrough,
    Instr(Instr),
    None,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub name: BB,
    pub instructions: Vec<Instr>,
    pub terminator: Instr,
}

impl BasicBlock {
    pub fn new(name: BB, instructions: Vec<Instr>, terminator: Instr) -> Self {
        Self {
            name,
            instructions,
            terminator,
        }
    }
}

impl std::fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:\n", self.name))?;
        for i in self.instructions.iter() {
            f.write_fmt(format_args!("\t{i}\n"))?;
        }
        Ok(())
    }
}
