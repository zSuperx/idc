mod basicblock;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

pub use basicblock::*;

mod instr;
pub use instr::*;

mod builder;
pub use builder::*;

mod env;
pub use env::*;

mod utils;
pub use crate::die;
pub use utils::*;

use registry::Id;
pub type Symbol = Id<String>;

pub enum MyOpt<T> {
    Some(T),
    None,
}

impl<T> DerefMut for MyOpt<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MyOpt::Some(t) => t,
            MyOpt::None => panic!("Dereferenced a None value"),
        }
    }
}

impl<T> Deref for MyOpt<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MyOpt::Some(t) => t,
            MyOpt::None => panic!("Dereferenced a None value"),
        }
    }
}

impl<T: Debug> Debug for MyOpt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Some(arg0) => f.debug_tuple("Some").field(arg0).finish(),
            Self::None => write!(f, "None"),
        }
    }
}

impl<T: Clone> Clone for MyOpt<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Some(arg0) => Self::Some(arg0.clone()),
            Self::None => Self::None,
        }
    }
}

impl<T: Copy> Copy for MyOpt<T> {}
