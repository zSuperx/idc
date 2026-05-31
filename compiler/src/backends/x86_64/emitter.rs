use std::collections::HashMap;

use crate::ast::BinOp;
use crate::backends::x86_64::instr::x86Instr;
use crate::backends::x86_64::{self, x86Function};
use crate::lir::{BB, BasicBlock, Builder, FnCtx, LirInstr, LirType, LirVal};
use crate::utils::align_n;

use x86_64::x86Reg::*;
use x86_64::x86Val::*;
use x86_64::x86Instr::*;

#[derive(Default)]
pub struct Emitter {
    v2h: HashMap<LirVal, x86_64::x86Val>,
    v_rsp: i128,
}

impl Emitter {
    pub fn new() -> Self {
        Self::default()
    }

    fn resolve_ptr(&self, v: LirVal, ty: LirType) -> x86_64::x86Val {
        match v {
            LirVal::Reg(id) => self.v2h.get(&v).copied().unwrap_or(Offset(ty, Virt(id), 0)),
            LirVal::Imm(_) => panic!("Resolve pointer called on immediate value"),
        }
    }

    fn resolve_val(&self, v: LirVal) -> x86_64::x86Val {
        match v {
            LirVal::Reg(id) => self.v2h.get(&v).copied().unwrap_or(Reg(Virt(id))),
            LirVal::Imm(i) => Imm(i),
        }
    }

    pub fn translate_func(&mut self, mut f: Builder<LirInstr>) -> Builder<x86Instr> {
        let mut builder = Builder::new(f.name, f.bb_count, f.vreg_count);

        let prologue = builder.next_bb("prologue");
        builder.start_new_block(prologue);
        builder.emit(Push(Reg(Rbp)));
        builder.emit(Mov(Reg(Rbp), Reg(Rsp)));

        let epilogue = builder.next_bb("epilogue");

        let mut bb_iter = f.bbs.iter().peekable();
        while let Some(bb) = bb_iter.next() {
            builder.start_new_block(bb.name);
            for i in bb.instructions.iter() {
                match *i {
                    LirInstr::Param(ty, dst, num, name) => {
                        let loc = match num {
                            0 => Reg(Rdi),
                            1 => Reg(Rsi),
                            2 => Reg(Rdx),
                            3 => Reg(Rcx),
                            4 => Reg(R8),
                            5 => Reg(R9),
                            _ => Offset(ty, Rbp, num.saturating_sub(6) as i128 + 8),
                        };
                        builder.emit(Comment(format!("{dst} -> {loc}")));
                        self.v2h.insert(dst, loc);
                    }
                    LirInstr::Alloc(ty, dst, name) => {
                        let size = align_n(ty.size() as i128, ty.alignment());
                        let loc = Offset(ty, Rbp, self.v_rsp - 8);
                        let aligned_size = align_n(size, 16);
                        self.v_rsp -= aligned_size;
                        builder.emit(Sub(Reg(Rsp), Imm(aligned_size as i128)));
                        builder.emit(Comment(format!("{dst} -> {loc}")));
                        self.v2h.insert(dst, loc);
                    }
                    LirInstr::Copy(ty, dst, rs1) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        builder.emit(Mov(dst, rs1))
                    }
                    // mov ..., [rs1]
                    LirInstr::Load(ty, dst, rs1) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_ptr(rs1, ty);

                        // I actually don't know if this will always be true
                        assert!(matches!(dst, Reg(_)));

                        let rs1 = match rs1 {
                            Reg(reg) => Offset(ty, reg, 0),
                            Offset(..) => rs1,
                            Imm(_) => unreachable!(),
                        };

                        builder.emit(Mov(dst, rs1));
                    }
                    // lea [rs1], rs2
                    LirInstr::Store(ty, rs1, rs2) => {
                        let rs1 = self.resolve_ptr(rs1, ty);
                        let rs2 = self.resolve_val(rs2);

                        // I actually don't know if this will always be true
                        assert!(matches!(rs1, Offset(..)));

                        match rs2 {
                            Offset(..) => {
                                let tmp = self.resolve_val(builder.next_reg());
                                builder.emit(Lea(tmp, rs2));
                                builder.emit(Mov(rs1, tmp));
                            }
                            _ => builder.emit(Mov(rs1, rs2)),
                        }
                    }
                    LirInstr::Br(rs1, bb1, bb2) => {
                        let rs1 = self.resolve_val(rs1);
                        builder.emit(Cmp(rs1, Imm(1)));
                        if bb_iter.peek().is_none_or(|next_bb| next_bb.name != bb1) {
                            builder.emit(Jnz(bb1));
                        }
                        builder.emit(Jz(bb2));
                    }
                    LirInstr::Jmp(bb) => builder.emit(Jmp(bb)),
                    LirInstr::Add(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Add(dst, rs2));
                    }
                    LirInstr::Sub(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Sub(dst, rs2));
                    }
                    LirInstr::Muls(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(IMul(dst, rs2));
                    }
                    LirInstr::Mulu(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        builder.emit(Mov(dst, rs1));
                        builder.emit(Mul(dst, rs2));
                    }
                    LirInstr::Eq(ty, dst, rs1, rs2)
                    | LirInstr::Sgt(ty, dst, rs1, rs2)
                    | LirInstr::Sge(ty, dst, rs1, rs2)
                    | LirInstr::Slt(ty, dst, rs1, rs2)
                    | LirInstr::Sle(ty, dst, rs1, rs2)
                    | LirInstr::Ugt(ty, dst, rs1, rs2)
                    | LirInstr::Uge(ty, dst, rs1, rs2)
                    | LirInstr::Ult(ty, dst, rs1, rs2)
                    | LirInstr::Ule(ty, dst, rs1, rs2) => {
                        let dst = self.resolve_val(dst);
                        let rs1 = self.resolve_val(rs1);
                        let rs2 = self.resolve_val(rs2);
                        builder.emit(Cmp(rs1, rs2));
                        let tmp = self.resolve_val(builder.next_reg());
                        builder.emit(Mov(tmp, Imm(1)));
                        let cmov_instr = match i {
                            LirInstr::Eq(..) => Cmove(dst, tmp),
                            LirInstr::Sgt(..) => Cmovg(dst, tmp),
                            LirInstr::Sge(..) => Cmovge(dst, tmp),
                            LirInstr::Slt(..) => Cmovl(dst, tmp),
                            LirInstr::Sle(..) => Cmovle(dst, tmp),
                            LirInstr::Ugt(..) => Cmovg(dst, tmp),
                            LirInstr::Uge(..) => Cmovge(dst, tmp),
                            LirInstr::Ult(..) => Cmovl(dst, tmp),
                            LirInstr::Ule(..) => Cmovle(dst, tmp),
                            _ => unreachable!(),
                        };
                        builder.emit(cmov_instr);
                    }

                    LirInstr::Ret(ty, rs1) => {
                        let rs1 = self.resolve_val(rs1);
                        builder.emit(Mov(Reg(Rax), rs1));
                        builder.emit(Jmp(epilogue));
                    }

                    LirInstr::RetVoid => {
                        builder.emit(Jmp(epilogue));
                    }
                }
            }
        }
        builder.start_new_block(epilogue);
        builder.emit(Mov(Reg(Rsp), Reg(Rbp)));
        builder.emit(Pop(Reg(Rbp)));
        builder.emit(Ret);

        let exit_bb = builder.next_bb("");
        builder.start_new_block(exit_bb);

        builder
    }

    pub fn allocate_registers(
        &mut self,
        instructions: Vec<x86Instr>,
    ) -> Vec<x86Instr> {
        todo!()
    }
}
