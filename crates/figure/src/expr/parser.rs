//! 式の構文解析．再帰下降法で，構文木にする．
//!
//! ```text
//! expr   = term { ("+" | "-") term }
//! term   = unary { ("*" | "/") unary }
//! unary  = ("-" | "+") unary | power
//! power  = atom [ "^" unary ]          (右に結ぶ．-x^2は-(x^2)，x^-2も読める)
//! atom   = number | name | name "(" expr ")" | "(" expr ")"
//! ```

use super::lexer::{Token, TokenKind};
use super::{BinaryOp, ExprError, ExprErrorKind, Function, Node, Span};

/// 括弧や単項の記号の入れ子の上限．
const MAX_DEPTH: usize = 128;

/// 定数の名前と値．
const CONSTANTS: [(&str, f64); 2] = [("pi", std::f64::consts::PI), ("e", std::f64::consts::E)];

/// 定数の名前か．
pub fn is_constant(name: &str) -> bool {
    CONSTANTS.iter().any(|(constant, _)| *constant == name)
}

/// 字句の列を，構文木にする．`names`にある名前は，その位置の変数になる．
///
/// # Errors
///
/// 構文の誤り，使えない名前，入れ子の深すぎる式があると，誤りを返す．
pub fn parse(tokens: &[Token], names: &[&str]) -> Result<Node, ExprError> {
    let mut parser = Parser {
        tokens,
        position: 0,
        names,
        depth: 0,
    };
    let node = parser.expression()?;
    let token = parser.peek()?;
    if token.kind == TokenKind::End {
        Ok(node)
    } else {
        Err(unexpected(token))
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
    names: &'a [&'a str],
    depth: usize,
}

fn unexpected(token: &Token) -> ExprError {
    let kind = if token.kind == TokenKind::End {
        ExprErrorKind::UnexpectedEnd
    } else {
        ExprErrorKind::UnexpectedToken(token.text.clone())
    };
    ExprError {
        kind,
        span: token.span.clone(),
    }
}

fn error_at(kind: ExprErrorKind, span: Span) -> ExprError {
    ExprError { kind, span }
}

impl Parser<'_> {
    fn peek(&self) -> Result<&Token, ExprError> {
        // 字句の列は，式の終わりで終わる．その先を読むことはない．
        self.tokens
            .get(self.position)
            .or_else(|| self.tokens.last())
            .ok_or_else(|| error_at(ExprErrorKind::UnexpectedEnd, 0..0))
    }

    fn advance(&mut self) {
        self.position = self.position.saturating_add(1);
    }

    fn enter(&mut self, span: &Span) -> Result<(), ExprError> {
        self.depth = self.depth.saturating_add(1);
        if self.depth > MAX_DEPTH {
            Err(error_at(ExprErrorKind::TooDeep, span.clone()))
        } else {
            Ok(())
        }
    }

    fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    fn expression(&mut self) -> Result<Node, ExprError> {
        let mut left = self.term()?;
        loop {
            let op = match self.peek()?.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => return Ok(left),
            };
            self.advance();
            let right = self.term()?;
            left = Node::Binary(op, Box::new(left), Box::new(right));
        }
    }

    fn term(&mut self) -> Result<Node, ExprError> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek()?.kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                _ => return Ok(left),
            };
            self.advance();
            let right = self.unary()?;
            left = Node::Binary(op, Box::new(left), Box::new(right));
        }
    }

    fn unary(&mut self) -> Result<Node, ExprError> {
        let token = self.peek()?;
        let (negate, span) = match token.kind {
            TokenKind::Minus => (true, token.span.clone()),
            TokenKind::Plus => (false, token.span.clone()),
            _ => return self.power(),
        };
        self.advance();
        self.enter(&span)?;
        let operand = self.unary();
        self.leave();
        let operand = operand?;
        Ok(if negate {
            Node::Neg(Box::new(operand))
        } else {
            operand
        })
    }

    fn power(&mut self) -> Result<Node, ExprError> {
        let base = self.atom()?;
        if self.peek()?.kind == TokenKind::Caret {
            let span = self.peek()?.span.clone();
            self.advance();
            self.enter(&span)?;
            let exponent = self.unary();
            self.leave();
            Ok(Node::Binary(
                BinaryOp::Pow,
                Box::new(base),
                Box::new(exponent?),
            ))
        } else {
            Ok(base)
        }
    }

    fn atom(&mut self) -> Result<Node, ExprError> {
        let token = self.peek()?.clone();
        match &token.kind {
            TokenKind::Number(value) => {
                self.advance();
                Ok(Node::Number(*value))
            }
            TokenKind::Name(name) => {
                self.advance();
                self.name(name, &token.span)
            }
            TokenKind::OpenParen => {
                self.advance();
                self.enter(&token.span)?;
                let inner = self.expression();
                self.leave();
                let inner = inner?;
                self.expect_close()?;
                Ok(inner)
            }
            _ => Err(unexpected(&token)),
        }
    }

    fn expect_close(&mut self) -> Result<(), ExprError> {
        let token = self.peek()?;
        match token.kind {
            TokenKind::CloseParen => {
                self.advance();
                Ok(())
            }
            TokenKind::End => Err(error_at(
                ExprErrorKind::MissingClosingParenthesis,
                token.span.clone(),
            )),
            _ => Err(unexpected(token)),
        }
    }

    /// 名前が，関数の呼び出し，変数，定数のどれかを決める．
    fn name(&mut self, name: &str, span: &Span) -> Result<Node, ExprError> {
        if self.peek()?.kind == TokenKind::OpenParen {
            let function = Function::from_name(name).ok_or_else(|| {
                error_at(
                    ExprErrorKind::UnknownFunction(name.to_owned()),
                    span.clone(),
                )
            })?;
            self.advance();
            self.enter(span)?;
            let argument = self.expression();
            self.leave();
            let argument = argument?;
            self.expect_close()?;
            return Ok(Node::Call(function, Box::new(argument)));
        }
        if let Some(index) = self.names.iter().position(|candidate| *candidate == name) {
            return Ok(Node::Variable(index));
        }
        if let Some((_, value)) = CONSTANTS.iter().find(|(constant, _)| *constant == name) {
            return Ok(Node::Number(*value));
        }
        let kind = if Function::from_name(name).is_some() {
            ExprErrorKind::FunctionNeedsArgument(name.to_owned())
        } else {
            ExprErrorKind::UnknownName(name.to_owned())
        };
        Err(error_at(kind, span.clone()))
    }
}
