///
/// Lowers from STIR to x86 MIR
///
use crate::comment;
use crate::common::builder::*;
use crate::stir::builder::IRFunction;
use crate::target::stir::isa::*;
use crate::target::x86::Backend;
use crate::target::x86::builder::x86Function;
use crate::target::x86::isa::*;
use std::collections::HashMap;

// Bring all x86 instructions into current namespace for convenience
use x86Instr::*;

/// Bumps `val` up to the next largest multiple of `n`
fn align_up_n(val: i128, n: i128) -> i128 {
    let rem = val % n;
    if rem > 0 { val + (n - rem) } else { val }
}

/// Brings `val` down to the next smallest multiple of `n`
fn align_down_n(val: i128, n: i128) -> i128 {
    let rem = val % n;
    val + rem
}

impl Backend {
    pub fn new() -> Self {
        Self::default()
    }

    fn createFrameSlot(
        &mut self,
        builder: &mut x86Function,
        value: &IRValue,
        ty: &IRType,
    ) -> x86Value {
        // TODO: Align each alloca based on its alignment
        // it's so wrong right now but i need to sleep
        let ty = LLType::fromIRType(ty);
        self.v_rsp -= ty.bytes() as i128;
        let old_v_rsp = self.v_rsp;
        self.v_rsp = align_down_n(self.v_rsp, ty.bytes() as i128);
        comment!(builder, "Virtual RSP aligned from {old_v_rsp} -> {}", self.v_rsp);

        builder.emit(Sub(RSP, x86Value::Imm(ty.bytes() as i128)));
        let slot = x86Value::memDisp(Reg::BP, self.v_rsp, ty);
        self.v2p.insert(*value, slot);
        slot
    }

    fn lowerToReg(&self, builder: &mut x86Function, value: &IRValue, ty: LLType) -> x86Value {
        match value {
            IRValue::Arg(n) => {
                let arg = &builder.args[*n];
                // if the n'th argument is a struct, we can't turn it into a register
                panic!("Cannot lower aggregate value {value} to a single register");
                // else if its a register or pointer we chilling, just treat it as that
            }
            IRValue::Imm(i) => {
                let reg = x86Value::reg(Reg::Virt(builder.nextReg()), ty);
                builder.emit(Mov(reg, x86Value::Imm(*i)));
                reg
            }
            IRValue::Reg(r) => {
                if let Some(s) = self.v2p.get(value) {
                    if s.is_reg() {
                        *s
                    } else {
                        assert!(s.is_mem(), "VReg mapped to immediate");
                        let reg = x86Value::reg(Reg::Virt(builder.nextReg()), ty);
                        builder.emit(Mov(reg, *s));
                        reg
                    }
                } else {
                    x86Value::reg(Reg::Virt(*r), ty)
                }
            }
            IRValue::Ptr(r) => {
                if let Some(s) = self.v2p.get(value) {
                    assert!(s.is_mem());
                    // If this pointer is mapped to a physical address (i.e. [rbp - 8])
                    // we need to emit a lea instruction
                    let reg = x86Value::reg(Reg::Virt(builder.nextReg()), LLType::I64);
                    builder.emit(Lea(reg, *s));
                    return reg;
                } else {
                    // But if it's a new pointer, we don't have to bind it to a physical address
                    // We also don't need to emit a lea, since address of [%1] is just %1
                    x86Value::reg(Reg::Virt(*r), ty)
                }
            }
        }
    }

    fn lowerToPtr(&mut self, value: &IRValue, offset: i128, ty: LLType) -> x86Value {
        if let Some(s) = self.v2p.get(value) {
            return *s;
        }
        if let IRValue::Ptr(r) = value {
            x86Value::mem(Reg::Virt(*r), ty)
        } else {
            panic!("cant turn {value} into pointer")
        }
    }

