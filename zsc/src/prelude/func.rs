use crate::{ast::Spanned};
use crate::prelude::*;

#[allow(nonstandard_style)]
pub struct Function<I> {
    name: Spanned<&'static str>,
    bbs: Vec<BasicBlock<I>>,
}

impl<I> Function<I> {
    pub fn new(name: Spanned<&'static str>, bbs: Vec<BasicBlock<I>>) -> Self {
        Self { name, bbs }
    }
}

impl<I: std::fmt::Display> std::fmt::Display for Function<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:\n", self.name.inner))?;
        for bb in self.bbs.iter() {
            f.write_fmt(format_args!("{bb}"))?;
        }
        f.write_str("\n")?;
        Ok(())
    }
}
