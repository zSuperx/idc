use crate::state::*;
use crate::ast::*;
use crate::IRs::tir::*;


impl GlobalState {
    pub fn validate_obj(&self, Spanned { inner: obj, span }: Spanned<TirObj>) {
        match obj {
            TirObj::Fn { symbol: name, returns, args, body } => {

            },
            TirObj::Global { lhs, rhs } => todo!(),
            TirObj::Struct { name, fields } => todo!(),
        }
    }
}
