use std::fmt::{Debug, Display};

use crate::common::*;

pub trait Instr: Display + Debug + Clone {
    type Val;

    fn is_terminator(&self) -> bool;

    fn uncond_jump(target: BBID) -> Self;

    fn dsts(&mut self) -> impl Iterator<Item = &mut Self::Val>;
    fn srcs(&mut self) -> impl Iterator<Item = &mut Self::Val>;
}
