use crate::stir::builder::IRFunction;
use crate::stir::isa::*;
use crate::target::x86::Backend;
use crate::target::x86::isa::*;

impl Backend {
    /// Scans the arguments of `stir_function` and maps them to physical registers by populating the
    /// `self.v2p` map. The mapping follows the x86_64 System V ABI specification.
    pub(crate) fn resolve_args(&mut self, stir_function: &mut IRFunction) {
        let mut registers = [Reg::DI, Reg::SI, Reg::D, Reg::C, Reg::R8, Reg::R9].iter();
        let mut used_stack_bytes = 0;
        let mut curr_reg = registers.next();
        // let mut new_args = vec![];
        for (arg_val, arg_ty) in stir_function.args.iter() {
            if let IRType::Struct(struct_id) = arg_ty {
                todo!("System V ABI: Handle structs")
            } else {
                let llty = LLType::fromIRType(arg_ty);
                let dst = match curr_reg {
                    Some(r) => x86Value::reg(*r, llty),
                    None => x86Value::memDisp(Reg::BP, used_stack_bytes + 8, llty),
                };
                self.v2p.insert(*arg_val, dst);
                curr_reg = registers.next();
            }
        }

        // stir_function.args = new_args;
    }
}
