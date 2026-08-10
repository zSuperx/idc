use std::collections::HashMap;

use crate::{arch::lir::*, common::*};
use std::cell::RefCell;
use std::rc::Rc;

use registry::Registry;

use crate::ast::*;

#[derive(Default)]
pub struct Compiler {
    // Lexer shit
    pub filename: &'static str,
    pub cursor: usize,
    pub row: usize,
    pub col: usize,

    // Parser shit
    pub last_span: Span,

    pub func: FnCtx,
    pub env: Env<&'static str, Symbol>, // Tracks scopes and string -> symbol mappings
    pub global_symbols: HashMap<Symbol, SymbolInfo>,

    pub type_names: HashMap<&'static str, TypeId>,
    pub types: Rc<RefCell<Registry<Type>>>, // Uniquely ID'd resolved types.
    pub symbols: Rc<RefCell<Registry<String>>>, // Uniquely ID'd scoped identifiers
    pub symbol_counter: usize,              // Distinguishes shadowed vars
}

impl Compiler {
    pub fn new(filename: &'static str) -> Self {
        let mut s = Self::default();
        s.filename = filename;

        #[rustfmt::skip]
        let builtin_types = [
            Type::I8,   Type::U8,
            Type::I16,  Type::U16,
            Type::I32,  Type::U32,
            Type::I64,  Type::U64,
            Type::Bool, Type::Void
        ];

        for ty in builtin_types {
            let ty_str = ty.to_string().leak();
            let id = s.types.borrow_mut().add(ty);
            s.type_names.insert(ty_str, id);
        }
        s
    }

    pub fn add_type(&mut self, ty: Type) -> TypeId {
        self.types.borrow_mut().add(ty)
    }

    pub fn current_function(&self) -> Symbol {
        self.func.symbol.unwrap()
    }

    pub fn next_sym(&mut self, argname: &str) -> Symbol {
        let id = self.symbol_counter;
        self.symbol_counter += 1;
        self.symbols.borrow_mut().add(format!("{argname}.{id}"))
    }

    pub fn add_global_symbol(
        &mut self,
        name: Spanned<&'static str>,
        ty: TypeId,
        kind: SymbolKind,
    ) -> Spanned<Symbol> {
        let sym = self.next_sym(name.inner);
        let spanned_sym = Spanned::new(sym, name.span);
        let None = self.env.insert_first(name.inner, sym) else {
            die!("Duplicate definition of {name}");
        };
        let info = SymbolInfo {
            raw_name: name,
            ty,
            kind,
            address_taken: false,
            value: Value::uninit(),
        };
        self.global_symbols.insert(sym, info);
        spanned_sym
    }

    pub fn add_local_symbol(
        &mut self,
        name: Spanned<&'static str>,
        ty: TypeId,
        kind: SymbolKind,
    ) -> Spanned<Symbol> {
        let sym = self.next_sym(name.inner);
        let spanned_sym = Spanned::new(sym, name.span);
        let None = self.env.insert(name.inner, sym) else {
            die!("Duplicate definition of {name}");
        };
        let info = SymbolInfo {
            raw_name: name,
            ty,
            kind,
            address_taken: false,
            value: Value::uninit(),
        };
        self.func.local_symbols.insert(sym, info);
        spanned_sym
    }

    pub fn lookup_symbol(&self, symbol: Symbol) -> &SymbolInfo {
        self.func
            .local_symbols
            .get(&symbol)
            .or(self.global_symbols.get(&symbol))
            .unwrap_or_else(|| die!("Symbol {symbol} does not exist"))
    }

    pub fn lookup_symbol_mut(&mut self, symbol: Symbol) -> &mut SymbolInfo {
        self.func
            .local_symbols
            .get_mut(&symbol)
            .or(self.global_symbols.get_mut(&symbol))
            .unwrap_or_else(|| die!("Symbol {symbol} does not exist"))
    }

    pub fn compile_prog(&mut self) -> Vec<Builder<LirInstr>> {
        let mut buf = vec![];
        let mut parsed_objects = vec![];
        while !self.is_next(Token::Eof) {
            let o = self.parse_obj();
            parsed_objects.push(o);
        }

        self.collect_all(&parsed_objects);

        let mut checked_objects: Vec<_> = parsed_objects
            .into_iter()
            .map(|o| self.check_obj(o))
            .collect();

        // Debug printing
        for (k, v) in self
            .global_symbols
            .iter()
            .chain(self.func.local_symbols.iter())
        {
            let SymbolInfo {
                raw_name,
                ty,
                kind,
                address_taken,
                ..
            } = v;
            println!("Symbol {k}");
            println!("Type: {ty}");
            println!("Raw name: {raw_name}\n\n");
        }
        die!("End");
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
    pub raw_name: Spanned<&'static str>,
    pub ty: TypeId,
    pub kind: SymbolKind,
    pub address_taken: bool,
    pub value: Value,
}

#[derive(Debug, Default)]
pub struct FnCtx {
    pub raw_name: Spanned<&'static str>,
    pub symbol: Option<Symbol>,
    pub local_symbols: HashMap<Symbol, SymbolInfo>,
    pub var2val: HashMap<Symbol, Value>,
}
