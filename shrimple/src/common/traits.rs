use std::fmt::{Debug, Display};

pub trait InstructionTrait: Display + Debug + Clone {
    type Val;

    fn is_terminator(&self) -> bool;
    fn dsts(&mut self) -> impl Iterator<Item = &mut Self::Val>;
    fn srcs(&mut self) -> impl Iterator<Item = &mut Self::Val>;
}
