use crate::{
    common::{
        basicblock::{BBID, BasicBlock},
        builder::FunctionBuilder,
    },
    target::x86::isa::{LLType, x86Instr, x86Value},
};

pub type x86BB = BBID<x86Instr>;
pub type x86BasicBlock = BasicBlock<x86Instr>;
pub type x86Function = FunctionBuilder<x86Instr, x86Value, LLType>;

impl x86Function {
    pub fn print(&mut self, include_comments: bool) {
        println!("{}:", self.name);
        self.dfs(|mcf, curr_id| {
            println!("{curr_id}:");
            let block = &mcf.blocks[&curr_id];
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
        });
    }
}


