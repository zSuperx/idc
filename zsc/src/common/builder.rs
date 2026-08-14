use std::collections::{HashMap, HashSet};

use crate::{
    IRs::lir::{
        LirInstr::{self, *},
        Value,
    }, ast::TypeId, common::Instr
};

#[derive(Debug, Clone, Copy, Default, Hash, Eq, PartialEq)]
pub struct BBID(&'static str, &'static str, usize);

#[derive(Debug, Clone, Default)]
pub struct IRBuilder {
    cursor: BBID,
    reg_count: usize,
    block_count: usize,
    functions: HashMap<&'static str, IRFunction>,
}

#[derive(Debug, Clone)]
struct IRFunction {
    entrypoint: BBID,
    blocks: HashMap<BBID, BasicBlock>,
}

impl IRBuilder {
    pub fn get_current_block(&self) -> BBID {
        self.cursor
    }

    pub fn set_insert_point(&mut self, id: BBID) {
        self.cursor = id;
    }

    pub fn get_current_function(&self) -> &'static str {
        self.cursor.0
    }

    pub fn create_function(&mut self, function_name: &'static str) {
        let id = BBID(function_name, "entry", self.block_count);
        self.block_count += 1;
        let mut function = IRFunction {
            entrypoint: id,
            blocks: HashMap::default(),
        };

        function.blocks.insert(id, BasicBlock::default());

        let None = self.functions.insert(function_name, function) else {
            panic!("Duplicate function detected: {function_name}");
        };

        self.cursor = id;
    }

    pub fn create_block(&mut self, function_name: &'static str) -> BBID {
        let function = self.functions.get_mut(function_name).unwrap();

        let id = BBID(function_name, "", self.block_count);
        self.block_count += 1;

        function.blocks.insert(id, BasicBlock::default());

        return id;
    }

    pub fn create_blockn(&mut self, function_name: &'static str, block_name: &'static str) -> BBID {
        let function = self.functions.get_mut(function_name).unwrap();

        let id = BBID(function_name, block_name, self.block_count);
        self.block_count += 1;

        function.blocks.insert(id, BasicBlock::default());

        return id;
    }

    #[inline]
    pub fn add_successors(&mut self, successors: &[BBID]) {
        self.add_successors_to(self.cursor, successors);
    }

    pub fn add_successors_to(&mut self, this: BBID, successors: &[BBID]) {
        let IRFunction { blocks, .. } = self.functions.get_mut(this.0).unwrap();

        for succ_id in successors {
            assert_eq!(this.0, succ_id.0);
            let this_block = blocks.get_mut(&this).unwrap();
            this_block.successors.insert(*succ_id);

            let other_block = blocks.get_mut(&succ_id).unwrap();
            other_block.predecessors.insert(this);
        }
    }

    #[inline]
    pub fn add_predeessors(&mut self, successors: &[BBID]) {
        self.add_predecessors_to(self.cursor, successors);
    }

    pub fn add_predecessors_to(&mut self, this: BBID, predecessors: &[BBID]) {
        let IRFunction { blocks, .. } = self.functions.get_mut(this.0).unwrap();

        for pred_id in predecessors {
            assert_eq!(this.0, pred_id.0);
            let this_block = blocks.get_mut(&this).unwrap();
            this_block.predecessors.insert(*pred_id);

            let other_block = blocks.get_mut(&pred_id).unwrap();
            other_block.successors.insert(this);
        }
    }

    pub fn emit(&mut self, instr: LirInstr) {
        let BBID(function_name, ..) = self.cursor;
        let IRFunction { blocks, .. } = self.functions.get_mut(function_name).unwrap();
        let basic_block = blocks.get_mut(&self.cursor).unwrap();
        if basic_block.terminator.is_some() {
            return;
        }
        if instr.is_terminator() {
            basic_block.terminator = Some(instr);
        } else {
            basic_block.instructions.push(instr);
        }
    }

    pub fn emit_br(&mut self, ty: TypeId, cond_val: Value, true_block: BBID, false_block: BBID) {
        let bool_val = self.next_reg();
        self.emit(Eq(ty, bool_val, cond_val, Value::imm(1)));
        self.emit(Br(bool_val, true_block, false_block));
    }

    pub fn next_reg(&mut self) -> Value {
        let ret = self.reg_count;
        self.reg_count += 1;
        Value::reg(ret)
    }

    pub fn next_mem(&mut self) -> Value {
        let ret = self.reg_count;
        self.reg_count += 1;
        Value::mem(ret)
    }

    pub fn print_function(&self, function_name: &'static str) {
        let function = self.functions.get(function_name).unwrap();
        println!("{function_name}:");
        let mut seen = HashSet::new();
        let mut stack = vec![function.entrypoint];
        while let Some(curr) = stack.pop() {
            let block = &function.blocks[&curr];
            seen.insert(curr);
            println!("{curr}:");
            for i in block.instructions.iter() {
                println!("\t{i}");
            }
            if let Some(term) = &block.terminator {
                println!("\t{term}");
            }
            for succ in block.successors.iter() {
                if !seen.contains(succ) {
                    stack.push(*succ);
                }
            }
        }
    }

    pub fn verify(&mut self, function_name: &'static str, has_return_value: bool) -> bool {
        let function = self.functions.get_mut(function_name).unwrap();
        let mut seen = HashSet::new();
        let mut stack = vec![function.entrypoint];
        while let Some(curr) = stack.pop() {
            let block = function.blocks.get_mut(&curr).unwrap();
            seen.insert(curr);
            if block.terminator.is_none() {
                if has_return_value {
                    return false;
                } else {
                    block.terminator = Some(Retv);
                }
            }
            for succ in block.successors.iter() {
                if !seen.contains(succ) {
                    stack.push(*succ);
                }
            }
        }
        return true;
    }
}

impl std::fmt::Display for BBID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let BBID(function, name, index) = self;
        if name.is_empty() {
            f.write_fmt(format_args!(".{function}.{index}"))
        } else {
            f.write_fmt(format_args!(".{name}.{index}"))
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BasicBlock {
    pub successors: HashSet<BBID>,
    pub predecessors: HashSet<BBID>,
    pub instructions: Vec<LirInstr>,
    pub terminator: Option<LirInstr>,
}