    pub(crate) fn translate(&mut self, stir_function: &IRFunction) {
        // Create the function
        let rty = LLType::fromIRType(stir_function.getReturnType());
        let mut new_function = x86Function::new(stir_function.name(), rty);
        for argty in stir_function.args.iter() {
            // self.ir_args.push(*argty);
        }

        let builder = &mut new_function;
        builder.setRegCount(stir_function.getRegCount());

        // Do a first pass to register all blocks in a map
        let mut block_map = HashMap::new();
        stir_function.dfs(|id, block| {
            let curr = builder.newNamedBlock(block.name);
            block_map.insert(id, curr);
            false
        });

        // Make the prologue the actual entrypoint
        let stir_ep = stir_function.getEntryPoint();
        let body = block_map[&stir_ep];

        // Create prologue
        let prologue = builder.newNamedBlock("prologue");
        builder.setEntryPoint(prologue);
        builder.setInsertPoint(prologue);
        builder.emit(Push(RBP));
        builder.emit(Mov(RBP, RSP));
        builder.emit(Jmp(body));
        builder.addSuccessors(&[body]);

        // Create epilogue
        let epilogue = builder.newNamedBlock("epilogue");
        builder.setInsertPoint(epilogue);
        builder.emit(Mov(RSP, RBP));
        builder.emit(Pop(RBP));
        builder.emit(Ret);

        // Perform a visitor pass through the function and translate each block one at a time
        stir_function.dfs(|id, block| {
            let curr = block_map[&id];
            builder.setInsertPoint(curr);

            for instr in block.instructions.iter().chain(&block.terminator) {
                match instr {
                    IRInstr::Comment(s) => builder.emit(Comment(s.clone())),
                    IRInstr::Jmp(b) => {
                        let b = block_map[b];
                        builder.addSuccessors(&[b]);
                        builder.emit(Jmp(b));
                    }
                    IRInstr::Store(ty, ptr, rs1) => {
                        let llty = LLType::fromIRType(ty);
                        let ptr = self.lowerToPtr(ptr, 0, llty);
                        let rs1 = self.lowerToReg(builder, rs1, llty);
                        builder.emit(Mov(ptr, rs1));
                    }
                    IRInstr::Load(ty, ptr, dst) => {
                        let llty = LLType::fromIRType(ty);
                        let ptr = self.lowerToPtr(ptr, 0, llty);
                        let dst = self.lowerToReg(builder, dst, llty);
                        builder.emit(Mov(dst, ptr));
                    }
                    IRInstr::Copy(ty, dst, rs1) => {
                        let llty = LLType::fromIRType(ty);
                        let dst = self.lowerToReg(builder, dst, llty);
                        let rs1 = self.lowerToReg(builder, rs1, llty);
                        builder.emit(Mov(dst, rs1));
                    }
                    IRInstr::Sdiv(ty, dst, rs1, rs2)
                    | IRInstr::Smul(ty, dst, rs1, rs2)
                    | IRInstr::Sub(ty, dst, rs1, rs2)
                    | IRInstr::Udiv(ty, dst, rs1, rs2)
                    | IRInstr::Umul(ty, dst, rs1, rs2)
                    | IRInstr::Add(ty, dst, rs1, rs2) => {
                        let llty = LLType::fromIRType(ty);
                        let dst = self.lowerToReg(builder, dst, llty);
                        let rs1 = self.lowerToReg(builder, rs1, llty);
                        let rs2 = self.lowerToReg(builder, rs2, llty);
                        builder.emit(Mov(dst, rs1));
                        let op = match instr {
                            IRInstr::Sdiv(..) => Idiv,
                            IRInstr::Smul(..) => Imul,
                            IRInstr::Sub(..) => Sub,
                            IRInstr::Udiv(..) => Idiv,
                            IRInstr::Umul(..) => Imul,
                            IRInstr::Add(..) => Add,
                            _ => unreachable!(),
                        };
                        builder.emit(op(dst, rs2));
                    }
                    IRInstr::Icmp(cmp, ty, dst, rs1, rs2) => {
                        let llty = LLType::fromIRType(ty);
                        let rs1 = self.lowerToReg(builder, rs1, llty);
                        let rs2 = self.lowerToReg(builder, rs2, llty);
                        builder.emit(Cmp(rs1, rs2));
                        let flag = match cmp {
                            CmpOp::Slt | CmpOp::Ult => RFLAG::LT,
                            CmpOp::Ule | CmpOp::Sle => RFLAG::LE,
                            CmpOp::Ugt | CmpOp::Sgt => RFLAG::GT,
                            CmpOp::Uge | CmpOp::Sge => RFLAG::GE,
                            CmpOp::Eq => RFLAG::EQ,
                            CmpOp::Ne => RFLAG::NE,
                        };
                        self.v2p.insert(*dst, x86Value::CC(flag));
                    }
                    IRInstr::Br(cond, then_bb, else_bb) => {
                        let x86then = block_map[then_bb];
                        let x86else = block_map[else_bb];
                        builder.addSuccessors(&[x86then]);
                        builder.addFallthrough(x86else);
                        if let Some(phy) = self.v2p.get(cond) {
                            match phy {
                                x86Value::CC(rflag) => {
                                    let jcc = match rflag {
                                        RFLAG::LT => Jl,
                                        RFLAG::LE => Jle,
                                        RFLAG::GT => Jg,
                                        RFLAG::GE => Jge,
                                        RFLAG::EQ => Je,
                                        RFLAG::NE => Jne,
                                        RFLAG::Z => Jz,
                                        RFLAG::NZ => Jnz,
                                        RFLAG::O => Jo,
                                        RFLAG::NO => Jno,
                                    };
                                    builder.emit(jcc(x86then));
                                }
                                _ => panic!("What"),
                            }
                        } else {
                            let cond = self.lowerToReg(builder, cond, LLType::I8);
                            builder.emit(Cmp(cond, x86Value::Imm(0)));
                            builder.emit(Jnz(x86then));
                        }
                    }
                    // TODO: Improve the getaddr IR instruction to take multi-dimensional offsets
                    // Then just input those to memFull as scale, index, and disp
                    IRInstr::Getaddr(dst, base, ty, index) => {
                        let llty = LLType::fromIRType(ty);
                        let dst = self.lowerToReg(builder, dst, LLType::I64);
                        let base = self.lowerToReg(builder, base, LLType::I64);
                        let addr = match index {
                            IRValue::Reg(_) | IRValue::Ptr(_) => {
                                let index_val = self.lowerToReg(builder, index, LLType::I64);
                                x86Value::memFull(
                                    base.getReg(),
                                    Some(index_val.getReg()),
                                    llty.bytes(),
                                    0,
                                    llty,
                                )
                            }
                            IRValue::Arg(n) => todo!(),
                            IRValue::Imm(i) => {
                                x86Value::memFull(base.getReg(), None, llty.bytes(), *i, llty)
                            }
                        };
                        builder.emit(Lea(dst, addr));
                    }
                    IRInstr::Alloca(ty, dst) => {
                        let dst = self.createFrameSlot(builder, dst, ty);
                    }
                    IRInstr::Retv => {
                        builder.addSuccessors(&[epilogue]);
                        builder.emit(Jmp(epilogue));
                    }
                    IRInstr::Ret(ty, rs1) => {
                        let llty = LLType::fromIRType(ty);
                        let rs1 = self.lowerToReg(builder, rs1, llty);

                        let rty_bits = builder.getReturnType().bits();
                        let a = if rty_bits == 64 { RAX } else { EAX };

                        if ty.bits() >= a.ty().bits() {
                            builder.emit(Mov(a, rs1));
                        } else {
                            builder.emit(Movzx(a, rs1));
                        }

                        builder.addSuccessors(&[epilogue]);
                        builder.emit(Jmp(epilogue));
                    }
                    IRInstr::Trunc(to_ty, dst, from_ty, rs1) => {
                        let to_llty = LLType::fromIRType(to_ty);
                        let dst = self.lowerToReg(builder, dst, to_llty);
                        let rs1 = self.lowerToReg(builder, rs1, to_llty);
                        builder.emit(Mov(dst, rs1));
                    }
                    IRInstr::Zext(to_ty, dst, from_ty, rs1)
                    | IRInstr::Sext(to_ty, dst, from_ty, rs1) => {
                        let to_llty = LLType::fromIRType(to_ty);
                        let dst = self.lowerToReg(builder, dst, to_llty);
                        let from_llty = LLType::fromIRType(from_ty);
                        let rs1 = self.lowerToReg(builder, rs1, from_llty);
                        match instr {
                            IRInstr::Zext(..) => builder.emit(Movzx(dst, rs1)),
                            IRInstr::Sext(..) => builder.emit(Movsx(dst, rs1)),
                            _ => unreachable!(),
                        }
                    }
                }
            }
            false
        });

        self.builder = Some(new_function);
    }
}
