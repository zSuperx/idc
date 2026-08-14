use std::collections::HashMap;

use crate::{
    IRs::lir::LirInstr,
    backend::x86::*,
    common::{IRBuilder, Value, ValueKind},
};

use x86Instr::*;

#[derive(Debug, Default)]
pub struct x86Backend {
    map: HashMap<usize, x86Val>,
    v_rsp: i128,
}

impl x86Backend {
    fn get(&mut self, lir_val: &Value, size: usize) -> x86Val {
        match lir_val.kind {
            ValueKind::Reg(r) => x86Val::reg(r + 16, size),
            ValueKind::Imm(i) => x86Val::imm(i, size),
            ValueKind::Arg(a) => x86Val::sysv_arg_n(a, size),
            ValueKind::Mem(_) => todo!(),
        }
    }

    pub fn translate(&mut self, lir_builder: IRBuilder<LirInstr>) -> IRBuilder<x86Instr> {
        let mut builder = IRBuilder::<x86Instr>::with_state(&lir_builder);

        for (name, function) in lir_builder.get_all_functions() {
            for (id, block) in function.blocks.iter() {
                builder.set_insert_point(*id);
                let terminator = block.terminator.clone().unwrap();
                for instr in block
                    .instructions
                    .iter()
                    .chain(std::iter::once(&terminator))
                {
                    match instr {
                        LirInstr::Add(ty, value, value1, value2) => todo!(),
                        LirInstr::Sub(ty, value, value1, value2) => todo!(),
                        LirInstr::Umul(ty, value, value1, value2) => todo!(),
                        LirInstr::Smul(ty, value, value1, value2) => todo!(),
                        LirInstr::Udiv(ty, value, value1, value2) => todo!(),
                        LirInstr::Sdiv(ty, value, value1, value2) => todo!(),
                        LirInstr::Eq(ty, value, value1, value2) => todo!(),
                        LirInstr::Ne(ty, value, value1, value2) => todo!(),
                        LirInstr::Sgt(ty, value, value1, value2) => todo!(),
                        LirInstr::Sge(ty, value, value1, value2) => todo!(),
                        LirInstr::Slt(ty, value, value1, value2) => todo!(),
                        LirInstr::Sle(ty, value, value1, value2) => todo!(),
                        LirInstr::Ugt(ty, value, value1, value2) => todo!(),
                        LirInstr::Uge(ty, value, value1, value2) => todo!(),
                        LirInstr::Ult(ty, value, value1, value2) => todo!(),
                        LirInstr::Ule(ty, value, value1, value2) => todo!(),
                        LirInstr::Comment(_) => todo!(),
                        LirInstr::Call(value) => todo!(),
                        LirInstr::Copy(value, value1) => {}
                        LirInstr::Load(ty, value, value1) => todo!(),
                        LirInstr::Store(ty, value, value1) => todo!(),
                        LirInstr::Alloca(ty, value) => todo!(),
                        LirInstr::Sext(ty, value, value1) => todo!(),
                        LirInstr::Zext(ty, value, value1) => todo!(),
                        LirInstr::Trunc(ty, value, value1) => todo!(),
                        LirInstr::Retv => {
                            builder.emit(Ret);
                        }
                        LirInstr::Ret(ty, value) => {
                            let dst = x86Val::reg(A, ty.bytes());
                            let val = self.get(value, ty.bytes());
                            builder.emit(Mov(dst, val));
                            builder.emit(Ret);
                        }
                        LirInstr::Br(value, bbid, bbid1) => todo!(),
                        LirInstr::Jmp(bbid) => todo!(),
                    }
                }
            }
        }
        builder
    }
}
