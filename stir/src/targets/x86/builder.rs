use crate::{
    common::{
        basicblock::{BBID, BasicBlock},
        builder::FunctionBuilder,
    },
    targets::x86::isa::{LLType, x86Instr},
};

pub type x86BB = BBID<x86Instr>;
pub type x86BasicBlock = BasicBlock<x86Instr>;
pub type x86Function = FunctionBuilder<x86Instr, LLType>;

impl x86Function {
    pub fn print(&self, include_comments: bool) {
        println!("{}:", self.name);
        self.dfs(|id, block| {
            println!("{id}:");
            for i in block.instructions.iter() {
                if matches!(i, x86Instr::Comment(..)) && !include_comments {
                    continue;
                }
                println!("\t{i}");
            }
            if let Some(term) = &block.terminator {
                println!("\t{term}");
            } else {
                println!("\t; !! (missing terminator)");
            }
            false
        });
    }
}
