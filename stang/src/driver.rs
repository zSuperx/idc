use crate::{CFG, IRs::hir::HirObj, state::*};

use stir::x86Backend;

pub fn compile_program(filename: &'static str) {
    let parsed_objects = get_state().parse_file(filename);

    resolve_top_level(&parsed_objects);

    let mut functions = vec![];
    for obj in parsed_objects {
        match obj.inner {
            HirObj::Fn(parsed_function) => {
                let mut function = Function::new(parsed_function);
                functions.push(function.codegen_func());
            }
            HirObj::Global { name, ty, rhs } => todo!(),
            HirObj::Struct { name, fields } => todo!(),
        }
    }

    let mut backend = x86Backend::new();

    for function in functions {
        function.print(CFG.verbose);
        println!();
        backend.lower(&function).print(CFG.verbose);
    }
}
