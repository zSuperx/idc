#![allow(unused)]
#![allow(static_mut_refs)]
use std::{
    collections::HashSet,
    sync::{LazyLock, OnceLock},
};

use crate::compiler::Compiler;

mod ast;
mod checker;
mod compiler;
mod hir;
mod lir;
mod lower;
mod optimizer;
mod parser;
mod tir;
mod token;

pub static SOURCE: OnceLock<Vec<u8>> = OnceLock::new();
pub static mut STRINGS: LazyLock<HashSet<String>> = LazyLock::new(HashSet::new);

pub fn source() -> &'static Vec<u8> {
    SOURCE.get().unwrap()
}

pub fn add_str(value: &str) -> &'static str {
    unsafe {
        if let Some(s) = STRINGS.get(value) {
            s
        } else {
            STRINGS.insert(value.to_string());
            STRINGS.get(value).unwrap()
        }
    }
}

fn main() {
    let mut args = std::env::args();
    let _ = args.next().unwrap();
    let infile = args.next().unwrap();

    let source_code = std::fs::read(infile).unwrap();

    SOURCE.set(source_code).unwrap();

    let mut comp = Compiler::new();
    let f = comp.parse_obj();
    let f = comp.check_obj(f);
    let mut lir_buf = vec![];
    let bbs = comp.lower_obj(&mut lir_buf, f);

    for mut bb in bbs {
        comp.const_fold_bb(&mut bb);
        for i in bb.instructions.iter() {
            println!("{i}");
        }
    }
}
