#![allow(static_mut_refs)]
#![allow(nonstandard_style)]
#![allow(unused)]
#![warn(unused_imports)]
use std::{
    collections::HashSet,
    fmt::Write,
    process::Command,
    sync::{LazyLock, OnceLock},
};

use crate::{arch::x86, aux::Compiler};

mod arch;
mod ast;
mod autogen;
mod aux;
mod checker;
mod hir;
mod lower;
mod optimizer;
mod parser;
mod prelude;
mod tir;

macro_rules! die {
    ($($fmtargs:tt)*) => {{
        eprintln!($($fmtargs)*);
        ::std::process::exit(1);
    }};
}

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
    match cfg.target {
        Target::IR => {}
        Target::x86 => {
            buf.insert_str(0, "section .text\nglobal main\n");
        }
    }

    // Linearize
    for func in prog {
        match cfg.target {
            Target::IR => buf.write_fmt(format_args!("{}", func)).unwrap(),
            Target::x86 => {
                let mut emitter = x86::Emitter::new();
                let func = emitter.translate_func(func);
                buf.write_fmt(format_args!("{func}")).unwrap();
            }
        }
    }

    match cfg.action {
        Action::EmitAsm => {
            if cfg.output == "-" {
                println!("{buf}");
            } else {
                std::fs::write(&cfg.output, &buf).unwrap();
            }
            return;
        }

        Action::CompileOnly => {
            let asm = tempfile::NamedTempFile::new().unwrap();

            std::fs::write(asm.path(), &buf).unwrap();

            let status = Command::new("nasm")
                .arg("-felf64")
                .arg(asm.path())
                .arg("-o")
                .arg(&cfg.output)
                .status()
                .unwrap();

            if !status.success() {
                die!("assembly failed");
            }

            return;
        }

        Action::AssembleAndLink => {
            let asm = tempfile::NamedTempFile::new().unwrap();
            let obj = tempfile::NamedTempFile::new().unwrap();

            std::fs::write(asm.path(), &buf).unwrap();

            let status = Command::new("nasm")
                .arg("-felf64")
                .arg(asm.path())
                .arg("-o")
                .arg(obj.path())
                .status()
                .unwrap();

            if !status.success() {
                die!("assembling failed");
            }

            let status = Command::new("mold")
                .arg(obj.path())
                .arg("runtime/rt.o")
                .arg("-o")
                .arg(&cfg.output)
                .status()
                .unwrap();

            if !status.success() {
                die!("linking failed");
            }
        }
    }
}

#[allow(nonstandard_style)]
#[derive(Debug, Clone, Copy)]
pub enum Target {
    IR,
    x86,
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    EmitAsm = 1,
    CompileOnly = 2,
    AssembleAndLink = 3,
}

#[derive(Debug)]
struct Config {
    target: Target,
    input: String,
    output: String,
    action: Action,
}

fn parse_args() -> Config {
    // Default values
    let mut target = None;
    let mut input = None;
    let mut output = None;
    let mut action = None;

    // Setup args
    let mut args = std::env::args().peekable();
    let argv0 = args.next().unwrap();

    // Parse args
    while let Some(arg) = args.next() {
        let flag = arg.trim();
        match flag {
            "-h" | "--help" => {
                eprintln!("Usage: {argv0} <INPUT> [-o <OUTPUT>] [-t <TARGET>]");
                eprintln!();
                eprintln!("Available targets:");
                eprintln!("\t ir\tLinearized intermediate representation");
                eprintln!("\t x86\tx86 assembly");
                die!();
            }
            "-t" | "--target" => {
                let Some(target_str) = args.next() else {
                    die!("{flag} flag expects target");
                };
                let target_enum = match target_str.to_lowercase().as_str() {
                    "x86" => Target::x86,
                    _ => die!("Unknown target: {target_str}"),
                };

                if target.replace(target_enum).is_some() {
                    die!("Only 1 target allowed");
                }
            }
            "-E" => {
                if action.replace(Action::EmitAsm).is_some() {
                    die!("Only 1 action allowed. Pass -h for more info");
                }
                if target.replace(Target::IR).is_some() {
                    die!("-E cannot be used with -t");
                }
            }
            "-S" => {
                if action.replace(Action::EmitAsm).is_some() {
                    die!("Only 1 action allowed. Pass -h for more info");
                }
            }
            "-c" => {
                if action.replace(Action::CompileOnly).is_some() {
                    die!("Only 1 action allowed. Pass -h for more info");
                }
            }
            "-o" | "--output" => {
                let Some(output_str) = args.next() else {
                    die!("{flag} flag expects target");
                };
                if output.is_some() {
                    die!("Only 1 file can be provided as output");
                }
                output = Some(output_str);
            }
            _ => {
                if input.is_some() {
                    die!("Only 1 file can be provided as input");
                }
                input = Some(arg);
            }
        }
    }
    let action = action.unwrap_or(Action::AssembleAndLink);
    let target = target.unwrap_or(Target::x86);
    let Some(input) = input else {
        die!("No input file");
    };
    let default_output = {
        let ext = input.rfind(".");
        let base = input.get(..ext.unwrap_or(input.len())).unwrap();
        match action {
            Action::EmitAsm => match target {
                Target::IR => format!("{base}.ir"),
                _ => format!("{base}.s"),
            },
            Action::CompileOnly => format!("{base}.o"),
            Action::AssembleAndLink => base.to_string(),
        }
    };

    let output = output.unwrap_or(default_output);
    let c = Config {
        target,
        input,
        output,
        action,
    };
    c
}
