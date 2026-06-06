mod basicblock;
pub use basicblock::*;

mod func;
// pub use func::*;

mod instr;
pub use instr::*;

mod builder;
pub use builder::*;

mod env;
pub use env::*;

mod utils;
use registry::Id;
pub use utils::*;

pub use crate::autogen::*;

pub type StringId = Id<String>;
