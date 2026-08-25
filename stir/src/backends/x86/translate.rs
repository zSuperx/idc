use std::collections::HashMap;
use std::collections::HashSet;

use super::MIR::*;
use crate::builder::*;
use crate::comment;
use crate::isa::*;
use x86Instr::*;

#[derive(Default)]
pub struct Backend {
    v_rsp: i128,
    ptr_map: HashMap<IRValue, x86Val>,
    builder: IRBuilder<x86Instr>,
}

impl Backend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn print_all_functions(&self) {
        self.builder.print_all_functions();
    }

    pub fn getAsAddr(&mut self, val: &IRValue, size: usize) -> x86Val {
        match val.kind {
            ValueKind::Reg(r) => panic!("Can't convert an IR register to an x86 address: {val}"),
            ValueKind::Imm(_) => {
                panic!("Cannot convert an IR immediate to a x86 address value: {val}")
            }
            ValueKind::Ptr(r) => match self.ptr_map.get(val) {
                Some(v) => {
                    assert!(matches!(v, x86Val::Address { .. }));
                    *v
                }
                None => {
                    let new = x86Val::addr(Reg::Virt(r), 0, size);
                    self.ptr_map.insert(*val, new);
                    new
                }
            },
        }
    }

    pub fn getAsValue(&mut self, val: &IRValue, size: usize) -> x86Val {
        match val.kind {
            // IR immediates can be trivially converted to their Phy version
            ValueKind::Imm(i) => x86Val::Imm(i),
            // Since builder state is preserved, IR register -> Phy register is also trivial
            ValueKind::Reg(r) => x86Val::reg(Reg::Virt(r), size),
            // Pointers are more complicated...
            //
            // We first check the pointer map to see if this IR pointer has already been bound to a
            // Phy pointer
            ValueKind::Ptr(r) => match self.ptr_map.get(val) {
                Some(v) => {
                    // If so, we check if its a trivial translation (lea x, [y] == mov x, y)
                    if let x86Val::Address {
                        base, offset: 0, ..
                    } = v
                    {
                        x86Val::reg(*base, 64)
                    } else {
                        // In the case that its non-trivial, emit an intermediate `lea`
                        let lea_dst = x86Val::reg(Reg::Virt(self.builder.next_reg()), 64);
                        self.builder.emit(Lea(lea_dst, *v));
                        lea_dst
                    }
                }
                // If it is a new pointer, add it as an address type but return a register
                None => {
                    let base = Reg::Virt(r);
                    let phy = x86Val::addr(base, 0, size);
                    self.ptr_map.insert(*val, phy);
                    // Since it's a pointer type, it must be 64 bits wide
                    x86Val::reg(base, 64)
                }
            },
        }
    }

    pub fn translate(&mut self, stir_builder: &IRBuilder<IRInstr>) {
        self.builder = IRBuilder::with_state(stir_builder);

        for (name, stir_function) in stir_builder.get_all_functions() {
            // Emit prologue
            let prologue = self.builder.newNamedBlock(name, "prologue");
            self.builder.set_insert_point(prologue);
            self.builder.emit(Push(RBP));
            self.builder.emit(Mov(RBP, RSP));
            self.builder.add_successors(&[stir_function.entrypoint]);
            let old_entrypoint = self.builder.set_entrypoint(name, prologue);
            self.builder.emit(Jmp(old_entrypoint));

            // Emit epilogue
            let epilogue = self.builder.newNamedBlock(name, "epilogue");
            self.builder.set_insert_point(epilogue);
            self.builder.emit(Mov(RSP, RBP));
            self.builder.emit(Pop(RBP));
            self.builder.emit(Ret);

            stir_function.dfs(|id, block| {
                // TODO: don't simply iterate through the function's blocks,
                // there may be invalid ones (i.e. empty/no terminator)
                // Do a DFS traversal!
                self.builder.set_insert_point(id);
                let terminator = block.terminator.clone().unwrap();
                for instr in block
                    .instructions
                    .iter()
                    .chain(std::iter::once(&terminator))
                {
                    match instr {
                        IRInstr::Umul(ty, dst, rs1, rs2) => {
                            let dst = self.getAsValue(dst, ty.bits());
                            let rs1 = self.getAsValue(rs1, ty.bits());
                            let rs2 = self.getAsValue(rs2, ty.bits());
                            self.builder.emit(Mov(dst, rs1));
                            self.builder.emit(Imul(dst, rs2));
                        }
                        IRInstr::Add(ty, dst, rs1, rs2) => {
                            let dst = self.getAsValue(dst, ty.bits());
                            let rs1 = self.getAsValue(rs1, ty.bits());
                            let rs2 = self.getAsValue(rs2, ty.bits());
                            self.builder.emit(Mov(dst, rs1));
                            self.builder.emit(Add(dst, rs2));
                        }
                        IRInstr::Comment(s) => self.builder.emit(Comment(s.clone())),
                        IRInstr::Alloca(ty, value) => {
                            self.v_rsp -= ty.bytes() as i128;
                            self.builder.emit(Sub(RSP, x86Val::Imm(ty.bytes() as i128)));
                            let loc = x86Val::addr(Reg::BP, self.v_rsp, ty.bits());
                            comment!(self.builder, "{value} -> {loc}");
                            self.ptr_map.insert(*value, loc);
                        }
                        IRInstr::Icmp(cmp, ty, dst, rs1, rs2) => {
                            if *ty == IRType::I1 {
                                // TODO: An i1 type can map really well to x86_64 RFLAGS register.
                                // A potential approach here could be to emit a cmp rs1, rs2,
                                // then map dst -> CC::LT.
                                //
                                // If we encounter a Br that depends on dst, we can try looking up dst
                                // in the CC flag map. If it shows up, we can emit
                                // CC::LT => emit Jl true_bb
                                // CC::LE => emit Jle true_bb
                                // CC::EQ => emit Je true_bb
                                // ...
                                //
                                // If dst doesn't show up in the CC map, check the register map. If it's
                                // there, we've bound that value to a space in memory, meaning we can
                                // emit a test followed by jnz
                                todo!()
                            } else {
                                todo!()
                            }
                        }
                        IRInstr::Br(dst, true_bb, false_bb) => todo!(),
                        IRInstr::Load(ty, ptr, dst) => {
                            let ptr = self.getAsAddr(ptr, ty.bits());
                            let dst = self.getAsValue(dst, ty.bits());
                            self.builder.emit(Mov(dst, ptr));
                        }
                        IRInstr::Store(ty, ptr, rs1) => {
                            let ptr = self.getAsAddr(ptr, ty.bits());
                            let rs1 = self.getAsValue(rs1, ty.bits());
                            self.builder.emit(Mov(ptr, rs1));
                        }
                        IRInstr::Copy(ty, dst, rs1) => {
                            let dst = self.getAsValue(dst, ty.bits());
                            let rs1 = self.getAsValue(rs1, ty.bits());
                            self.builder.emit(Mov(dst, rs1));
                        }
                        IRInstr::Getaddr(dst, base, index_ty, index) => {
                            let phy_dst = x86Val::addr(Reg::Virt(self.builder.next_reg()), 0, index_ty.bits());
                            self.ptr_map.insert(*dst, phy_dst);
                            let phy_base = self.getAsValue(base, IRType::Ptr.bits());
                            let phy_index = self.getAsValue(index, IRType::I64.bits());
                            if let x86Val::Imm(i) = phy_index {
                                let x86Val::Reg { reg, size: 64 } = phy_base else {
                                    panic!("idk yet")
                                };
                                comment!(self.builder, "Size of {index_ty} = {}", index_ty.bits());
                                let mem = x86Val::addr(reg, i, index_ty.bits());
                                self.builder.emit(Lea(phy_dst, mem));
                            } else {
                                todo!()
                            }
                        }
                        IRInstr::Jmp(target) => self.builder.emit(Jmp(*target)),
                        IRInstr::Ret(ty, rs1) => {
                            let rs1 = self.getAsValue(rs1, ty.bits());
                            let a = x86Val::reg(Reg::A, ty.bits());
                            self.builder.emit(Mov(a, rs1));
                            self.builder.emit(Jmp(epilogue));
                            self.builder.add_successors(&[epilogue]);
                        }
                        IRInstr::Retv => {
                            self.builder.emit(Jmp(epilogue));
                            self.builder.add_successors(&[epilogue]);
                        }
                        x => todo!("Translate {x}"),
                    }
                }
                false
            });
            let mut f = self.builder.get_current_function();
        }
    }
}
