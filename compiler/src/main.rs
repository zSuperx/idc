#![allow(unused)]
#![allow(static_mut_refs)]
use std::{
    collections::HashSet,
    sync::{LazyLock, OnceLock},
};

use crate::{backends::x86_64, compiler::Compiler};

mod ast;
mod backends;
mod checker;
mod compiler;
mod hir;
mod lir;
mod lower;
mod optimizer;
mod parser;
mod tir;
mod utils;

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
    let prog = comp.compile_prog();

    for func in prog {
        println!("\n=== IR ===\n");
        println!("{func}");

        println!("\n=== x86_64 IR ===\n");
        let mut emitter = x86_64::Emitter::new();
        let func_x86_64 = emitter.translate_func(func);
        // println!("{func_x86_64}");
    }
}
