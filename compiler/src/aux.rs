use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use registry::Registry;

use crate::{
    ast::*,
    lir::{self, BB, BasicBlock, Instr, LIRFunction, LirVal, VVal},
    tir::{TirType, TypeId, VarId},
    utils::Env,
};

#[derive(Default)]
pub struct Compiler {
    // Lexer shit
    pub cursor: usize,
    pub row: usize,
    pub col: usize,

    // Parser shit
    pub last_span: Span,

    // This context is reset per function
    pub curr_fn: LIRFunction,

    pub symbols: Registry<String>, // Uniquely ID'd scoped identifiers
    pub var_count: usize,          // Distinguishes shadowed vars
    pub known_types: Registry<TirType>, // Uniquely ID'd types.
    pub builtin_types: HashMap<&'static str, TirType>,

    // Lowering shit
    pub bb_count: usize,
}

impl Compiler {
    pub fn new() -> Self {
        let mut s = Self::default();

        #[rustfmt::skip]
        let builtin_types = [
            TirType::I8,   TirType::U8,
            TirType::I16,  TirType::U16,
            TirType::I32,  TirType::U32,
            TirType::I64,  TirType::U64,
            TirType::Bool, TirType::Void
        ];

        for ty in builtin_types {
            let ty_str = ty.to_string().leak();
            s.builtin_types.insert(ty_str, ty.clone());
            s.known_types.add(ty);
        }
        s
    }

    pub fn compile_prog(&mut self) -> Vec<LIRFunction> {
        let mut buf = vec![];
        while !self.is_next(Token::Eof) {
            _ = std::mem::take(&mut self.curr_fn);
            // TODO: add all functions to self.curr_fn
            // FnCtx {
            //     env: todo!(), // Populate env
            //     symbol_table: todo!(), // Populate symbol table
            //     return_type: todo!(), // this can be ignored
            // }
            let f = self.parse_obj();
            let f = self.check_obj(f);
            let f = self.lower_func(f);
            let f = self.optim_func(f);
            buf.push(f);
        }
        buf
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SymbolKind {
    Local,
    Param(usize),
    Global,
    Function,
}

#[derive(Debug, Clone, Copy)]
pub struct SymbolInfo {
    pub name: VarId,
    pub ty: TypeId,
    pub kind: SymbolKind,
    pub address_taken: bool,
}
