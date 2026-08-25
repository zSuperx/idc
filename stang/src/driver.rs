use crate::{
    IRs::{hir::HirObj},
    state::*,
};

use stir::isa::*;
use stir::builder::IRBuilder;
use stir::backends::x86::Backend;

pub fn compile_program(filename: &'static str) {
    let parsed_objects = get_state().parse_file(filename);

    resolve_top_level(&parsed_objects);

    let mut builder = IRBuilder::<IRInstr>::default();
    for obj in parsed_objects {
        match obj.inner {
            HirObj::Fn(parsed_function) => {
                let mut function = Function::new(parsed_function);
                function.codegen_func(&mut builder);
            }
            HirObj::Global { name, ty, rhs } => todo!(),
            HirObj::Struct { name, fields } => todo!(),
        }
    }

    builder.print_all_functions();

    {
        let mut backend = Backend::new();
        backend.translate(&builder);
        backend.print_all_functions();
    }
}
