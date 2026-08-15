use std::collections::HashMap;

use crate::{
    IRs::lir::LirInstr,
    backend::x86::*,
    common::{IRBuilder, Value, ValueKind},
};

use x86Instr::*;

#[derive(Debug, Default)]
pub struct x86Backend {
    map: HashMap<Value, x86Val>,
    v_rsp: i128,
    // flags: Option<x86Flags>,
}

impl x86Backend {
    fn get(&mut self, lir_val: &Value, size: usize) -> x86Val {
        match lir_val.kind {
            ValueKind::Reg(r) => x86Val::reg(r + 16, size),
            ValueKind::Imm(i) => x86Val::imm(i, size),
            ValueKind::Arg(a) => x86Val::sysv_arg_n(a, size),
            ValueKind::Mem(m) => self.map.get(lir_val).cloned().unwrap(),
        }
    }

    pub fn translate(&mut self, lir_builder: IRBuilder<LirInstr>) -> IRBuilder<x86Instr> {
        let mut builder = IRBuilder::<x86Instr>::with_state(&lir_builder);

        for (name, function) in lir_builder.get_all_functions() {
            let prologue = builder.create_blockn(name, "prologue");
            builder.set_insert_point(prologue);
            builder.emit(Push(RBP));
            builder.emit(Mov(RBP, RSP));
            builder.add_successors(&[function.entrypoint]);
            let old_entrypoint = builder.set_entrypoint(name, prologue);
            builder.emit(Jmp(old_entrypoint));

            let epilogue = builder.create_blockn(name, "epilogue");
            builder.set_insert_point(epilogue);
            builder.emit(Mov(RSP, RBP));
            builder.emit(Pop(RBP));
            builder.emit(Ret);

            for (id, block) in function.blocks.iter() {
                builder.set_insert_point(*id);
                let terminator = block.terminator.clone().unwrap();
                for instr in block
                    .instructions
                    .iter()
                    .chain(std::iter::once(&terminator))
                {
                    match instr {
                        LirInstr::Add(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Sub(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Umul(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Smul(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Udiv(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Sdiv(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Eq(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Ne(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Sgt(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Sge(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Slt(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Sle(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Ugt(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Uge(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Ult(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Ule(ty, dst, rs1, rs2) => todo!(),
                        LirInstr::Comment(_) => todo!(),
                        LirInstr::Call(dst) => todo!(),
                        LirInstr::Copy(dst, rs1) => todo!(),
                        LirInstr::Load(ty, dst, value1) => {
                            let dst = self.get(dst, ty.bits());
                            let ptr = self.get(value1, ty.bits());
                            builder.emit(Mov(dst, ptr));
                        }
                        LirInstr::Store(ty, value, value1) => {
                            let ptr = self.get(value, ty.bits());
                            let val = self.get(value1, ty.bits());
                            builder.emit(Mov(ptr, val));
                        }
                        LirInstr::Alloca(ty, value) => {
                            self.v_rsp -= ty.bytes() as i128;
                            builder.emit(Sub(RSP, x86Val::imm(ty.bytes() as i128, 64)));
                            let loc = x86Val::mem(RegName::BP as usize, self.v_rsp, ty.bits());
                            self.map.insert(*value, loc);
                        }
                        LirInstr::Sext(ty, dst, rs1) => todo!(),
                        LirInstr::Zext(ty, dst, rs1) => todo!(),
                        LirInstr::Trunc(ty, dst, rs1) => todo!(),
                        LirInstr::Retv => {
                            builder.emit(Jmp(epilogue));
                            builder.add_successors(&[epilogue]);
                        }
                        LirInstr::Ret(ty, value) => {
                            let dst = x86Val::reg(RegName::A as usize, ty.bits());
                            let val = self.get(value, ty.bits());
                            builder.emit(Mov(dst, val));
                            builder.emit(Jmp(epilogue));
                            builder.add_successors(&[epilogue]);
                        }
                        LirInstr::Br(value, bbid, bbid1) => todo!(),
                        LirInstr::Jmp(bbid) => todo!(),
                    }
                }
            }
            assert!(builder.verify(name, None));
        }
        builder
    }
}
