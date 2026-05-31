use crate::{ast::Spanned, backends::x86_64, lir::BasicBlock};

use x86_64::x86Instr;

#[allow(nonstandard_style)]
pub struct x86Function {
    name: Spanned<&'static str>,
    bbs: Vec<BasicBlock<x86Instr>>
}

impl x86Function {
    pub fn new(name: Spanned<&'static str>, bbs: Vec<BasicBlock<x86Instr>>) -> Self {
        Self { name, bbs }
    }
}

impl std::fmt::Display for x86Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:\n", self.name.inner))?;
        for bb in self.bbs.iter() {
            f.write_fmt(format_args!("{bb}"))?;
        }
        f.write_str("\n")?;
        Ok(())
    }
}

