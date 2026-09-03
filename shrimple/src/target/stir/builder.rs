use crate::{
    common::{
        basicblock::{BBID, BasicBlock},
        builder::FunctionBuilder,
    },
    stir::isa::{IRValue, VReg},
    target::stir::isa::{IRInstr, IRType},
};

pub type IRBB = BBID<IRInstr>;
pub type IRBasicBlock = BasicBlock<IRInstr>;
pub type IRFunction = FunctionBuilder<IRInstr, IRValue, IRType>;

impl IRFunction {
    pub fn nextReg(&mut self) -> VReg {
        let ret = self.reg_count;
        self.reg_count += 1;
        ret
    }

    pub fn print(&mut self, include_comments: bool) {
        println!(
            "{}({}):",
            self.name,
            self.args
                .iter()
                .map(|(name, ty)| format!("{name}: {ty}"))
                .collect::<Vec<String>>()
                .join(", ")
        );
        self.dfs(|mcf, curr_id| {
            println!("{curr_id}:");
            let block = &mcf.blocks[&curr_id];
            for i in block.instructions.iter() {
                if matches!(i, IRInstr::Comment(..)) && !include_comments {
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
