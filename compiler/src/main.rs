#![feature(trim_prefix_suffix)]
#![allow(unused)]
#![allow(static_mut_refs)]
use std::{
    collections::HashSet,
    fmt::Write,
    sync::{LazyLock, OnceLock},
};

use crate::{backends::x86_64, aux::Compiler};

mod ast;
mod backends;
mod checker;
mod aux;
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
    let cfg = parse_args();
    let source_code = std::fs::read(cfg.input).unwrap();
    SOURCE.set(source_code).unwrap();

    let mut comp = Compiler::new();
    let prog = comp.compile_prog();

    let mut buf = String::new();
    for func in prog {
        match cfg.target {
            Target::IR => buf.write_fmt(format_args!("{func}")).unwrap(),
            Target::x86 => {
                let mut emitter = x86_64::Emitter::new();
                let func = emitter.translate_func(func);
                buf.write_fmt(format_args!("{func}")).unwrap();
            }
        }
    }

    match cfg.target {
        Target::IR => {}
        Target::x86 => {
            buf.insert_str(0, "section .text\nglobal main\n");
        }
    }

    if cfg.output == "-" {
        println!("{buf}");
    } else {
        std::fs::write(cfg.output, buf).unwrap();
    }
}

#[allow(nonstandard_style)]
pub enum Target {
    IR,
    x86,
}

struct Config {
    target: Target,
    input: String,
    output: String,
}

macro_rules! die {
    ($($fmtargs:tt)*) => {{
        eprintln!($($fmtargs)*);
        ::std::process::exit(1);
    }};
}

fn parse_args() -> Config {
    // Default values
    let mut target = Target::x86;
    let mut input = None;
    let mut output = None;

    // Setup args
    let mut args = std::env::args().peekable();
    let argv0 = args.next().unwrap();

    // Parse args
    while let Some(arg) = args.next() {
        match arg.trim() {
            flag @ ("-h" | "--help") => {
                eprintln!("Usage: {argv0} <INPUT> [-o <OUTPUT>] [-t <TARGET>]");
                eprintln!();
                eprintln!("Available targets:");
                eprintln!("\t ir\tLinearized intermediate representation");
                eprintln!("\t x86\tx86 assembly");
                die!();
            }
            flag @ ("-t" | "--target") => {
                let Some(target_str) = args.next() else {
                    die!("{flag} flag expects target");
                };
                target = match target_str.to_lowercase().as_str() {
                    "x86" => Target::x86,
                    "ir" => Target::IR,
                    _ => die!("Unknown target: {target_str}"),
                }
            }
            flag @ ("-o" | "--output") => {
                let Some(output_str) = args.next() else {
                    die!("{flag} flag expects target");
                };
                if output.is_some() {
                    die!("Only 1 file can be provided as output");
                }
                output = Some(output_str);
            }
            s => {
                if input.is_some() {
                    die!("Only 1 file can be provided as input");
                }
                input = Some(arg);
            }
        }
    }

    let Some(input) = input else {
        die!("No input file");
    };
    let default_output = {
        let ext = input.rfind(".");
        let base = input.get(..ext.unwrap_or(input.len())).unwrap();
        format!("{base}.s")
    };
    let output = output.unwrap_or(default_output);
    Config {
        target,
        input,
        output,
    }
}
