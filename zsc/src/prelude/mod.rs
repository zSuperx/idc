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
#[macro_export]
pub use utils::*;
pub use crate::die;

use registry::Id;
pub type Symbol = Id<String>;
