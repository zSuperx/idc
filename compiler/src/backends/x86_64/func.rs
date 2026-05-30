use crate::backends::x86_64;

use x86_64::Instr::*;


#[allow(nonstandard_style)]
pub struct x86_64Function {
    instructions: Vec<x86_64::Instr>,
}

impl x86_64Function {
    pub fn new(instructions: Vec<x86_64::Instr>) -> Self {
        Self { instructions }
    }
}

impl std::fmt::Display for x86_64Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in self.instructions.iter() {
            match i {
                Raw(_) | Label(_) => {}
                _ => f.write_str("\t")?,
            }
            f.write_fmt(format_args!("{i}\n"))?;
        }
        Ok(())
    }
}
