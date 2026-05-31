use std::{collections::HashMap, fmt::Display};

use crate::{
    ast::Spanned,
    aux::SymbolInfo,
    lir::{BB, BasicBlock, Instr, LirInstr, LirVal, VVal},
    tir::{TypeId, VarId},
    utils::Env,
};

#[derive(Debug, Default)]
pub struct FnCtx {
    pub name: Option<VarId>,
    pub raw_name: Spanned<&'static str>,
    pub env: Env<&'static str, (VarId, TypeId)>, // Tracks scopes and string -> var, type mappings
    pub return_type: Option<Spanned<TypeId>>,    // Return type of current function
    pub symbol_table: HashMap<VarId, SymbolInfo>,
    pub var2val: HashMap<VarId, VVal>,
}

#[derive(Debug)]
pub struct Builder<I> {
    pub name: &'static str,
    pub vreg_count: usize,
    // BBs we have built so far
    pub bbs: Vec<BasicBlock<I>>,
    pub bb_count: usize,
    // In-progress stuff for building BBs
    pub buf: Vec<I>,
    pub curr_bb_name: Option<BB>,
}

impl<I: Display> std::fmt::Display for Builder<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:\n", self.name))?;
        for bb in self.bbs.iter() {
            f.write_fmt(format_args!("{bb}"))?;
        }
        Ok(())
    }
}

impl<I> Default for Builder<I> {
    fn default() -> Self {
        Self {
            name: Default::default(),
            vreg_count: Default::default(),
            bbs: Default::default(),
            bb_count: Default::default(),
            buf: Default::default(),
            curr_bb_name: Default::default(),
        }
    }
}

impl<I: Clone + Instr> Builder<I> {
    pub fn new(name: &'static str, bb_count: usize, vreg_count: usize) -> Self {
        let mut s = Self::default();
        s.name = name;
        s.vreg_count = vreg_count;
        s.bb_count = bb_count;
        s
    }

    pub fn next_reg(&mut self) -> LirVal {
        let id = self.vreg_count;
        self.vreg_count += 1;
        LirVal::Reg(id)
    }

    pub fn next_bb(&mut self, name: &'static str) -> BB {
        self.bb_count += 1;
        BB(self.bb_count - 1, name)
    }

    pub fn start_new_block(&mut self, name: BB) {
        if let Some(old_name) = self.curr_bb_name {
            // Commit the old block, but first check if it terminated
            let tmp = self.get_terminator();
            let mut buf = std::mem::take(&mut self.buf);
            let terminator = match tmp {
                Some(t) => t,
                None => {
                    let chain = I::uncond_jump(name);
                    buf.push(chain.clone());
                    chain
                } // if it didn't terminate, hook it up to the new one
            };
            let bb = BasicBlock::new(old_name, buf, terminator);
            self.bbs.push(bb);
        }
        self.curr_bb_name = Some(name);
    }

    pub fn get_terminator(&self) -> Option<I> {
        self.buf.last().filter(|&i| i.is_terminator()).cloned()
    }

    pub fn emit(&mut self, instr: I) {
        if self.get_terminator().is_none() {
            self.buf.push(instr);
        }
    }
}
