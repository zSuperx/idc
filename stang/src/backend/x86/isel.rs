use std::collections::HashMap;

use crate::{
    IRs::lir::Instr,
    backend::x86::*,
    comment,
    common::{IRBuilder, Value, ValueKind},
    die,
};

use x86Instr::*;

#[derive(Debug, Default)]
pub struct Backend {
    map: HashMap<Value, x86Val>,
    v_rsp: i128,
    // flags: Option<x86Flags>,
}

impl Backend {
    fn to_phys_reg(
        &mut self,
        builder: &mut IRBuilder<x86Instr>,
        lir_val: &Value,
        size: usize,
    ) -> x86Val {
        match lir_val.kind {
            ValueKind::Reg(r) => x86Val::reg(Reg::Virt(r), size),
            ValueKind::Imm(i) => x86Val::imm(i, size),
            ValueKind::Arg(a) => todo!(),
            ValueKind::Mem(..) => {
                let ptr = self.map.get(lir_val).cloned().unwrap();
                let dst = x86Val::reg(Reg::Virt(builder.next_reg()), 64);
                builder.emit(Lea(dst, ptr));
                dst
            }
        }
    }

    fn to_phys_ptr(
        &mut self,
        builder: &mut IRBuilder<x86Instr>,
        lir_val: &Value,
        size: usize,
    ) -> x86Val {
        match lir_val.kind {
            ValueKind::Mem(..) => self.map.get(lir_val).cloned().unwrap(),
            ValueKind::Reg(r) => x86Val::mem(Reg::Virt(r), 0, 64),
            _ => die!("Expected memory value, got {lir_val}"),
        }
    }

    pub fn translate(&mut self, lir_builder: IRBuilder<Instr>) -> IRBuilder<x86Instr> {
        let mut new_builder = IRBuilder::<x86Instr>::with_state(&lir_builder);
        new_builder.reg_count += 16;
        let builder = &mut new_builder;

        for (name, function) in lir_builder.get_all_functions() {
            // Emit prologue
            let prologue = builder.create_blockn(name, "prologue");
            builder.set_insert_point(prologue);
            builder.emit(Push(RBP));
            builder.emit(Mov(RBP, RSP));
            builder.add_successors(&[function.entrypoint]);
            let old_entrypoint = builder.set_entrypoint(name, prologue);
            builder.emit(Jmp(old_entrypoint));

            // Emit epilogue
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
                        Instr::Add(ty, dst, rs1, rs2) => todo!(),
                        Instr::Sub(ty, dst, rs1, rs2) => todo!(),
                        Instr::Umul(ty, dst, rs1, rs2) => todo!(),
                        Instr::Smul(ty, dst, rs1, rs2) => todo!(),
                        Instr::Udiv(ty, dst, rs1, rs2) => todo!(),
                        Instr::Sdiv(ty, dst, rs1, rs2) => todo!(),
                        Instr::Eq(ty, dst, rs1, rs2) => todo!(),
                        Instr::Ne(ty, dst, rs1, rs2) => todo!(),
                        Instr::Sgt(ty, dst, rs1, rs2) => todo!(),
                        Instr::Sge(ty, dst, rs1, rs2) => todo!(),
                        Instr::Slt(ty, dst, rs1, rs2) => todo!(),
                        Instr::Sle(ty, dst, rs1, rs2) => todo!(),
                        Instr::Ugt(ty, dst, rs1, rs2) => todo!(),
                        Instr::Uge(ty, dst, rs1, rs2) => todo!(),
                        Instr::Ult(ty, dst, rs1, rs2) => todo!(),
                        Instr::Ule(ty, dst, rs1, rs2) => todo!(),
                        Instr::Comment(s) => {
                            comment!(builder, "{s}");
                        }
                        Instr::Call(dst) => todo!(),
                        Instr::Copy(dst, rs1) => todo!(),
                        Instr::Load(ty, dst, src) => {
                            let dst = self.to_phys_reg(builder, &dst, ty.bits());
                            let ptr = self.to_phys_ptr(builder, &src, ty.bits());
                            builder.emit(Mov(dst, ptr));
                        }
                        Instr::Store(ty, dst, rs1) => {
                            let dst = self.to_phys_ptr(builder, &dst, ty.bits());
                            let rs1 = self.to_phys_reg(builder, &rs1, ty.bits());
                            builder.emit(Mov(dst, rs1));
                        }
                        Instr::Alloca(ty, value) => {
                            self.v_rsp -= ty.bytes() as i128;
                            builder.emit(Sub(RSP, x86Val::imm(ty.bytes() as i128, 64)));
                            let loc = x86Val::mem(Reg::BP, self.v_rsp, ty.bits());
                            comment!(builder, "{value} -> {loc}");
                            self.map.insert(*value, loc);
                        }
                        Instr::Sext(ty, dst, rs1) => todo!(),
                        Instr::Zext(ty, dst, rs1) => todo!(),
                        Instr::Trunc(ty, dst, rs1) => todo!(),
                        Instr::Retv => {
                            builder.emit(Jmp(epilogue));
                            builder.add_successors(&[epilogue]);
                        }
                        Instr::Ret(ty, value) => {
                            let dst = x86Val::reg(Reg::A, ty.bits());
                            let val = self.to_phys_reg(builder, &value, ty.bits());
                            builder.emit(Mov(dst, val));
                            builder.emit(Jmp(epilogue));
                            builder.add_successors(&[epilogue]);
                        }
                        Instr::Br(value, bbid, bbid1) => todo!(),
                        Instr::Jmp(bbid) => todo!(),
                    }
                }
            }
            assert!(builder.verify(name, None));
        }
        new_builder
    }
}
