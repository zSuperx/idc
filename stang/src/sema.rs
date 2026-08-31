use crate::IRs::{hir::*, tir::*};
use crate::ast::*;
use crate::common::*;
use crate::state::*;
use crate::state::{Function, SymbolInfo, SymbolKind, add_type, get_state, next_symbol};

impl Function {
    fn add_local_symbol(
        &mut self,
        name: Spanned<&'static str>,
        ty: TypeId,
        kind: SymbolKind,
    ) -> Symbol {
        let symbol = next_symbol(name.inner);
        self.env.insert(name.inner, symbol);
        self.symbol_table.insert(
            symbol,
            SymbolInfo {
                symbol,
                raw_name: name,
                ty,
                kind,
                address_taken: Default::default(),
                value: Default::default(),
            },
        );
        symbol
    }

    pub fn lookup_symbol(&mut self, symbol: Symbol) -> &SymbolInfo {
        match self.symbol_table.get(&symbol) {
            Some(i) => i,
            None => get_state().symbol_table.get(&symbol).unwrap_or_else(|| {
                die!("Symbol not found: {symbol}");
            }),
        }
    }

    pub fn lookup_symbol_mut(&mut self, symbol: Symbol) -> &mut SymbolInfo {
        match self.symbol_table.get_mut(&symbol) {
            Some(i) => i,
            None => get_state().symbol_table.get_mut(&symbol).unwrap(),
        }
    }

    pub fn new(
        ParsedFunction {
            name,
            returns,
            args,
            body,
        }: ParsedFunction,
    ) -> Self {
        let mut function = Self {
            name,
            symbol: get_state().globals[name.inner],
            return_type: resolve_type(&returns),
            env: Default::default(),
            loop_labels: Default::default(),
            loop_depth: 0,
            symbol_table: Default::default(),
            body: None,
        };

        // Type check the function header
        function.env.push_scope();
        let mut checked_args = vec![];
        for (i, (argname, ty)) in args.into_iter().enumerate() {
            let var_ty = resolve_type(&ty);
            if *var_ty == Type::Void {
                die!("Function argument cannot have type {var_ty}");
            }

            let argsym = next_symbol(argname.inner);
            function.env.insert(argname.inner, argsym);
            function.symbol_table.insert(
                argsym,
                SymbolInfo {
                    symbol: argsym,
                    raw_name: argname,
                    ty: var_ty,
                    kind: SymbolKind::Arg(i),
                    address_taken: false,
                    value: None,
                },
            );

            checked_args.push((argsym, var_ty));
        }

        // Type check the function body
        let body = function.check_stmt(*body);
        function.env.pop_scope();
        function.body = Some(body);
        function
    }

    fn check_stmt(&mut self, Spanned { inner: stmt, span }: Spanned<HirStmt>) -> TirStmt {
        match stmt {
            HirStmt::Let { lhs, ty, rhs } => {
                let ty = ty.map(|t| resolve_type(&t));
                let checked_rhs = self.check_rvalue_expr(&rhs, ty);

                if let Some(ty) = ty
                    && ty != checked_rhs.ty
                {
                    die!(
                        "Type mismatch. Expected {ty} but got {}: {span}",
                        checked_rhs.ty,
                    );
                }
                let var_ty = checked_rhs.ty;

                let lhs_symbol = self.add_local_symbol(lhs, var_ty, SymbolKind::Local);

                TirStmt::Let {
                    lhs: lhs_symbol,
                    ty,
                    rhs: checked_rhs,
                }
            }
            HirStmt::While { cond, body } => {
                let checked_cond = self.check_rvalue_expr(&cond, None);
                let cond_ty = checked_cond.ty;
                if *cond_ty != Type::Bool {
                    die!(
                        "Type mismatch. Expected `{}` but got {}",
                        Type::Bool,
                        span.wrap(cond_ty)
                    )
                }
                self.env.push_scope();
                self.loop_depth += 1;
                let checked_body = self.check_stmt(*body);
                self.loop_depth -= 1;
                self.env.push_scope();
                TirStmt::While {
                    cond: checked_cond,
                    body: Box::new(checked_body),
                }
            }
            HirStmt::Continue => {
                if self.loop_depth > 0 {
                    TirStmt::Continue
                } else {
                    die!("Continue statements can only be used inside a loop body: {span}");
                }
            }
            HirStmt::Break => {
                if self.loop_depth > 0 {
                    TirStmt::Break
                } else {
                    die!("Break statements can only be used inside a loop body: {span}");
                }
            }
            HirStmt::If { cond, then_, else_ } => {
                let checked_cond = self.check_rvalue_expr(&cond, None);
                let cond_ty = checked_cond.ty;
                if *cond_ty != Type::Bool {
                    die!(
                        "Type mismatch. Expected `{}` but got {cond_ty}: {span}",
                        Type::Bool,
                    )
                }
                let checked_then = Box::new(self.check_stmt(*then_));
                let checked_else = Box::new(self.check_stmt(*else_));
                TirStmt::If {
                    cond: checked_cond,
                    then_: checked_then,
                    else_: checked_else,
                }
            }
            HirStmt::Return(val) => {
                let returns = self.return_type;

                let checked_val = self.check_rvalue_expr(&val, Some(returns));
                if checked_val.ty != returns {
                    die!(
                        "Mismatched return type. Function expects {returns} but got {}: {}",
                        checked_val.ty,
                        span.wrap(self.name.inner)
                    )
                }

                if *returns == Type::Void {
                    TirStmt::Return(None)
                } else {
                    TirStmt::Return(Some(checked_val))
                }
            }
            HirStmt::Block(s) => {
                self.env.push_scope();
                let stmt = TirStmt::Block(s.into_iter().map(|st| self.check_stmt(st)).collect());
                self.env.pop_scope();
                stmt
            }
            HirStmt::Expr(e) => {
                let e = self.check_rvalue_expr(&e, None);
                TirStmt::Expr(e)
            }
        }
    }

