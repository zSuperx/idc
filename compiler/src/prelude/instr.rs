use crate::prelude::*;

pub trait Instr {
    type Val;

    fn is_terminator(&self) -> bool;

    fn uncond_jump(target: BB) -> Self;

    fn dsts(&self) -> Vec<&Self::Val>;
    fn srcs(&self) -> Vec<&Self::Val>;
}
