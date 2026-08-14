use std::collections::{HashMap, HashSet};

use crate::common::Instr;

#[derive(Debug, Clone, Copy, Default, Hash, Eq, PartialEq)]
pub struct BBID(&'static str, &'static str, usize);

#[derive(Debug, Clone)]
pub struct IRBuilder<I: Instr> {
    cursor: BBID,
    pub reg_count: usize,
    pub block_count: usize,
    functions: HashMap<&'static str, IRFunction<I>>,
}

#[derive(Debug, Clone)]
pub struct IRFunction<I: Instr> {
    pub entrypoint: BBID,
    pub blocks: HashMap<BBID, BasicBlock<I>>,
}

impl<I: Instr> Default for IRBuilder<I> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<I: Instr> IRBuilder<I> {
    pub fn empty() -> Self {
        Self {
            cursor: Default::default(),
            reg_count: Default::default(),
            block_count: Default::default(),
            functions: Default::default(),
        }
    }

    pub fn with_state<O: Instr>(other: &IRBuilder<O>) -> Self {
        let mut s = Self::empty();
        s.cursor = other.cursor;
        s.reg_count = other.reg_count;
        s.block_count = other.block_count;
        for (name, function) in other.get_all_functions() {
            let mut empty_function = IRFunction {
                entrypoint: function.entrypoint,
                blocks: Default::default(),
            };
            for (id, block) in function.blocks.iter() {
                let empty_block = BasicBlock {
                    successors: block.successors.clone(),
                    predecessors: block.predecessors.clone(),
                    instructions: Default::default(),
                    terminator: Default::default(),
                };
                empty_function.blocks.insert(*id, empty_block);
            }
            s.functions.insert(name, empty_function);
        }
        s
    }

    pub fn get_current_block(&self) -> BBID {
        self.cursor
    }

    pub fn set_insert_point(&mut self, id: BBID) {
        self.cursor = id;
    }

    pub fn get_current_function(&self) -> &'static str {
        self.cursor.0
    }

    pub fn get_all_functions(&self) -> &HashMap<&'static str, IRFunction<I>> {
        &self.functions
    }

    pub fn create_function(&mut self, function_name: &'static str) {
        let id = BBID(function_name, "entry", self.block_count);
        self.block_count += 1;
        let mut function = IRFunction {
            entrypoint: id,
            blocks: HashMap::default(),
        };

        function.blocks.insert(id, BasicBlock::empty());

        let None = self.functions.insert(function_name, function) else {
            panic!("Duplicate function detected: {function_name}");
        };

        self.cursor = id;
    }

    pub fn create_block(&mut self, function_name: &'static str) -> BBID {
        let function = self.functions.get_mut(function_name).unwrap();

        let id = BBID(function_name, "", self.block_count);
        self.block_count += 1;

        function.blocks.insert(id, BasicBlock::empty());

        return id;
    }

    pub fn create_blockn(&mut self, function_name: &'static str, block_name: &'static str) -> BBID {
        let function = self.functions.get_mut(function_name).unwrap();

        let id = BBID(function_name, block_name, self.block_count);
        self.block_count += 1;

        function.blocks.insert(id, BasicBlock::empty());

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
    pub fn add_predecessors(&mut self, successors: &[BBID]) {
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

    pub fn emit(&mut self, instr: I) {
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

    pub fn is_current_terminated(&self) -> bool {
        let IRFunction { blocks, .. } = self.functions.get(self.cursor.0).unwrap();
        blocks[&self.cursor].terminator.is_some()
    }

    pub fn is_terminated(&self, this: BBID) -> bool {
        let IRFunction { blocks, .. } = self.functions.get(this.0).unwrap();
        blocks[&this].terminator.is_some()
    }

    pub fn next_reg(&mut self) -> usize {
        let ret = self.reg_count;
        self.reg_count += 1;
        ret
    }

    pub fn print_all_functions(&self) {
        for (f, _) in self.functions.iter() {
            self.print_function(f);
            println!("\n");
        }
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
            } else {
                println!("\t; (missing terminator)");
            }
            for succ in block.successors.iter() {
                if !seen.contains(succ) {
                    stack.push(*succ);
                }
            }
        }
    }

    pub fn verify(&mut self, function_name: &'static str, default_return_instr: Option<I>) -> bool {
        let function = self.functions.get_mut(function_name).unwrap();
        let mut seen = HashSet::new();
        let mut stack = vec![function.entrypoint];
        while let Some(curr) = stack.pop() {
            let block = function.blocks.get_mut(&curr).unwrap();
            seen.insert(curr);
            if block.terminator.is_none() {
                if default_return_instr.is_some() {
                    block.terminator = default_return_instr.clone();
                } else {
                    return false;
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

#[derive(Debug, Clone)]
pub struct BasicBlock<I: Instr> {
    pub successors: HashSet<BBID>,
    pub predecessors: HashSet<BBID>,
    pub instructions: Vec<I>,
    pub terminator: Option<I>,
}

impl<I: Instr> BasicBlock<I> {
    pub fn empty() -> Self {
        Self {
            successors: Default::default(),
            predecessors: Default::default(),
            instructions: Default::default(),
            terminator: Default::default(),
        }
    }
}
