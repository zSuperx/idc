use crate::{
    IRs::{hir::*, sir::*},
    ast::{ResolvedType, Spanned},
    state::{Compiler, SymbolKind},
};

impl Compiler {
    pub fn symbolize_obj(
        &mut self,
        Spanned { inner: obj, span }: Spanned<HirObj>,
    ) -> Spanned<SirObj> {
        match obj {
            HirObj::Fn {
                name,
                returns,
                args,
                body,
            } => todo!(),
            HirObj::Global { lhs, rhs } => todo!(),
            HirObj::Struct { name, fields } => todo!(),
        }
    }

    fn symbolize_stmt(
        &mut self,
        Spanned { inner: stmt, span }: Spanned<HirStmt>,
    ) -> Spanned<SirStmt> {
        match stmt {
            HirStmt::Let { lhs, ty, rhs } => todo!(),
            HirStmt::While { cond, body } => todo!(),
            HirStmt::Continue => todo!(),
            HirStmt::Break => todo!(),
            HirStmt::If { cond, then_, else_ } => todo!(),
            HirStmt::Return(spanned) => todo!(),
            HirStmt::Block(spanneds) => todo!(),
            HirStmt::Expr(spanned) => todo!(),
        }
    }

    fn symbolize_expr(
        &mut self,
        Spanned { inner: expr, span }: Spanned<HirExpr>,
    ) -> Spanned<SirExpr> {
        let inner = match expr {
            HirExpr::Void => SirExpr::Void,
            HirExpr::Num(val) => SirExpr::Num(val),
            HirExpr::Bool(val) => SirExpr::Bool(val),
            HirExpr::Ident(val) => {
                let id = self.resolved_types.add(ResolvedType::Unknown);
                let sym = self.add_local_symbol(Spanned::new(val, span), id, SymbolKind::Local);
                SirExpr::Ident(sym.inner)
            }
            HirExpr::Assign { lhs, rhs } => todo!(),
            HirExpr::AddrOf { rhs } => todo!(),
            HirExpr::SizeOfTy { ty } => todo!(),
            HirExpr::SizeOfExpr { expr } => todo!(),
            HirExpr::Deref { rhs } => todo!(),
            HirExpr::Index { expr, index } => todo!(),
            HirExpr::Un { op, rhs } => todo!(),
            HirExpr::Bin { op, lhs, rhs } => todo!(),
            HirExpr::Cast { target_ty, rhs } => todo!(),
            HirExpr::Call { callee, args } => todo!(),
        };
        Spanned::new(inner, span)
    }
}
