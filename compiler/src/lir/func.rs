use std::collections::HashMap;

use crate::{
    ast::Spanned,
    compiler::SymbolInfo,
    lir::{BB, BasicBlock, Instr, LirVal, VVal},
    tir::{TypeId, VarId},
    utils::Env,
};

#[derive(Debug, Default)]
pub struct LIRFunction {
    pub name: Option<VarId>,
    pub vreg_count: usize,

    pub raw_name: Spanned<&'static str>,
    pub env: Env<&'static str, (VarId, TypeId)>, // Tracks scopes and string -> var, type mappings
    pub return_type: Option<Spanned<TypeId>>,    // Return type of current function
    pub symbol_table: HashMap<VarId, SymbolInfo>,
    pub var2val: HashMap<VarId, VVal>,

    // BBs we have built so far
    pub bbs: Vec<BasicBlock>,
    // In-progress stuff for building BBs
    pub buf: Vec<Instr>,
    pub curr_bb_name: Option<BB>,
}

impl LIRFunction {
    pub fn next_reg(&mut self) -> LirVal {
        let id = self.vreg_count;
        self.vreg_count += 1;
        LirVal::Reg(id)
    }

    pub fn start_new_block(&mut self, name: BB) {
        if let Some(old_name) = self.curr_bb_name {
            // Commit the old block, but first check if it terminated
            let buf = std::mem::take(&mut self.buf);
            let terminator = match self.get_terminator() {
                Some(t) => t,
                None => Instr::Jmp(name), // if it didn't terminate, hook it up to the new one
            };
            let bb = BasicBlock::new(old_name, buf, terminator);
            self.bbs.push(bb);
        }
        self.curr_bb_name = Some(name);
    }

    pub fn get_terminator(&self) -> Option<Instr> {
        match self.buf.last() {
            Some(i) => match i {
                Instr::RetVoid | Instr::Ret(..) | Instr::Br(..) | Instr::Jmp(..) => Some(*i),
                _ => None,
            },
            None => None,
        }
    }

    pub fn emit(&mut self, instr: Instr) {
        if self.get_terminator().is_none() {
            self.buf.push(instr);
        }
    }
}

impl std::fmt::Display for LIRFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:\n", self.raw_name.inner))?;
        for bb in self.bbs.iter() {
            f.write_fmt(format_args!("{bb}"))?;
        }
        Ok(())
    }
}
