//! This file is responsible for collecting all types, then all functions, and registering them with
//! the global type and symbol tables

use crate::{IRs::hir::*, ast::Spanned, aux::Compiler};


impl Compiler {
    pub fn collect_all(&mut self, objects: &Vec<Spanned<HirObj>>) {
        for obj in objects {
            self.collect_types(obj);
        }

        for obj in objects {
            self.collect_global_symbols(obj);
        }
    }

    fn collect_types(&mut self, Spanned { inner: obj, span }: &Spanned<HirObj>) {

    }

    fn collect_global_symbols(&mut self, Spanned { inner: obj, span }: &Spanned<HirObj>) {

    }
}