    // NOTE: This should probably return a pointer?
    fn check_lvalue_expr(
        &mut self,
        Spanned { inner: expr, span }: &Spanned<HirExpr>,
        hint: Option<TypeId>,
    ) -> TirExpr {
        match expr {
            // x = ...
            HirExpr::Ident(symbol) => {
                let Some(symbol) = self.env.get(symbol) else {
                    die!("Undefined variable: {}", span.wrap(symbol));
                };
                let SymbolInfo { ty, .. } = self.lookup_symbol(symbol);
                let ptr_ty = add_type(Type::Pointer(*ty));
                let kind = TirExprKind::AddrOf(symbol);
                TirExpr::new(kind, ptr_ty)
            }

            // x[i] = ...
            HirExpr::Index { base, index } => {
                let checked_base = self.check_rvalue_expr(base, hint);
                let checked_index = self.check_rvalue_expr(index, Some(add_type(Type::U64)));
                if !checked_base.ty.is_pointer() {
                    die!(
                        "Can't index into non-pointer type {}: {base}",
                        checked_base.ty
                    )
                }

                if !checked_index.ty.is_integral() {
                    die!(
                        "Can't index using non-integer type {}: {index}",
                        checked_index.ty
                    )
                }

                let ty = checked_base.ty;
                let kind = TirExprKind::Bin {
                    op: BinOp::PtrAdd,
                    lhs: Box::new(checked_base),
                    rhs: Box::new(checked_index),
                };
                TirExpr::new(kind, ty)
            }

            // *x = ...
            HirExpr::Deref { inner } => {
                let ptr = self.check_rvalue_expr(inner, hint);
                assert!(ptr.ty.is_pointer());
                ptr
            }
            _ => die!(
                "This expression not a valid LVALUE and therefore cannot be assigned to! {span}"
            ),
        }
    }

