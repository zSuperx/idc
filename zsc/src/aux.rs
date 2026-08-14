use std::collections::HashMap;

use crate::{IRs::lir::*, common::*};
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

    pub current_function: Option<Symbol>,

    /// Each entry is COND BLOCK, END BLOCK
    pub loop_labels: Vec<(BBID, BBID)>,
    pub loop_depth: usize,

    pub functions: HashMap<Symbol, HashMap<Symbol, SymbolInfo>>,
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
            value: None,
        };
        self.global_symbols.insert(sym, info);
        spanned_sym
    }

    pub fn get_current_function_info(&self) -> &SymbolInfo {
        let sym = self.current_function.unwrap();
        self.lookup_symbol(sym)
    }

    pub fn add_local_symbol(
        &mut self,
        name: Spanned<&'static str>,
        ty: TypeId,
        kind: SymbolKind,
    ) -> Symbol {
        let sym = self.next_sym(name.inner);
        let None = self.env.insert(name.inner, sym) else {
            die!("Duplicate definition of {name}");
        };
        let info = SymbolInfo {
            raw_name: name,
            ty,
            kind,
            address_taken: false,
            value: None,
        };
        let local_symbols = self
            .functions
            .get_mut(&self.current_function.unwrap())
            .unwrap();
        local_symbols.insert(sym, info);
        sym
    }

    pub fn get_local_symbols_mut(&mut self) -> &mut HashMap<Symbol, SymbolInfo> {
        self.functions.get_mut(&self.current_function.unwrap()).unwrap()
    }

    pub fn lookup_symbol(&self, symbol: Symbol) -> &SymbolInfo {
        let local_symbols = self.functions.get(&self.current_function.unwrap()).unwrap();
        local_symbols
            .get(&symbol)
            .or(self.global_symbols.get(&symbol))
            .unwrap_or_else(|| die!("Symbol {symbol} does not exist"))
    }

    pub fn lookup_symbol_mut(&mut self, symbol: Symbol) -> &mut SymbolInfo {
        let local_symbols = self
            .functions
            .get_mut(&self.current_function.unwrap())
            .unwrap();
        local_symbols
            .get_mut(&symbol)
            .or(self.global_symbols.get_mut(&symbol))
            .unwrap_or_else(|| die!("Symbol {symbol} does not exist"))
    }

    pub fn compile_prog(&mut self) {
        let mut parsed_objects = vec![];
        while !self.is_next(Token::Eof) {
            let o = self.parse_obj();
            parsed_objects.push(o);
        }

        self.collect_all(&parsed_objects);

        let checked_objects: Vec<_> = parsed_objects
            .into_iter()
            .map(|o| self.check_obj(o))
            .collect();

        let mut builder = IRBuilder::default();

        for obj in checked_objects {
            self.lower_func(&mut builder, &obj);
            let f = builder.get_current_function();
            builder.print_function(f);
        }

        // Debug printing
        // for (k, v) in self
        //     .global_symbols
        //     .iter()
        //     .chain(self.func.local_symbols.iter())
        // {
        //     let SymbolInfo {
        //         raw_name,
        //         ty,
        //         kind,
        //         address_taken,
        //         value,
        //         ..
        //     } = v;
        //     println!("Symbol {k}");
        //     println!("Type: {ty}");
        //     println!("Value: {value}");
        //     println!("Raw name: {raw_name}\n");
        // }
        die!("End");
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SymbolKind {
    Local,
    Arg(usize),
    Global,
    Function,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub raw_name: Spanned<&'static str>,
    pub ty: TypeId,
    pub kind: SymbolKind,
    pub address_taken: bool,
    pub value: Option<Value>,
}

