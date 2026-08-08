use std::collections::HashMap;

use crate::{arch::lir::*, prelude::*};

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
    pub env: Env<&'static str, Symbol>, // Tracks scopes and string -> var, type mappings
    pub global_symbols: HashMap<Symbol, SymbolInfo>,

    pub resolved_types: Registry<ResolvedType>, // Uniquely ID'd resolved types.
    pub raw_types: Registry<RawType>,           // Uniquely ID'd raw types.
    pub symbols: Registry<String>,              // Uniquely ID'd scoped identifiers
    pub symbol_counter: usize,                  // Distinguishes shadowed vars
    pub builtin_types: HashMap<&'static str, ResolvedType>,
}

impl Compiler {
    pub fn new(filename: &'static str) -> Self {
        let mut s = Self::default();
        s.filename = filename;

        #[rustfmt::skip]
        let builtin_types = [
            ResolvedType::I8,   ResolvedType::U8,
            ResolvedType::I16,  ResolvedType::U16,
            ResolvedType::I32,  ResolvedType::U32,
            ResolvedType::I64,  ResolvedType::U64,
            ResolvedType::Bool, ResolvedType::Void
        ];

        for ty in builtin_types {
            let ty_str = ty.to_string().leak();
            s.builtin_types.insert(ty_str, ty.clone());
            s.resolved_types.add(ty);
        }
        s
    }

    pub fn next_sym(&mut self, argname: &str) -> Symbol {
        let id = self.symbol_counter;
        self.symbol_counter += 1;
        self.symbols.add(format!("{argname}.{id}"))
    }

    pub fn add_global_symbol(
        &mut self,
        name: Spanned<&'static str>,
        ty: ResolvedTypeId,
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
        };
        self.global_symbols.insert(sym, info);
        spanned_sym
    }

    pub fn add_local_symbol(
        &mut self,
        name: Spanned<&'static str>,
        ty: ResolvedTypeId,
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
        };
        self.func.local_symbols.insert(sym, info);
        spanned_sym
    }

    pub fn lookup_symbol(&mut self, symbol: Symbol) -> &mut SymbolInfo {
        self.func
            .local_symbols
            .get_mut(&symbol)
            .or(self.global_symbols.get_mut(&symbol))
            .unwrap_or_else(|| die!("Symbol {symbol} does not exist"))
    }

    pub fn get_symbol_info(&mut self, raw_name: Spanned<&str>) -> &mut SymbolInfo {
        let Some(sym) = self.env.get(&raw_name.inner) else {
            die!("Variable used but not defined: {raw_name}");
        };
        self.global_symbols.get_mut(&sym).unwrap()
    }

    pub fn compile_prog(&mut self) -> Vec<Builder<LirInstr>> {
        let mut buf = vec![];
        let mut parsed_objects = vec![];
        while !self.is_next(Token::Eof) {
            let o = self.parse_obj();
            parsed_objects.push(o);
        }

        let mut checked_objects: Vec<_> = parsed_objects
            .into_iter()
            .map(|o| {
                match &o.inner {
                    crate::hir::HirObj::Fn {
                        name,
                        returns,
                        args,
                        body,
                    } => self.func.raw_name = *name,
                    _ => todo!(),
                }
                self.check_obj(o)
            })
            .collect();

        // Debug printing
        for (k, v) in self.global_symbols.iter() {
            let SymbolInfo {
                raw_name,
                ty,
                kind,
                address_taken,
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
    pub ty: ResolvedTypeId,
    pub kind: SymbolKind,
    pub address_taken: bool,
}

use crate::ast::Spanned;

#[derive(Debug, Default)]
pub struct FnCtx {
    pub raw_name: Spanned<&'static str>,
    pub local_symbols: HashMap<Symbol, SymbolInfo>,
    pub var2val: HashMap<Symbol, LirVal>,
}
