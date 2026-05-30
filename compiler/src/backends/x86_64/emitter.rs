use std::collections::HashMap;

use crate::ast::BinOp;
use crate::backends::x86_64::{self, x86_64Function};
use crate::lir::{BB, Instr, LIRFunction, VReg};
use crate::utils::align_n;

use x86_64::Instr::*;
use x86_64::Reg::*;
use x86_64::Val::*;

#[derive(Default)]
pub struct Emitter {
    v2h: HashMap<VReg, x86_64::Val>,
    v_rsp: i128,
}

impl Emitter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn translate_func(&mut self, mut f: LIRFunction) -> x86_64Function {
        let mut buf = vec![];
        self.emit_prologue(&mut buf);
        let mut bbs = f.bbs.iter().peekable();
        while let Some(bb) = bbs.next() {
            buf.push(Label(bb.name));
            for i in bb.instructions.iter() {
                buf.push(Comment(format!("IR: {i}")));
                match *i {
                    Instr::Param(ty, dst, num, name) => {
                        let loc = match num {
                            0 => Reg(Rdi),
                            1 => Reg(Rsi),
                            2 => Reg(Rdx),
                            3 => Reg(Rcx),
                            4 => Reg(R8),
                            5 => Reg(R9),
                            _ => Offset(Rbp, num.saturating_sub(6) as i128 + 8),
                        };
                        buf.push(Comment(format!("{dst} -> {loc}")));
                        self.v2h.insert(dst, loc);
                    }
                    Instr::Alloc(ty, dst, name) => {
                        let size = align_n(ty.size() as i128, ty.alignment());
                        let loc = Offset(Rbp, self.v_rsp - 8);
                        let aligned_size = align_n(size, 16);
                        self.v_rsp -= aligned_size;
                        buf.push(Sub(Reg(Rsp), Imm(aligned_size as i128)));
                        buf.push(Comment(format!("{dst} -> {loc}")));
                        self.v2h.insert(dst, loc);
                    }
                    Instr::Const(ty, dst, imm) => buf.push(Mov(Reg(Virt(dst)), Imm(imm))),
                    Instr::Copy(ty, dst, rs1) => {
                        let dst = self.v2h.get(&dst).copied().unwrap_or(Reg(Virt(dst)));
                        let rs1 = self.v2h.get(&rs1).copied().unwrap_or(Reg(Virt(rs1)));
                        buf.push(Mov(dst, rs1))
                    }
                    // mov ..., [rs1]
                    Instr::Load(ty, dst, rs1) => {
                        let dst = self.v2h.get(&dst).copied().unwrap_or(Reg(Virt(dst)));
                        let rs1 = self.v2h.get(&rs1).copied().unwrap_or(Offset(Virt(rs1), 0));

                        // I actually don't know if this will always be true
                        assert!(matches!(dst, Reg(_)));

                        let rs1 = match rs1 {
                            Reg(reg) => Offset(reg, 0),
                            Offset(reg, _) => rs1,
                            Imm(_) => unreachable!(),
                        };

                        buf.push(Mov(dst, rs1));
                    }
                    // lea [rs1], rs2
                    Instr::Store(ty, rs1, rs2) => {
                        let rs1 = self.v2h.get(&rs1).copied().unwrap_or(Offset(Virt(rs1), 0));
                        let rs2 = self.v2h.get(&rs2).copied().unwrap_or(Reg(Virt(rs2)));

                        // I actually don't know if this will always be true
                        assert!(matches!(rs1, Offset(..)));

                        match rs2 {
                            Offset(..) => {
                                f.vreg_count += 1;
                                let tmp = Virt(VReg(f.vreg_count));
                                buf.push(Lea(Reg(tmp), rs2));
                                buf.push(Mov(rs1, Reg(tmp)));
                            }
                            _ => buf.push(Mov(rs1, rs2)),
                        }
                    }
                    Instr::Br(rs1, bb1, bb2) => {
                        let rs1 = self.v2h.get(&rs1).copied().unwrap_or(Reg(Virt(rs1)));
                        buf.push(Cmp(rs1, Imm(1)));
                        if bbs.peek().is_none_or(|next_bb| next_bb.name != bb1) {
                            buf.push(Jnz(bb1));
                        }
                        buf.push(Jz(bb2));
                    }
                    Instr::Jmp(bb) => buf.push(Jmp(bb)),
                    Instr::Add(ty, dst, rs1, rs2) => {
                        buf.push(Mov(Reg(Virt(dst)), Reg(Virt(rs1))));
                        buf.push(Add(Reg(Virt(dst)), Reg(Virt(rs2))));
                    }
                    Instr::Sub(ty, dst, rs1, rs2) => {
                        buf.push(Mov(Reg(Virt(dst)), Reg(Virt(rs1))));
                        buf.push(Sub(Reg(Virt(dst)), Reg(Virt(rs2))));
                    }
                    Instr::Muls(ty, dst, rs1, rs2) => {
                        buf.push(Mov(Reg(Virt(dst)), Reg(Virt(rs1))));
                        buf.push(IMul(Reg(Virt(dst)), Reg(Virt(rs2))));
                    }
                    Instr::Mulu(ty, dst, rs1, rs2) => {
                        buf.push(Mov(Reg(Virt(dst)), Reg(Virt(rs1))));
                        buf.push(Mul(Reg(Virt(dst)), Reg(Virt(rs2))));
                    }
                    Instr::Eq(ty, dst, rs1, rs2)
                    | Instr::Sgt(ty, dst, rs1, rs2)
                    | Instr::Sge(ty, dst, rs1, rs2)
                    | Instr::Slt(ty, dst, rs1, rs2)
                    | Instr::Sle(ty, dst, rs1, rs2)
                    | Instr::Ugt(ty, dst, rs1, rs2)
                    | Instr::Uge(ty, dst, rs1, rs2)
                    | Instr::Ult(ty, dst, rs1, rs2)
                    | Instr::Ule(ty, dst, rs1, rs2) => {
                        buf.push(Cmp(Reg(Virt(rs1)), Reg(Virt(rs2))));
                        f.vreg_count += 1;
                        let tmp = Reg(Virt(VReg(f.vreg_count)));
                        buf.push(Mov(tmp, Imm(1)));
                        let cmov_instr = match i {
                            Instr::Eq(..) => Cmove(Reg(Virt(dst)), tmp),
                            Instr::Sgt(..) => Cmovg(Reg(Virt(dst)), tmp),
                            Instr::Sge(..) => Cmovge(Reg(Virt(dst)), tmp),
                            Instr::Slt(..) => Cmovl(Reg(Virt(dst)), tmp),
                            Instr::Sle(..) => Cmovle(Reg(Virt(dst)), tmp),
                            Instr::Ugt(..) => Cmovg(Reg(Virt(dst)), tmp),
                            Instr::Uge(..) => Cmovge(Reg(Virt(dst)), tmp),
                            Instr::Ult(..) => Cmovl(Reg(Virt(dst)), tmp),
                            Instr::Ule(..) => Cmovle(Reg(Virt(dst)), tmp),
                            _ => unreachable!(),
                        };
                        buf.push(cmov_instr);
                    }

                    Instr::Ret(ty, rs1) => {
                        let rs1 = self.v2h.get(&rs1).copied().unwrap_or(Reg(Virt(rs1)));
                        buf.push(Mov(Reg(Rax), rs1));
                        self.emit_epilogue(&mut buf);
                    }

                    Instr::RetVoid => {
                        self.emit_epilogue(&mut buf);
                    }
                }
            }
        }
        x86_64Function::new(buf)
    }

    fn emit_prologue(&mut self, buf: &mut Vec<x86_64::Instr>) {
        buf.push(Push(Reg(Rbp)));
        buf.push(Mov(Reg(Rbp), Reg(Rsp)));
    }

    fn emit_epilogue(&mut self, buf: &mut Vec<x86_64::Instr>) {
        buf.push(Mov(Reg(Rsp), Reg(Rbp)));
        buf.push(Pop(Reg(Rbp)));
        buf.push(Ret);
    }

    pub fn allocate_registers(&mut self, instructions: Vec<x86_64::Instr>) -> Vec<x86_64::Instr> {
        todo!()
    }
}
