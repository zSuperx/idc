#![allow(nonstandard_style)]
#![allow(clippy::upper_case_acronyms)]
#![allow(unused)]

pub(crate) mod common;
pub(crate) mod targets;

pub use targets::stir::builder;
pub use targets::stir::isa;

pub use targets::x86::Backend as x86Backend;
