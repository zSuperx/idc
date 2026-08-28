use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, OnceLock};

use crate::IRs::hir::{HirObj, ParsedFunction};
use crate::IRs::tir::TirStmt;
use crate::common::*;
use std::cell::RefCell;
use std::rc::Rc;

use registry::Registry;
use stir::builder::IRBB;
use stir::isa::IRValue;

use crate::ast::*;

#[derive(Default)]
pub struct GlobalState {
    // Lexer shit
    pub filename: &'static str,
    pub cursor: usize,
    pub row: usize,
    pub col: usize,

    // Parser shit
    pub last_span: Span,

    pub current_function: Option<Symbol>,

    /// Global string to symbol map
    ///
    /// Contains names of functions and global variables
    pub globals: HashMap<&'static str, Symbol>,

    /// Global symbol table
    pub symbol_table: HashMap<Symbol, SymbolInfo>,

    pub type_names: HashMap<&'static str, TypeId>,
    /// Uniquely ID'd resolved types.
    pub types: Rc<RefCell<Registry<Type>>>,
    /// Uniquely ID'd scoped identifiers
    pub symbols: Rc<RefCell<Registry<String>>>,
    /// Distinguishes shadowed vars
    pub symbol_counter: usize,
}

pub static mut GLOBAL_STATE: LazyLock<GlobalState> = LazyLock::new(GlobalState::new);
pub static SOURCE: OnceLock<Vec<u8>> = OnceLock::new();
pub static mut STRINGS: LazyLock<HashSet<String>> = LazyLock::new(HashSet::new);

pub fn source() -> &'static Vec<u8> {
    SOURCE.get().unwrap()
}

pub fn add_str(value: &str) -> &'static str {
    unsafe {
        if let Some(s) = STRINGS.get(value) {
            s
        } else {
            STRINGS.insert(value.to_string());
            STRINGS.get(value).unwrap()
        }
    }
}

pub fn get_state() -> &'static mut GlobalState {
    unsafe { &mut GLOBAL_STATE }
}

pub fn add_type(ty: Type) -> TypeId {
    get_state().types.borrow_mut().add(ty)
}

pub fn next_symbol(name: &str) -> Symbol {
    let state = get_state();
    let id = state.symbol_counter;
    state.symbol_counter += 1;
    state.symbols.borrow_mut().add(format!("{name}.{id}"))
}

#[derive(Debug)]
pub struct Function {
    /// The raw name of the function
    pub name: Spanned<&'static str>,
    /// The symbol its mapped to (can be retrieved via global state, but put here for convenience)
    pub symbol: Symbol,

    /// Tracks string -> symbol mappings. Lookups searches in an inner-most to outer-most fashion
    pub env: Env<&'static str, Symbol>,

    /// Used by the codegen phase.
    /// Each entry is COND_BB, END_BB
    /// That is, `continue` jumps to COND_BB, and `break` jumps to END_BB
    pub loop_labels: Vec<(IRBB, IRBB)>,

    /// Loop depth tracks how many loops deep we are: +1 when entering, -1 when leaving
    /// Allows us to determine if `break` or `continue` were called from outside a loop
    pub loop_depth: usize,

    /// The symbol table for this function. Holds info on local variables and function parameters
    pub symbol_table: HashMap<Symbol, SymbolInfo>,

    pub return_type: TypeId,

    /// The AST node of the function
    pub node: Option<TirStmt>,
}

impl GlobalState {
    pub fn new() -> Self {
        let mut s = Self::default();

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
    pub value: Option<IRValue>,
}

/// Reads all parsed objects and registers global symbols and global types
pub fn resolve_top_level(objects: &Vec<Spanned<HirObj>>) {
    for obj in objects {
        resolve_global_types(obj);
    }

    for obj in objects {
        resolve_global_names(obj);
    }
}

/// Resolves a Type::Unresolved(..) into a Type
pub fn resolve_type(s @ Spanned { inner: ty, span }: &Spanned<TypeId>) -> TypeId {
    match ty.lookup() {
        Type::Unresolved(name) => {
            let Some(id) = get_state().type_names.get(name) else {
                die!("Unknown type {name}: {span}")
            };
            *id
        }
        Type::Function { args, returns } => todo!(),
        Type::Pointer(id) => {
            let inner_ty = resolve_type(&Spanned::new(*id, *span));
            add_type(Type::Pointer(inner_ty))
        }
        _ => s.inner,
    }
}

fn resolve_global_types(Spanned { inner: obj, span }: &Spanned<HirObj>) {
    match obj {
        HirObj::Fn(ParsedFunction {
            name,
            returns,
            args,
            body,
        }) => {}
        HirObj::Global { name, ty, rhs } => {}
        HirObj::Struct { name, fields } => {
            add_type(Type::Base(name.inner));
        }
    }
}

fn resolve_global_names(Spanned { inner: obj, span }: &Spanned<HirObj>) {
    match obj {
        HirObj::Fn(ParsedFunction {
            name,
            returns,
            args,
            body,
        }) => {
            let mut arg_types = vec![];
            for (_, ty) in args.iter() {
                let resolved_ty = resolve_type(ty);
                arg_types.push(resolved_ty);
            }
            let return_ty = resolve_type(returns);
            let function_ty = Type::Function {
                args: arg_types,
                returns: return_ty,
            };
            let ty = add_type(function_ty);
            let symbol = next_symbol(name.inner);
            get_state().globals.insert(name.inner, symbol);
            get_state().symbol_table.insert(
                symbol,
                SymbolInfo {
                    raw_name: *name,
                    ty,
                    kind: SymbolKind::Function,
                    address_taken: false,
                    value: None,
                },
            );
        }
        HirObj::Global { name, ty, rhs } => {
            todo!()
        }
        HirObj::Struct { name, fields } => {}
    }
}
