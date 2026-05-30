#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,

    Eq,

    Le,
    Lt,
    Ge,
    Gt,
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[rustfmt::skip]
        let s = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Eq  => "==",
            BinOp::Le  => "<=",
            BinOp::Lt  => "<",
            BinOp::Ge  => ">=",
            BinOp::Gt  => ">",
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
