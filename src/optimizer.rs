use std::collections::{HashMap, HashSet};

use crate::{
    ast::{BinOp, UnOp},
    compiler::Compiler,
    lir::{BasicBlock, Instr},
};

use Instr::*;

impl Compiler {
    pub fn const_fold_bb(&mut self, bb: &mut BasicBlock) {
        let mut map = HashMap::new();
        let mut is_read = HashSet::new();
        for i in 0..bb.instructions.len() {
            match bb.instructions[i] {
                Const { dst, imm } => {
                    map.insert(dst, imm);
                }
                Copy { dst, rs1 } => {
                    let v1 = map.get(&rs1).copied();
                    match v1 {
                        Some(imm) => {
                            map.insert(dst, imm);
                            bb.instructions[i] = Const { dst, imm };
                        }
                        None => {
                            is_read.insert(rs1);
                        }
                    }
                }
                Bin { dst, op, rs1, rs2 } => {
                    let v1 = map.get(&rs1).copied();
                    let v2 = map.get(&rs2).copied();

                    match (v1, v2) {
                        (Some(imm1), Some(imm2)) => {
                            let imm = match op {
                                BinOp::Add => imm1 + imm2,
                                BinOp::Sub => imm1 - imm2,
                                BinOp::Mul => imm1 * imm2,
                                BinOp::Div => imm1 / imm2,
                                BinOp::Eq => (imm1 == imm2) as i128,
                                BinOp::Le => (imm1 <= imm2) as i128,
                                BinOp::Lt => (imm1 < imm2) as i128,
                                BinOp::Ge => (imm1 >= imm2) as i128,
                                BinOp::Gt => (imm1 > imm2) as i128,
                            };
                            map.insert(dst, imm);
                            bb.instructions[i] = Const { dst, imm };
                        }
                        _ => {
                            is_read.insert(rs1);
                            is_read.insert(rs2);
                        }
                    }
                }
                Un { dst, op, rs1 } => {
                    let v1 = map.get(&rs1).copied();
                    match v1 {
                        Some(imm1) => {
                            let imm = match op {
                                UnOp::Not => (imm1 == 0) as i128,
                                UnOp::Neg => -imm1,
                            };
                            map.insert(dst, imm);
                            bb.instructions[i] = Const { dst, imm };
                        }
                        None => {
                            is_read.insert(rs1);
                        }
                    }
                }
                Write { rs1, .. } | Arg { rs1, .. } | Br { rs1, .. } => {
                    is_read.insert(rs1);
                }
                _ => {}
            }
        }

        let mut dce = vec![];
        for instr in bb.instructions.iter() {
            match instr {
                Const { dst, .. }
                | Copy { dst, .. }
                | Bin { dst, .. }
                | Un { dst, .. }
                | AddrOf { dst, .. }
                | Read { dst, .. } => {
                    if is_read.contains(dst) {
                        dce.push(*instr);
                    }
                }
                _ => dce.push(*instr),
            }
        }
        _ = std::mem::replace(&mut bb.instructions, dce);
    }
}
