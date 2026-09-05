use std::fmt::{Debug, Display};

use smallvec::SmallVec;

pub trait InstructionTrait: Display + Debug + Clone {
    type Val;

    fn is_terminator(&self) -> bool;
    fn defs(&self) -> SmallVec<[&Self::Val; 4]>;
    fn uses(&self) -> SmallVec<[&Self::Val; 4]>;
}
