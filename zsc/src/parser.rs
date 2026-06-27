use crate::aux::Compiler;
use crate::{add_str, ast::*};
use crate::{hir::*, source};

use crate::prelude::*;

// Parser implementation
impl Compiler {
    pub fn parse_obj(&mut self) -> Spanned<HirObj> {
        let tok = self.peek();
        match tok.inner {
            Token::Fn => self.parse_func(),
            Token::Global => self.parse_global(),
            Token::Struct => self.parse_struct(),
            _ => die!("Expected `global`, `fn`, or `struct`, but got {tok}"),
        }
    }

    fn parse_struct(&mut self) -> Spanned<HirObj> {
        todo!()
    }

    fn parse_global(&mut self) -> Spanned<HirObj> {
        todo!()
    }

    fn parse_func(&mut self) -> Spanned<HirObj> {
        let span_start = self.mark();
        self.expect(Token::Fn);
        let name = {
            let span_start = self.mark();
            let name = self.expect_ident();
            self.commit(name, span_start)
        };

        self.expect(Token::LParen);
        let mut args = vec![];
        while self.peek().inner != Token::RParen {
            if !args.is_empty() {
                self.expect(Token::Comma);
                if self.is_next(Token::RParen) {
                    break;
                }
            }
            let argname = {
                let span_start = self.mark();
                let inner = self.expect_ident();
                self.commit(inner, span_start)
            };
            self.expect(Token::Colon);
            let ty = self.parse_type();
            args.push((argname, ty));
        }
        self.expect(Token::RParen);

        let peeked = self.peek();
        let returns = match peeked.inner {
            Token::RArrow => {
                self.expect(Token::RArrow);
                self.parse_type()
            }
            Token::LCurly => {
                let span_start = self.mark();
                let ty = RawType::Named(add_str("void"));
                self.commit(ty, span_start)
            }
            _ => die!("Expected return type or function body, found {peeked}"),
        };

        let body = Box::new(self.parse_block());

        let kind = HirObjKind::Fn {
            name,
            returns,
            args,
            body,
        };
        let obj = HirObj::new(kind, ());
        self.commit(obj, span_start)
    }

    fn parse_type(&mut self) -> Spanned<RawType> {
        let span_start = self.mark();
        let tok = self.peek();
        let ty = match tok.inner {
            Token::Star => {
                self.expect(Token::Star);
                RawType::Pointer(Box::new(self.parse_type().inner))
            }
            _ => RawType::Named(self.expect_ident()),
        };

        self.commit(ty, span_start)
    }

    fn parse_stmt(&mut self) -> Spanned<HirStmt> {
        let span_start = self.mark();
        let tok = self.peek();
        let stmt = match tok.inner {
            Token::Let => {
                self.eat();
                let sym_start = self.mark();
                let varname = self.expect_ident();
                let ty = self.is_next(Token::Colon).then(|| {
                    self.expect(Token::Colon);
                    self.parse_type()
                });
                self.expect(Token::Eq);
                let rhs = self.parse_expr();
                self.expect(Token::Semi);
                let kind = HirStmtKind::Let {
                    lhs: self.commit(varname, sym_start),
                    ty,
                    rhs,
                };
                HirStmt::new(kind, ())
            }
            Token::If => {
                self.eat();
                let cond = self.parse_expr();
                let then_ = self.parse_block();
                let else_ = if self.is_next(Token::Else) {
                    self.eat();
                    if self.is_next(Token::If) {
                        self.parse_stmt()
                    } else {
                        self.parse_block()
                    }
                } else {
                    let kind = HirStmtKind::Block(vec![]);
                    Spanned::new(HirStmt::new(kind, ()), Span::default())
                };

                let kind = HirStmtKind::If {
                    cond,
                    then_: Box::new(then_),
                    else_: Box::new(else_),
                };
                HirStmt::new(kind, ())
            }
            Token::Break => {
                self.eat();
                self.expect(Token::Semi);
                let kind = HirStmtKind::Break;
                HirStmt::new(kind, ())
            }
            Token::Continue => {
                self.eat();
                self.expect(Token::Semi);
                let kind = HirStmtKind::Continue;
                HirStmt::new(kind, ())
            }
            Token::Return => {
                self.eat();
                let ret_val = if self.is_next(Token::Semi) {
                    tok.map(|_| HirExpr::new(HirExprKind::Void, ()))
                } else {
                    self.parse_expr()
                };
                let kind = HirStmtKind::Return(ret_val);
                self.expect(Token::Semi);
                HirStmt::new(kind, ())
            }
            Token::While => {
                self.eat();
                let cond = self.parse_expr();
                let body = self.parse_block();
                let kind = HirStmtKind::While {
                    cond,
                    body: Box::new(body),
                };
                HirStmt::new(kind, ())
            }
            _ => {
                let expr = self.parse_expr();
                self.expect(Token::Semi);
                let kind = HirStmtKind::Expr(expr);
                HirStmt::new(kind, ())
            }
        };
        self.commit(stmt, span_start)
    }

