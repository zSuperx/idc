use std::collections::{HashMap, HashSet};

use crate::{
    ast::{BinOp, UnOp},
    aux::Compiler,
    lir::{BasicBlock, Instr, LIRFunction, LirVal},
};

use Instr::*;

impl Compiler {
    pub fn optim_func(&mut self, mut f: LIRFunction) -> LIRFunction {
        let mut const_map = HashMap::new();
        let mut is_read = HashSet::new();
        for mut bb in f.bbs.iter_mut() {
            self.const_fold_bb(&mut bb, &mut const_map);
        }

        for mut bb in f.bbs.iter_mut() {
            self.track_live_code(&mut bb, &mut is_read);
        }

        for mut bb in f.bbs.iter_mut() {
            self.dead_code_elim(&mut bb, &mut is_read);
        }
        f
    }

    fn const_fold_bb(&mut self, bb: &mut BasicBlock, map: &mut HashMap<LirVal, i128>) {
        for i in 0..bb.instructions.len() {
            match bb.instructions[i] {
                Copy(ty, dst, rs1) => {
                    let v1 = map.get(&rs1).copied();
                    if let Some(imm) = v1 {
                        map.insert(dst, imm);
                    }
                }
                op @ (Add(ty, dst, rs1, rs2)
                | Sub(ty, dst, rs1, rs2)
                | Muls(ty, dst, rs1, rs2)
                | Mulu(ty, dst, rs1, rs2)
                | Sgt(ty, dst, rs1, rs2)
                | Sge(ty, dst, rs1, rs2)
                | Slt(ty, dst, rs1, rs2)
                | Sle(ty, dst, rs1, rs2)
                | Ugt(ty, dst, rs1, rs2)
                | Uge(ty, dst, rs1, rs2)
                | Ult(ty, dst, rs1, rs2)
                | Ule(ty, dst, rs1, rs2)
                | Eq(ty, dst, rs1, rs2)) => {
                    let v1 = map.get(&rs1).copied();
                    let v2 = map.get(&rs2).copied();

                    match (v1, v2) {
                        (Some(imm1), Some(imm2)) => {
                            let uimm1 = imm1 as u128;
                            let uimm2 = imm2 as u128;
                            let imm = match op {
                                Add(..) => imm1 + imm2,
                                Sub(..) => imm1 - imm2,
                                Muls(..) => imm1 * imm2,
                                Mulu(..) => (uimm1 * uimm2) as i128,
                                Eq(..) => (imm1 == imm2) as i128,
                                Sgt(..) => (imm1 > imm2) as i128,
                                Sge(..) => (imm1 >= imm2) as i128,
                                Slt(..) => (imm1 < imm2) as i128,
                                Sle(..) => (imm1 <= imm2) as i128,
                                Ugt(..) => (uimm1 > uimm2) as i128,
                                Uge(..) => (uimm1 >= uimm2) as i128,
                                Ult(..) => (uimm1 < uimm2) as i128,
                                Ule(..) => (uimm1 <= uimm2) as i128,
                                _ => unreachable!(),
                            };
                            map.insert(dst, imm);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    /// This pass should simply mark which registers are read from
    fn track_live_code(&mut self, bb: &BasicBlock, is_read: &mut HashSet<LirVal>) {
        for instr in bb.instructions.iter() {
            match *instr {
                Ret(_, rs1) | Br(rs1, ..) | Copy(_, _, rs1) | Load(_, _, rs1) => {
                    is_read.insert(rs1);
                }
                Add(_, _, rs1, rs2)
                | Sub(_, _, rs1, rs2)
                | Muls(_, _, rs1, rs2)
                | Mulu(_, _, rs1, rs2)
                | Eq(_, _, rs1, rs2)
                | Sgt(_, _, rs1, rs2)
                | Sge(_, _, rs1, rs2)
                | Slt(_, _, rs1, rs2)
                | Sle(_, _, rs1, rs2)
                | Ugt(_, _, rs1, rs2)
                | Uge(_, _, rs1, rs2)
                | Ult(_, _, rs1, rs2)
                | Ule(_, _, rs1, rs2)
                | Store(_, rs1, rs2) => {
                    is_read.insert(rs1);
                    is_read.insert(rs2);
                }
                _ => {}
            }
        }
    }

    // This pass should look at all instructions who PRODUCE a value. If that value is read, it is
    // considered a useful instructions
    fn dead_code_elim(&mut self, bb: &mut BasicBlock, is_read: &HashSet<LirVal>) {
        let mut survivors = vec![];
        for instr in bb.instructions.iter() {
            match *instr {
                Copy(_, dst, ..)
                | Add(_, dst, ..)
                | Sub(_, dst, ..)
                | Muls(_, dst, ..)
                | Mulu(_, dst, ..)
                | Eq(_, dst, ..)
                | Sgt(_, dst, ..)
                | Sge(_, dst, ..)
                | Slt(_, dst, ..)
                | Sle(_, dst, ..)
                | Ugt(_, dst, ..)
                | Uge(_, dst, ..)
                | Ult(_, dst, ..)
                | Ule(_, dst, ..)
                | Load(_, dst, ..) => {
                    if is_read.contains(&dst) {
                        survivors.push(*instr);
                    }
                }
                _ => survivors.push(*instr),
            }
        }
        _ = std::mem::replace(&mut bb.instructions, survivors);
    }
}
