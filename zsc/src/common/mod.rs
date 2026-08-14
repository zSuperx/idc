mod instr;
pub use instr::*;

// mod _basicblock_old;
// mod builder_old;
// pub use builder_old::*;

mod builder;
pub use builder::*;

mod value;
pub use value::*;

mod env;
pub use env::*;

pub use crate::die;
mod utils;
#[allow(unused_imports)]
pub use utils::*;

use registry::Id;
pub type Symbol = Id<String>;
