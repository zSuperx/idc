//! This file is responsible for collecting all types, then all functions, and registering them with
//! the global type and symbol tables

use crate::{
    IRs::hir::*,
    ast::{Spanned, Type},
    aux::{Compiler, SymbolKind},
};

impl Compiler {
    pub fn collect_all(&mut self, objects: &Vec<Spanned<HirObj>>) {
        for obj in objects {
            self.resolve_types(obj);
        }

        for obj in objects {
            self.resolve_globals(obj);
        }
    }

    fn resolve_types(&mut self, Spanned { inner: obj, span }: &Spanned<HirObj>) {
        match obj {
            HirObj::Fn {
                name,
                returns,
                args,
                body,
            } => {}
            HirObj::Global { name, ty, rhs } => {}
            HirObj::Struct { name, fields } => {
                self.add_type(Type::Base(name.inner));
            }
        }
    }

    fn resolve_globals(&mut self, Spanned { inner: obj, span }: &Spanned<HirObj>) {
        match obj {
            HirObj::Fn {
                name,
                returns,
                args,
                body,
            } => {
                let mut arg_types = vec![];
                for (_, ty) in args.iter() {
                    let resolved_ty = self.check_type(ty);
                    arg_types.push(resolved_ty);
                }
                let return_ty = self.check_type(returns);
                let function_ty = Type::Function {
                    args: arg_types,
                    returns: return_ty,
                };
                let ty_id = self.add_type(function_ty);
                self.add_global_symbol(name.clone(), ty_id, SymbolKind::Function);
            }
            HirObj::Global { name, ty, rhs } => {
                todo!()
            }
            HirObj::Struct { name, fields } => {}
        }
    }
}