    fn check_rvalue_expr(
        &mut self,
        Spanned { inner: expr, span }: &Spanned<HirExpr>,
        hint: Option<TypeId>,
    ) -> TirExpr {
        match expr {
            HirExpr::Void => {
                let kind = TirExprKind::Void;
                TirExpr::new(kind, add_type(Type::Void))
            }
            HirExpr::Num(int_str) => {
                let ty = match hint {
                    Some(hint_id) => {
                        if hint_id.is_integral() || hint_id.is_pointer() {
                            hint_id
                        } else {
                            add_type(Type::I32)
                        }
                    }
                    None => add_type(Type::I32),
                };
                let result = match ty.lookup() {
                    Type::I8 => int_str.parse::<i8>().map(|i| i as i128),
                    Type::U8 => int_str.parse::<u8>().map(|i| i as i128),
                    Type::I16 => int_str.parse::<i16>().map(|i| i as i128),
                    Type::U16 => int_str.parse::<u16>().map(|i| i as i128),
                    Type::I32 => int_str.parse::<i32>().map(|i| i as i128),
                    Type::U32 => int_str.parse::<u32>().map(|i| i as i128),
                    Type::I64 => int_str.parse::<i64>().map(|i| i as i128),
                    Type::U64 => int_str.parse::<u64>().map(|i| i as i128),
                    Type::Pointer(..) if int_str.parse::<i32>().is_ok_and(|x| x == 0) => Ok(0),
                    _ => {
                        die!("`{int_str}` could not be parsed as a {ty}");
                    }
                };
                let Ok(num) = result else {
                    die!("`{int_str}` could not be parsed as a `{ty}`: {span}");
                };
                let kind = TirExprKind::Num(num);
                TirExpr::new(kind, ty)
            }
            HirExpr::Bool(b) => {
                let ty = add_type(Type::Bool);
                let kind = TirExprKind::Bool(*b);
                TirExpr::new(kind, ty)
            }
            HirExpr::Assign { lhs, rhs } => {
                // This should become a Store?
                // Check the LHS as an LVALUE. It must be a storage location
                let checked_lhs = self.check_lvalue_expr(lhs, hint);
                let lhs_ty = checked_lhs.ty.get_pointee();

                // RHS can be anything
                let checked_rhs = self.check_rvalue_expr(rhs, Some(lhs_ty));
                let rhs_ty = checked_rhs.ty;

                // Since LHS is a storage location, it's technically a pointer
                // so the RHS type should match whatever LHS is pointing to
                if lhs_ty != rhs_ty {
                    die!("Cannot assign a `{rhs_ty}` to `{lhs_ty}`: {span}")
                }

                // x = y = z should be possible
                let ty = checked_rhs.ty;
                let kind = TirExprKind::Store {
                    ptr: Box::new(checked_lhs),
                    val: Box::new(checked_rhs),
                };
                TirExpr::new(kind, ty)
            }
            HirExpr::Ident(symbol) => {
                let Some(symbol) = self.env.get(symbol) else {
                    die!("Undefined variable: {}", span.wrap(symbol));
                };
                let SymbolInfo { ty, .. } = self.lookup_symbol(symbol);
                let kind = TirExprKind::ValueOf(symbol);
                TirExpr::new(kind, *ty)
            }
            HirExpr::Index { base, index } => {
                // This is an rvalue, so it should always be a Load
                let inner = self.check_lvalue_expr(&span.wrap(expr.clone()), hint);
                let ty = inner.ty.get_pointee();
                let kind = TirExprKind::Load {
                    inner: Box::new(inner),
                };

                TirExpr::new(kind, ty)
            }
            HirExpr::Deref { inner } => {
                // This is an rvalue, so it should always be a Load
                let ptr = self.check_rvalue_expr(inner, hint);
                if !ptr.ty.is_pointer() {
                    die!("Cannot dereference non-pointer type {}", span.wrap(ptr));
                }
                let ty = ptr.ty.get_pointee();
                let kind = TirExprKind::Load {
                    inner: Box::new(ptr),
                };
                TirExpr::new(kind, ty)
            }
            HirExpr::Field { base, field } => {
                let mut base = self.check_rvalue_expr(base, None);
                let s = base.ty.lookup();
                let Type::Base { name, fields } = s else {
                    die!("Type {s} is not a struct type. {span}");
                };
                let mut offset = 0;
                let mut maybe_field_ty = None;
                for (field_name, field_ty) in fields.iter() {
                    if field_name == field {
                        maybe_field_ty = Some(*field_ty);
                        break;
                    }
                    // TODO: this can't be bytes, this is pointer math so it needs to be scaled down
                    // by field size
                    offset += field_ty.bytes();
                }

                let Some(field_ty) = maybe_field_ty else {
                    die!("Struct {s} has no field {field}");
                };
                base.ty = add_type(Type::Pointer(field_ty));
                let ptr = {
                    let num = {
                        let kind = TirExprKind::Num(offset as i128);
                        let ty = add_type(Type::U64);
                        TirExpr::new(kind, ty)
                    };
                    let ty = add_type(Type::Pointer(field_ty));
                    let kind = TirExprKind::Bin {
                        op: BinOp::PtrAdd,
                        lhs: Box::new(base),
                        rhs: Box::new(num),
                    };
                    TirExpr::new(kind, ty)
                };
                let kind = TirExprKind::Load {
                    inner: Box::new(ptr),
                };

                TirExpr::new(kind, field_ty)
            }
            // I hope this is right...
            HirExpr::AddrOf { inner } => self.check_lvalue_expr(inner, hint),
            HirExpr::Cast { target_ty, rhs } => {
                // TODO: enforce type casting rules
                // Casting should be valid between:
                // - Same sized types (this means all pointers can be cast to and from each other)
                // - Any primitive with any other primitive
                let checked_ty = resolve_type(target_ty);
                let checked_rhs = Box::new(self.check_rvalue_expr(rhs, Some(checked_ty)));
                let kind = TirExprKind::Cast {
                    target_ty: checked_ty,
                    expr: checked_rhs,
                };
                TirExpr::new(kind, checked_ty)
            }
            HirExpr::Un { op, rhs } => {
                let checked_rhs = self.check_rvalue_expr(rhs, hint);
                let rhs_ty = checked_rhs.ty;
                let ty = match op {
                    UnOp::Not => {
                        if *rhs_ty != Type::Bool {
                            die!("Cannot logical not a {}", span.wrap(rhs_ty))
                        }
                        rhs_ty
                    }
                    UnOp::Neg => {
                        if !rhs_ty.is_signed() {
                            die!("Cannot negate a {}", span.wrap(rhs_ty))
                        }
                        rhs_ty
                    }
                };
                let kind = TirExprKind::Un {
                    op: *op,
                    rhs: Box::new(checked_rhs),
                };
                TirExpr::new(kind, ty)
            }
            HirExpr::Bin { op, lhs, rhs } => {
                let lhs = self.check_rvalue_expr(lhs, hint);
                let lhs_class = classify_type(lhs.ty.lookup());
                let rhs = self.check_rvalue_expr(rhs, Some(lhs.ty));
                let rhs_class = classify_type(rhs.ty.lookup());

                let (op, ty) = match (lhs_class, op, rhs_class) {
                    // ptr - ptr = int
                    (BinType::Pointer, BinOp::Sub, BinType::Pointer) if lhs.ty == rhs.ty => {
                        (BinOp::Sub, lhs.ty)
                    }
                    // ptr + int = ptr
                    (BinType::Pointer, BinOp::Add, BinType::Integral) => (BinOp::PtrAdd, lhs.ty),
                    // ptr - int = ptr
                    (BinType::Pointer, BinOp::Sub, BinType::Integral) => (BinOp::PtrSub, lhs.ty),
                    // int + ptr = ptr
                    (BinType::Integral, BinOp::Add, BinType::Pointer) => (BinOp::PtrAdd, rhs.ty),
                    // int +,-,/,* int = int
                    (BinType::Integral, op, BinType::Integral)
                        if lhs.ty == rhs.ty && op.is_arithmetic() =>
                    {
                        (*op, lhs.ty)
                    }
                    // int >,>=,<,<= int = bool, ptr >,>=,<,<= ptr = bool
                    (
                        BinType::Integral | BinType::Pointer,
                        op,
                        BinType::Integral | BinType::Pointer,
                    ) if lhs.ty == rhs.ty && op.is_ordered() => (*op, add_type(Type::Bool)),
                    // any == any = bool, any != any = bool
                    (_, BinOp::Eq | BinOp::Ne, _) if lhs.ty == rhs.ty => {
                        (*op, add_type(Type::Bool))
                    }
                    _ => die!(
                        "Unsupported operation `{op}` between {} and {}",
                        lhs.ty,
                        rhs.ty
                    ),
                };
                let kind = TirExprKind::Bin {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                TirExpr::new(kind, ty)
            }
            HirExpr::Call { callee, args } => {
                // TODO: we can't check if the user is passing in the right types of arguments until
                // we add the callee to the global symbol table
                let callee = Box::new(self.check_rvalue_expr(callee, hint));
                let Type::Function { returns, .. } = *callee.ty else {
                    die!("Function callee does not resolve to a function type: {span}");
                };

                let args = args
                    .iter()
                    .map(|a| self.check_rvalue_expr(a, None))
                    .collect();
                let kind = TirExprKind::Call { callee, args };
                TirExpr::new(kind, returns)
            }
            HirExpr::SizeOfTy { ty } => {
                let ty_size = resolve_type(ty).bytes();
                let kind = TirExprKind::Num(ty_size as i128);
                let ty = add_type(Type::U64);
                TirExpr::new(kind, ty)
            }
            HirExpr::SizeOfExpr { expr } => {
                let ty_size = self.check_rvalue_expr(expr, None).ty.bytes();
                let kind = TirExprKind::Num(ty_size as i128);
                let ty = add_type(Type::U64);
                TirExpr::new(kind, ty)
            }
        }
    }

    fn scale_ptr_int_math(&mut self, ptr: TirExpr, op: BinOp, int: TirExpr) -> TirExpr {
        let ptr_ty = ptr.ty;
        let scaled_int = {
            let scale = {
                let pointee_size = ptr.ty.get_pointee().bytes();
                let kind = TirExprKind::Num(pointee_size as i128);
                TirExpr::new(kind, add_type(Type::U64))
            };
            let kind = TirExprKind::Bin {
                op: BinOp::Mul,
                lhs: Box::new(int),
                rhs: Box::new(scale),
            };
            TirExpr::new(kind, add_type(Type::U64))
        };
        let kind = TirExprKind::Bin {
            op,
            lhs: Box::new(ptr),
            rhs: Box::new(scaled_int),
        };
        TirExpr::new(kind, ptr_ty)
    }
}

enum BinType {
    Integral,
    Pointer,
    Other,
}

/// Helper to avoid making calls to `ty.is_pointer()` or `ty.is_integral()`
fn classify_type(ty: &Type) -> BinType {
    if ty.is_pointer() {
        BinType::Pointer
    } else if ty.is_integral() {
        BinType::Integral
    } else {
        BinType::Other
    }
}
