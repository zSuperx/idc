use std::collections::HashMap;

use crate::{arch::lir::*, prelude::*};

use registry::Registry;

use crate::{
    ast::*,
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
    pub func: FnCtx,

    pub symbols: Registry<String>, // Uniquely ID'd scoped identifiers
    pub var_count: usize,          // Distinguishes shadowed vars
    pub known_types: Registry<RealType>, // Uniquely ID'd types.
    pub builtin_types: HashMap<&'static str, RealType>,

    // Lowering shit
    pub bb_count: usize,
}

impl Compiler {
    pub fn new() -> Self {
        let mut s = Self::default();

        #[rustfmt::skip]
        let builtin_types = [
            RealType::I8,   RealType::U8,
            RealType::I16,  RealType::U16,
            RealType::I32,  RealType::U32,
            RealType::I64,  RealType::U64,
            RealType::Bool, RealType::Void
        ];

        for ty in builtin_types {
            let ty_str = ty.to_string().leak();
            s.builtin_types.insert(ty_str, ty.clone());
            s.known_types.add(ty);
        }
        s
    }

    pub fn compile_prog(&mut self) -> Vec<Builder<LirInstr>> {
        let mut buf = vec![];
        while !self.is_next(Token::Eof) {
            _ = std::mem::take(&mut self.func);
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
    pub name: StringId,
    pub ty: TypeId,
    pub kind: SymbolKind,
    pub address_taken: bool,
}

use crate::ast::Spanned;

#[derive(Debug, Default)]
pub struct FnCtx {
    pub raw_name: Spanned<&'static str>,
    pub env: Env<&'static str, (StringId, TypeId)>, // Tracks scopes and string -> var, type mappings
    pub return_type: Option<Spanned<TypeId>>,       // Return type of current function
    pub symbol_table: HashMap<StringId, SymbolInfo>,
    pub var2val: HashMap<StringId, LirVal>,
}
