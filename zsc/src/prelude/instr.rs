use crate::prelude::*;

pub trait Instr {
    type Val;

    fn is_terminator(&self) -> bool;

    fn uncond_jump(target: BB) -> Self;

    fn dsts(&mut self) -> impl Iterator<Item = &mut Self::Val>;
    fn srcs(&mut self) -> impl Iterator<Item = &mut Self::Val>;
}
