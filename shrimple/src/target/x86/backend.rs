use std::collections::HashMap;

///
/// Lowers from STIR to x86 MIR
///
use crate::comment;
use crate::common::builder::*;
use crate::stir::builder::IRFunction;
use crate::target::stir::isa::*;
use crate::target::x86::builder::x86Function;
use crate::target::x86::isa::*;
use x86Instr::*;

#[derive(Default)]
pub struct Backend {
    pub(super) v2p: HashMap<IRValue, x86Value>,
    pub(super) v_rsp: i128,
    pub(super) builder: Option<x86Function>,
    pub(super) ir_args: Vec<IRType>,
}

impl Backend {
    /// Driver function to lower STIR code to x86 assembly
    pub fn lower(&mut self, stir_function: &mut IRFunction) -> x86Function {
        // Handle ABI impl
        self.resolve_args(stir_function);

        // This creates the builder
        self.translate(stir_function);

        // Legalizes instructions that may have been mangled by conforming to the ABI
        self.legalize();

        // Opt passes mutate the builder
        self.merge_degenerate_jumps();

        // Finally, take the builder out of self and return it
        self.builder.take().unwrap()
    }
}
