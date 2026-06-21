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

use clap::{Parser, ValueEnum};

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
pub static CFG: LazyLock<Config> = LazyLock::new(validate_config);
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
    let source_code = std::fs::read(&CFG.input).unwrap();
    SOURCE.set(source_code).unwrap();

    let mut comp = Compiler::new();
    let prog = comp.compile_prog();

    let mut buf = String::new();
    match CFG.target {
        Target::x86 => {
            buf.insert_str(0, "section .text\nglobal main\n");
        }
    }

    // Linearize
    for func in prog {
        match CFG.action {
            Action::EmitIr => buf.write_fmt(format_args!("{}", func)).unwrap(),
            _ => match CFG.target {
                Target::x86 => {
                    let mut emitter = x86::Emitter::new();
                    let func = emitter.translate_func(func);
                    buf.write_fmt(format_args!("{func}")).unwrap();
                }
            },
        }
    }

    match CFG.action {
        Action::EmitIr | Action::EmitAsm => {
            if CFG.output == "-" {
                println!("{buf}");
            } else {
                std::fs::write(&CFG.output, &buf).unwrap();
            }
            return;
        }

        Action::CompileOnly => {
            if CFG.output == "-" {
                die!("Can only output to stdout when emitting assembly or IR (see -S or -E)");
            }
            let asm = tempfile::NamedTempFile::new().unwrap();

            std::fs::write(asm.path(), &buf).unwrap();

            let status = Command::new("nasm")
                .arg("-felf64")
                .arg(asm.path())
                .arg("-o")
                .arg(&CFG.output)
                .status()
                .unwrap();

            if !status.success() {
                die!("assembly failed");
            }

            return;
        }

        Action::AssembleAndLink => {
            if CFG.output == "-" {
                die!("Can only output to stdout when emitting assembly or IR (see -S or -E)");
            }
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
                .arg(&CFG.output)
                .status()
                .unwrap();

            if !status.success() {
                die!("linking failed");
            }
        }
    }
}

#[allow(nonstandard_style)]
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Target {
    x86,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    EmitIr,

    EmitAsm,

    CompileOnly,

    AssembleAndLink,
}

pub struct Config {
    target: Target,
    input: String,
    output: String,
    action: Action,
    verbose: bool,
    do_reg_alloc: bool,
}

fn validate_config() -> Config {
    let args = Args::parse();

    let mut action_opt = None;

    if args.emit_ir {
        if action_opt.replace(Action::EmitIr).is_some() {
            die!("Only 1 action can be performed. See -h");
        }
    }

    if args.emit_asm {
        if action_opt.replace(Action::EmitAsm).is_some() {
            die!("Only 1 action can be performed. See -h");
        }
    }

    if args.compile_only {
        if action_opt.replace(Action::CompileOnly).is_some() {
            die!("Only 1 action can be performed. See -h");
        }
    }

    let action = action_opt.unwrap_or(Action::AssembleAndLink);

    let target = args.target.unwrap_or(Target::x86);
    let input = args.input;
    let default_output = {};
    let output = args.output.unwrap_or_else(|| {
        let ext = input.rfind(".");
        let base = input.get(..ext.unwrap_or(input.len())).unwrap();
        match action {
            Action::EmitIr => format!("{base}.ir"),
            Action::EmitAsm => format!("{base}.s"),
            Action::CompileOnly => format!("{base}.o"),
            Action::AssembleAndLink => base.to_string(),
        }
    });
    let verbose = args.verbose;
    let do_reg_alloc = if args.no_reg_alloc {
        if action != Action::EmitAsm {
            die!("--no-reg-alloc can only be used with -S");
        }
        false
    } else {
        true
    };

    Config {
        target,
        input,
        output,
        action,
        verbose,
        do_reg_alloc,
    }
}

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    target: Option<Target>,

    input: String,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    verbose: bool,

    /// Emit IR, do not compile, assemble, or link
    #[arg(short = 'E')]
    emit_ir: bool,

    /// Compile, do not assemble or link
    #[arg(short = 'S')]
    emit_asm: bool,

    /// Compile and assemble, do not link
    #[arg(short = 'c')]
    compile_only: bool,

    /// Skips the register allocation phase. This is only valid if using -S
    #[arg(long)]
    no_reg_alloc: bool,
}

