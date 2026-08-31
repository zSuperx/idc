#![allow(static_mut_refs)]
#![allow(nonstandard_style)]
#![allow(unused)]
#![warn(unused_imports)]
#![warn(unused_qualifications)]
#![warn(unused_allocation)]
use std::sync::LazyLock;

use clap::{Parser, ValueEnum};

use crate::{IRs::hir::HirObj, state::*};

mod ast;
mod codegen;
mod sema;
mod state;
// mod optimize;
mod IRs;
mod common;
mod parse;
mod validate;

pub static CFG: LazyLock<Config> = LazyLock::new(validate_config);

fn main() {
    let parsed_objects = get_state().parse_file(&CFG.input);

    resolve_top_level(&parsed_objects);

    let mut functions = vec![];
    for obj in parsed_objects {
        match obj.inner {
            HirObj::Fn(parsed_function) => {
                let mut function = Function::new(parsed_function);
                functions.push(function.codegen_func());
            }
            HirObj::Global { name, ty, rhs } => todo!("deal with global"),
            HirObj::Struct { name, fields } => {
            }
        }
    }

    let mut backend = match CFG.target {
        Target::x86 => stir::x86Backend::new(),
    };

    match CFG.action {
        Action::EmitIr => {
            for function in functions.iter() {
                function.print(CFG.verbose);
                println!();
            }
            println!()
        }
        Action::EmitAsm => {
            for function in functions.iter() {
                backend.lower(function).print(CFG.verbose);
                println!();
            }
        }
        Action::CompileOnly => todo!(),
        Action::AssembleAndLink => todo!(),
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
    no_regalloc: bool,
}

fn validate_config() -> Config {
    let args = Args::parse();

    let mut action_opt = None;

    if args.emit_ir && action_opt.replace(Action::EmitIr).is_some() {
        die!("Only 1 action can be performed. See -h");
    }

    if args.emit_asm && action_opt.replace(Action::EmitAsm).is_some() {
        die!("Only 1 action can be performed. See -h");
    }

    if args.compile_only && action_opt.replace(Action::CompileOnly).is_some() {
        die!("Only 1 action can be performed. See -h");
    }

    let action = action_opt.unwrap_or(Action::AssembleAndLink);

    let target = args.target.unwrap_or(Target::x86);
    let input = args.input;

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
    let no_regalloc = args.no_regalloc;
    if no_regalloc && action != Action::EmitAsm {
        die!("--no-regalloc can only be used with -S");
    }

    Config {
        target,
        input,
        output,
        action,
        verbose,
        no_regalloc,
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
    no_regalloc: bool,
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