    fn parse_block(&mut self) -> Spanned<HirStmt> {
        let span_start = self.mark();
        self.expect(Token::LCurly);
        let mut stmts = vec![];
        while !self.is_next(Token::RCurly) {
            let stmt = self.parse_stmt();
            if matches!(stmt.inner.kind, HirStmtKind::Block(ref inner) if inner.is_empty()) {
                // Skip nested empty {} blocks, they're useless.
                continue;
            }
            stmts.push(stmt);
        }
        self.expect(Token::RCurly);
        let kind = HirStmtKind::Block(stmts);
        self.commit(HirStmt::new(kind, ()), span_start)
    }

    /// Start a span. This does not change state, but simply returns
    /// the span of the next token that will be consumed.
    /// Use this before you start parsing a "thing", i.e...
    ///
    /// let start = self.mark();
    /// self.expect(Token::Fn);
    /// self.expect(...);
    /// self.commit(..., start)
    fn mark(&mut self) -> Span {
        self.peek().span
    }

    /// Creates a new Spanned "thing", given a starting point
    fn commit<T>(&mut self, inner: T, start: Span) -> Spanned<T> {
        Spanned::new(inner, start.merge(self.last_span))
    }

    fn parse_prefix(&mut self) -> Spanned<HirExpr> {
        let span_start = self.mark();
        let tok = self.eat();
        let typed_expr = match tok.inner {
            Token::Sizeof => {
                self.expect(Token::LParen);
                let ty = self.parse_type();
                self.expect(Token::RParen);
                let kind = HirExprKind::SizeOfTy { ty };
                HirExpr::new(kind, ())
            }
            Token::Ident(s) => {
                let kind = HirExprKind::Ident(s);
                HirExpr::new(kind, ())
            }
            Token::Minus => {
                let rhs = Box::new(self.parse_expr());
                let kind = HirExprKind::Un { op: UnOp::Neg, rhs };
                HirExpr::new(kind, ())
            }
            Token::Bool(x) => {
                let kind = HirExprKind::Bool(x);
                HirExpr::new(kind, ())
            }
            Token::Int(x) => {
                let kind = HirExprKind::Num(x);
                HirExpr::new(kind, ())
            }
            Token::LParen => {
                let inner_expr = self.parse_expr();
                self.expect(Token::RParen);
                inner_expr.inner
            }
            Token::Star => {
                let power = prefix_power(Token::Star).unwrap();
                let rhs = Box::new(self._parse_expr(power));
                let kind = HirExprKind::Deref { rhs };
                HirExpr::new(kind, ())
            }
            Token::And => {
                let power = prefix_power(Token::Star).unwrap();
                let rhs = Box::new(self._parse_expr(power));
                let kind = HirExprKind::AddrOf { rhs };
                HirExpr::new(kind, ())
            }
            Token::At => {
                let power = prefix_power(Token::At).unwrap();
                self.expect(Token::LParen);
                let target_ty = self.parse_type();
                self.expect(Token::RParen);
                let rhs = Box::new(self._parse_expr(power));
                let kind = HirExprKind::Cast { target_ty, rhs };
                HirExpr::new(kind, ())
            }
            _ => die!("Expected start of expression, found {tok}"),
        };
        self.commit(typed_expr, span_start.merge(self.last_span))
    }

