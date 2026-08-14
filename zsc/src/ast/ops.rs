#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,

    Ne,
    Eq,

    Le,
    Lt,
    Ge,
    Gt,
}

impl BinOp {
    pub fn is_arithmetic(&self) -> bool {
        matches!(self, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div)
    }

    pub fn is_ptr_arithmetic(&self) -> bool {
        matches!(self, BinOp::Add | BinOp::Sub)
    }

    pub fn is_logical(&self) -> bool {
        matches!(
            self,
            BinOp::Eq | BinOp::Le | BinOp::Lt | BinOp::Ge | BinOp::Gt
        )
    }

    pub fn is_ordered(&self) -> bool {
        matches!(self, BinOp::Le | BinOp::Lt | BinOp::Ge | BinOp::Gt)
    }
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Ne => "!=",
            BinOp::Eq => "==",
            BinOp::Le => "<=",
            BinOp::Lt => "<",
            BinOp::Ge => ">=",
            BinOp::Gt => ">",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Not,
    Neg,
}

impl std::fmt::Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[rustfmt::skip]
        let s = match self {
            UnOp::Not => "!",
            UnOp::Neg => "-",
        };
        f.write_str(s)
    }
}
