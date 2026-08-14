use std::fmt::Display;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BB(pub usize, pub &'static str);

impl std::fmt::Display for BB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(".L{}", self.0))?;
        if !self.1.is_empty() {
            f.write_str(".")?;
            f.write_str(self.1)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BasicBlockOld<I> {
    pub name: BB,
    pub instructions: Vec<I>,
    pub succ: Vec<BB>,
    pub pred: Vec<BB>, // currently unused
    pub terminator: I,
}

impl<I> BasicBlockOld<I> {
    pub fn new(name: BB, instructions: Vec<I>, terminator: I) -> Self {
        Self {
            name,
            instructions,
            terminator,
            succ: Default::default(),
            pred: Default::default(),
        }
    }
}

impl<I: Display> std::fmt::Display for BasicBlockOld<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:\n", self.name))?;
        for i in self.instructions.iter() {
            f.write_fmt(format_args!("    {i}\n"))?;
        }
        Ok(())
    }
}