// fn parse_args() -> Config {
//     // Default values
//     let mut target = None;
//     let mut input = None;
//     let mut output = None;
//     let mut action = None;
//     let mut verbose = None;
//
//     // Setup args
//     let mut args = std::env::args().peekable();
//     let argv0 = args.next().unwrap();
//
//     // Parse args
//     while let Some(arg) = args.next() {
//         let flag = arg.trim();
//         match flag {
//             "-h" | "--help" => {
//                 die!(
//                     r"zsc: The official zSuper compiler.
//
// Usage: {argv0} <INPUT> <ACTION> [OPTIONS]
//
//
// Actions:
//     -E                          Emit IR, do not compile, assemble, or link.
//
//     -S                          Compile only, do not assemble or link.
//
//     -c                          Compile and assemble only, do not link.
//
//     (Note: default action is to compile, assemble, and link)
//
// Options:
//     -o, --output <OUTPUT>       Write output to <OUTPUT>. Defaults to <INPUT> with
//                                 `.zs` stripped if omitted.
//
//     -t, --target <TARGET>       Specifies target to compile to. Available targets:
//                                    x86_64
//
//     -v, --verbose               Be verbose in stdout via logging and
//                                 outputted assembly/IR via comments.
//
//     --no-regalloc               Do not perform register allocation. This means the
//                                 program cannot be assembled. Useful for debugging.
//
//     -h, --help                  Prints this message
// "
//                 );
//             }
//             "-t" | "--target" => {
//                 let Some(target_str) = args.next() else {
//                     die!("{flag} flag expects target");
//                 };
//                 let target_enum = match target_str.to_lowercase().as_str() {
//                     "x86_64" => Target::x86,
//                     _ => die!("Unknown target: {target_str}"),
//                 };
//
//                 if target.replace(target_enum).is_some() {
//                     die!("Only 1 target allowed");
//                 }
//             }
//             "-E" => {
//                 if action.replace(Action::EmitAsm).is_some() {
//                     die!("Only 1 action allowed. Pass -h for more info");
//                 }
//                 if target.replace(Target::IR).is_some() {
//                     die!("-E cannot be used with -t");
//                 }
//             }
//             "-S" => {
//                 if action.replace(Action::EmitAsm).is_some() {
//                     die!("Only 1 action allowed. Pass -h for more info");
//                 }
//             }
//             "-c" => {
//                 if action.replace(Action::CompileOnly).is_some() {
//                     die!("Only 1 action allowed. Pass -h for more info");
//                 }
//             }
//             "-o" | "--output" => {
//                 let Some(output_str) = args.next() else {
//                     die!("{flag} flag expects target");
//                 };
//                 if output.is_some() {
//                     die!("Only 1 file can be provided as output");
//                 }
//                 output = Some(output_str);
//             }
//             "-v" | "--verbose" => {
//                 verbose = Some(true);
//             }
//             "--no-regalloc" => {
//
//             }
//             _ => {
//                 if input.is_some() {
//                     die!("Only 1 file can be provided as input");
//                 }
//                 input = Some(arg);
//             }
//         }
//     }
//     let action = action.unwrap_or(Action::AssembleAndLink);
//     let target = target.unwrap_or(Target::x86);
//     let Some(input) = input else {
//         die!("No input file");
//     };
//     let default_output = {
//         let ext = input.rfind(".");
//         let base = input.get(..ext.unwrap_or(input.len())).unwrap();
//         match action {
//             Action::EmitAsm => match target {
//                 Target::IR => format!("{base}.ir"),
//                 _ => format!("{base}.s"),
//             },
//             Action::CompileOnly => format!("{base}.o"),
//             Action::AssembleAndLink => base.to_string(),
//         }
//     };
//
//     let output = output.unwrap_or(default_output);
//
//     let verbose = verbose.unwrap_or(false);
//
//     let c = Config {
//         target,
//         input,
//         output,
//         action,
//         verbose,
//     };
//     c
// }