    fn parse_infix(
        &mut self,
        lhs: Spanned<HirExpr>,
        op: Spanned<Token>,
        op_power: f32,
    ) -> Spanned<HirExpr> {
        let span_start = lhs.span;
        let output = match op.inner {
            Token::LBrack => {
                let idx = self.parse_expr();
                self.expect(Token::RBrack);
                let kind = HirExprKind::Index {
                    expr: Box::new(lhs),
                    index: Box::new(idx),
                };
                HirExpr::new(kind, ())
            }
            Token::LParen => {
                let mut args = vec![];
                while !self.is_next(Token::RParen) {
                    if !args.is_empty() {
                        self.expect(Token::Comma);
                    }
                    args.push(self.parse_expr());
                }
                self.expect(Token::RParen);
                let kind = HirExprKind::Call {
                    callee: Box::new(lhs),
                    args,
                };
                HirExpr::new(kind, ())
            }
            Token::Eq => {
                let rhs = self._parse_expr(op_power);
                let kind = HirExprKind::Assign {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                HirExpr::new(kind, ())
            }
            Token::LBrack => {
                let index = self.parse_expr();
                todo!()
            }
            arith @ (Token::Plus | Token::Minus | Token::Star | Token::Slash) => {
                let rhs = self._parse_expr(op_power);
                let op = match arith {
                    Token::Plus => BinOp::Add,
                    Token::Minus => BinOp::Sub,
                    Token::Star => BinOp::Mul,
                    Token::Slash => BinOp::Div,
                    _ => unreachable!(),
                };

                let kind = HirExprKind::Bin {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                HirExpr::new(kind, ())
            }
            rel @ (Token::EqEq | Token::LtEq | Token::Lt | Token::GtEq | Token::Gt) => {
                let rhs = self._parse_expr(op_power);
                let op = match rel {
                    Token::EqEq => BinOp::Eq,
                    Token::LtEq => BinOp::Le,
                    Token::Lt => BinOp::Lt,
                    Token::GtEq => BinOp::Ge,
                    Token::Gt => BinOp::Gt,
                    _ => unreachable!(),
                };

                let kind = HirExprKind::Bin {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                HirExpr::new(kind, ())
            }
            _ => die!("Expected infix operator, found {op}"),
        };
        self.commit(output, span_start)
    }

    // Pratt parsing!
    fn _parse_expr(&mut self, min_power: f32) -> Spanned<HirExpr> {
        let mut lhs = self.parse_prefix();

        loop {
            let op = self.peek();

            let Some(op_power) = infix_power(op.inner) else {
                break;
            };

            if op_power.0 < min_power {
                break;
            }

            self.eat();

            lhs = self.parse_infix(lhs, op, op_power.1);
        }

        lhs
    }

    fn parse_expr(&mut self) -> Spanned<HirExpr> {
        self._parse_expr(0.0)
    }
}

fn prefix_power(kind: Token) -> Option<f32> {
    let power = match kind {
        Token::Sizeof | Token::Bang | Token::Minus | Token::Star | Token::And | Token::At => 9.0,
        _ => return None, // Not an prefix operator
    };
    Some(power)
}

fn infix_power(kind: Token) -> Option<(f32, f32)> {
    let power = match kind {
        // Assignment: Right-Associative
        // We use a lower Left power so it "gives up" easily,
        // but a higher Right power to "grab" everything to the right.
        Token::Eq => (2.1, 2.0),

        Token::AndAnd | Token::OrOr => (4.1, 4.0),

        Token::EqEq | Token::BangEq => (5.0, 5.1),
        Token::Lt | Token::Gt => (6.0, 6.1),
        Token::LtEq | Token::GtEq => (6.0, 6.1),

        Token::Plus | Token::Minus => (7.0, 7.1),

        Token::Star | Token::Slash => (8.0, 8.1),

        // Postfix: Highest priority
        // (Call, Indexing, Member Access)
        Token::LParen | Token::LBrack | Token::Dot => (10.0, 10.1),

        _ => return None, // Not an infix operator
    };
    Some(power)
}

// Lexer implementation
impl Compiler {
    fn peek_byte(&mut self) -> Option<u8> {
        source().get(self.cursor).copied()
    }

    fn eat_byte(&mut self) -> Option<u8> {
        self.cursor += 1;
        match source().get(self.cursor - 1).copied() {
            Some(c) => {
                if c == b'\n' {
                    self.row += 1;
                    self.col = 0;
                } else {
                    self.col += 1;
                }
                Some(c)
            }
            None => None,
        }
    }

    fn make_token(&mut self, kind: Token, lo: usize) -> Spanned<Token> {
        Spanned::new(kind, Span::new(lo, self.cursor, self.row, self.col))
    }

    fn read_num(&mut self) -> Option<Spanned<Token>> {
        let start = self.cursor;
        let mut buf = vec![];

        // Make sure the number starts with a digit
        let first = self.peek_byte()?;
        if !first.is_ascii_digit() {
            return None;
        }

        // Now read all digits and underscores
        while let Some(c) = self.peek_byte()
            && b"0123456789_".contains(&c)
        {
            let c = self.eat_byte()?;
            if c != b'_' {
                buf.push(c);
            }
        }

        let Ok(raw) = str::from_utf8(&buf) else {
            die!(
                "Non-utf8 characters are not supported: `{}` found near: {}",
                source()[0],
                self.last_span
            );
        };

        let kind = Token::Int(add_str(raw));

        Some(self.make_token(kind, start))
    }

    // This returns either an Ident or a Keyword, depending on what the string equates to
    fn read_word(&mut self) -> Option<Spanned<Token>> {
        let start = self.cursor;
        // Identifiers can only start with letters or underscores
        let first = self.peek_byte()?;
        if !(first.is_ascii_alphabetic() || first == b'_') {
            return None;
        }

        while let Some(c) = self.peek_byte()
            && (c.is_ascii_alphanumeric() || c == b'_')
        {
            self.eat_byte()?;
        }

        let Ok(raw) = str::from_utf8(source().get(start..self.cursor)?) else {
            die!(
                "Non-utf8 characters are not supported: `{}` found near: {}",
                source()[0],
                self.last_span
            );
        };

        let kind = match raw {
            "let" => Token::Let,
            "fn" => Token::Fn,
            "struct" => Token::Struct,
            "global" => Token::Global,
            "while" => Token::While,
            "continue" => Token::Continue,
            "break" => Token::Break,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "sizeof" => Token::Sizeof,
            _ => Token::Ident(add_str(raw)),
        };

        Some(self.make_token(kind, start))
    }

    fn read_strlit(&mut self) -> Option<Spanned<Token>> {
        let start = self.cursor;
        if self.peek_byte()? == b'"' {
            self.eat_byte();
        } else {
            return None;
        }

        loop {
            let curr = self.eat_byte().expect("LEXER: Unclosed quote");
            match curr {
                b'"' => break,
                b'\\' => {
                    self.eat_byte().expect("LEXER: Unclosed quote");
                }
                _ => {}
            }
        }

        // +1/-1 to disclude the surrounding "..."
        let Ok(raw) = str::from_utf8(source().get(start + 1..self.cursor - 1)?) else {
            die!(
                "Non-utf8 characters are not supported: `{}` found near: {}",
                source()[0],
                self.last_span
            );
        };
        let token = Token::Str(add_str(raw));

        Some(self.make_token(token, start))
    }

    fn read_punct(&mut self) -> Option<Spanned<Token>> {
        let start = self.cursor;
        let known_punctuators = &["==", "!=", "<=", ">=", "->", "&&", "||", "<<", ">>"];

        let mut length = 1;
        let src = source().get(start..)?;
        for p in known_punctuators {
            if src.starts_with(p.as_bytes()) {
                length = p.len();
                break;
            }
        }

        let first = self.peek_byte()?;

        if length == 1 && !first.is_ascii_punctuation() || first == b'_' {
            return None;
        }

        let s = (0..length)
            .filter_map(|_| self.eat_byte())
            .collect::<Vec<_>>();

        use Token::*;
        let kind = match s.as_slice() {
            // Delimiters
            b"(" => LParen,
            b")" => RParen,
            b"{" => LCurly,
            b"}" => RCurly,
            b"[" => LBrack,
            b"]" => RBrack,
            // Separators
            b"," => Comma,
            b"." => Dot,
            b":" => Colon,
            b";" => Semi,
            b"->" => RArrow,
            // Operators
            b"+" => Plus,    // +
            b"-" => Minus,   // -
            b"*" => Star,    // *
            b"/" => Slash,   // /
            b"%" => Percent, // %
            b"&" => And,     // &
            b"|" => Or,      // |
            b"^" => Caret,   // ^
            b"!" => Bang,    // !
            b"=" => Eq,      // =
            b"&&" => AndAnd,
            b"||" => OrOr,
            // Relationals
            b"==" => EqEq,
            b"!=" => BangEq,
            b"<" => Lt,
            b">" => Gt,
            b"<=" => LtEq,
            b">=" => GtEq,
            b"@" => At,
            x => die!(
                "Unknown token: `{}` found near: {}",
                str::from_utf8(x).unwrap(),
                self.last_span
            ),
        };

        Some(self.make_token(kind, start))
    }

    fn read_whitespace(&mut self) -> Option<Spanned<Token>> {
        while let Some(c) = self.peek_byte()
            && c.is_ascii_whitespace()
        {
            self.eat_byte()?;
        }
        None
    }

    pub fn eat(&mut self) -> Spanned<Token> {
        let tok = self
            .read_whitespace()
            .or_else(|| self.read_word())
            .or_else(|| self.read_num())
            .or_else(|| self.read_strlit())
            .or_else(|| self.read_punct())
            .unwrap_or_default();
        self.last_span = tok.span;
        tok
    }

    pub fn peek(&mut self) -> Spanned<Token> {
        // Screenshot state
        let tmp = (self.cursor, self.row, self.col);
        // Eat a token
        let tok = self.eat();
        // Restore state
        (self.cursor, self.row, self.col) = tmp;
        // Return the peeked token
        tok
    }

    pub fn expect(&mut self, expected: Token) {
        let next = self.eat();
        if next.inner != expected {
            next.span.content();
            die!("Expected {expected:?}, found {next}");
        }
    }

    pub fn is_next(&mut self, expected: Token) -> bool {
        self.peek().inner == expected
    }

    pub fn expect_ident(&mut self) -> &'static str {
        let next = self.eat();
        match next.inner {
            Token::Ident(s) => s,
            _ => die!("Expected identifier, found {next}"),
        }
    }
}
