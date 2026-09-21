//! 構文解析．記号の列を，構文木にする．再帰下降法で，演算子の優先順位は，次の文法で表す．
//!
//! ```text
//! 式     = 項 { ("+" | "-") 項 }
//! 項     = 単項 { ("*" | "/") 単項 | 省略された掛け算 の累乗 }
//! 単項   = ("-" | "+") 単項 | 累乗
//! 累乗   = 原子 [ "^" 単項 ]
//! 原子   = 数 | 変数 | "(" 式 ")"
//! ```
//!
//! 累乗は右結合で，単項の符号より強く結ぶ．したがって，`-x^2`は`-(x^2)`，`2^3^2`は`2^(3^2)`である．
//! 掛け算の記号は，次に変数か開き括弧が続くときに限って，省略できる(`2x`，`x(x+1)`)．数が続くときは省略できない．

use crate::ast::{BinaryOperator, Expr, ExprKind};
use crate::error::{Error, ErrorKind};
use crate::lexer::{Token, TokenKind, tokenize};

/// 入力の最大の長さ(文字数)．
pub const MAX_INPUT_CHARS: usize = 2_000;

/// 括弧や符号の，入れ子の最大の深さ．
const MAX_DEPTH: usize = 128;

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    depth: usize,
    /// 入力の文字数．式が途中で終わったときの，誤りの位置に使う．
    length: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|token| &token.kind)
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned();
        self.position = self.position.saturating_add(1);
        token
    }

    /// 入れ子の深さを数えながら，`parse`を実行する．深すぎるときは，誤りにする．
    fn nested<T>(&mut self, parse: impl FnOnce(&mut Self) -> Result<T, Error>) -> Result<T, Error> {
        self.depth = self.depth.saturating_add(1);
        let result = if self.depth > MAX_DEPTH {
            let span = self
                .peek()
                .map_or(self.length..self.length, |token| token.span.clone());
            Err(Error::new(ErrorKind::TooDeep, span))
        } else {
            parse(self)
        };
        self.depth = self.depth.saturating_sub(1);
        result
    }

    fn unexpected(&self, token: Option<&Token>) -> Error {
        token.map_or_else(
            || Error::new(ErrorKind::UnexpectedEnd, self.length..self.length),
            |token| {
                Error::new(
                    ErrorKind::UnexpectedToken(token.text.clone()),
                    token.span.clone(),
                )
            },
        )
    }

    fn parse_expression(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_term()?;
        while let Some(operator) = match self.peek_kind() {
            Some(TokenKind::Plus) => Some(BinaryOperator::Add),
            Some(TokenKind::Minus) => Some(BinaryOperator::Subtract),
            _ => None,
        } {
            self.advance();
            let right = self.parse_term()?;
            left = Expr::binary(operator, left, right);
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_unary()?;
        loop {
            let (operator, right) = match self.peek_kind() {
                Some(TokenKind::Star) => {
                    self.advance();
                    (BinaryOperator::Multiply, self.parse_unary()?)
                }
                Some(TokenKind::Slash) => {
                    self.advance();
                    (BinaryOperator::Divide, self.parse_unary()?)
                }
                // 省略された掛け算．右の因子は，累乗までを含める(`2x^2`は`2*(x^2)`)．
                Some(TokenKind::Letter(_) | TokenKind::LeftParen) => {
                    (BinaryOperator::Multiply, self.parse_power()?)
                }
                _ => return Ok(left),
            };
            left = Expr::binary(operator, left, right);
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, Error> {
        self.nested(|parser| match parser.peek_kind() {
            Some(TokenKind::Minus) => {
                let minus = parser.advance();
                let start = minus.map_or(parser.length, |token| token.span.start);
                let operand = parser.parse_unary()?;
                let span = start..operand.span.end;
                Ok(Expr {
                    kind: ExprKind::Negate(Box::new(operand)),
                    span,
                })
            }
            Some(TokenKind::Plus) => {
                parser.advance();
                parser.parse_unary()
            }
            _ => parser.parse_power(),
        })
    }

    fn parse_power(&mut self) -> Result<Expr, Error> {
        let base = self.parse_atom()?;
        if self.peek_kind() == Some(&TokenKind::Caret) {
            self.advance();
            let exponent = self.parse_unary()?;
            Ok(Expr::binary(BinaryOperator::Power, base, exponent))
        } else {
            Ok(base)
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, Error> {
        let Some(token) = self.advance() else {
            return Err(self.unexpected(None));
        };
        match token.kind {
            TokenKind::Number(value) => Ok(Expr {
                kind: ExprKind::Number(value),
                span: token.span,
            }),
            TokenKind::Letter(name) => Ok(Expr {
                kind: ExprKind::Variable(name),
                span: token.span,
            }),
            TokenKind::LeftParen => {
                let inner = self.nested(Self::parse_expression)?;
                match self.advance() {
                    Some(Token {
                        kind: TokenKind::RightParen,
                        span,
                        ..
                    }) => {
                        // 括弧を含む範囲にする．括弧そのものは，構文木に残さない．
                        Ok(Expr {
                            kind: inner.kind,
                            span: token.span.start..span.end,
                        })
                    }
                    _ => Err(Error::new(ErrorKind::MissingClosingParenthesis, token.span)),
                }
            }
            _ => Err(self.unexpected(Some(&token))),
        }
    }
}

/// 式を構文木にする．
///
/// # Errors
///
/// 入力が長すぎるとき，文法に合わないとき，使えない文字があるときに，位置つきの誤りを返す．
pub fn parse(source: &str) -> Result<Expr, Error> {
    let length = source.chars().count();
    if length > MAX_INPUT_CHARS {
        return Err(Error::new(ErrorKind::InputTooLong, 0..length));
    }
    let mut parser = Parser {
        tokens: tokenize(source)?,
        position: 0,
        depth: 0,
        length,
    };
    let expression = parser.parse_expression()?;
    match parser.peek() {
        None => Ok(expression),
        Some(token) => Err(parser.unexpected(Some(token))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rational::Rational;

    /// 構文木を，括弧で構造を示した文字列にする．
    fn show(expr: &Expr) -> String {
        match &expr.kind {
            ExprKind::Number(value) => value.to_string(),
            ExprKind::Variable(name) => name.to_string(),
            ExprKind::Negate(inner) => format!("(-{})", show(inner)),
            ExprKind::Binary {
                operator,
                left,
                right,
            } => {
                let symbol = match operator {
                    BinaryOperator::Add => '+',
                    BinaryOperator::Subtract => '-',
                    BinaryOperator::Multiply => '*',
                    BinaryOperator::Divide => '/',
                    BinaryOperator::Power => '^',
                };
                format!("({} {symbol} {})", show(left), show(right))
            }
        }
    }

    fn shape(source: &str) -> String {
        show(&parse(source).expect("構文解析に成功する"))
    }

    fn error_of(source: &str) -> Error {
        parse(source).expect_err("構文解析に失敗する")
    }

    #[test]
    fn 掛け算は足し算より強く結ぶ() {
        assert_eq!(shape("1+2*3"), "(1 + (2 * 3))");
        assert_eq!(shape("1*2+3"), "((1 * 2) + 3)");
    }

    #[test]
    fn 足し算と引き算と割り算は左に結ぶ() {
        assert_eq!(shape("1-2-3"), "((1 - 2) - 3)");
        assert_eq!(shape("8/4/2"), "((8 / 4) / 2)");
    }

    #[test]
    fn 累乗は右に結び符号より強い() {
        assert_eq!(shape("2^3^2"), "(2 ^ (3 ^ 2))");
        assert_eq!(shape("-x^2"), "(-(x ^ 2))");
        assert_eq!(shape("x^-1"), "(x ^ (-1))");
    }

    #[test]
    fn 省略された掛け算を読む() {
        assert_eq!(shape("2x"), "(2 * x)");
        assert_eq!(shape("2x^2y"), "((2 * (x ^ 2)) * y)");
        assert_eq!(shape("(x+1)(x-1)"), "((x + 1) * (x - 1))");
        assert_eq!(shape("x(x+1)"), "(x * (x + 1))");
    }

    #[test]
    fn 数の前の掛け算は省略できない() {
        let error = error_of("x 2");
        assert_eq!(error.kind, ErrorKind::UnexpectedToken("2".to_string()));
        assert_eq!(error.span, 2..3);
    }

    #[test]
    fn 括弧は構文木に残さず範囲に含める() {
        let expr = parse("(x)").expect("構文解析に成功する");
        assert_eq!(expr.kind, ExprKind::Variable('x'));
        assert_eq!(expr.span, 0..3);
    }

    #[test]
    fn 数は正確な有理数で持つ() {
        assert_eq!(
            parse("0.5").expect("成功").kind,
            ExprKind::Number(Rational::new(1, 2).expect("有理数"))
        );
    }

    #[test]
    fn 閉じ括弧がないとき開き括弧の位置を示す() {
        let error = error_of("2(x+1");
        assert_eq!(error.kind, ErrorKind::MissingClosingParenthesis);
        assert_eq!(error.span, 1..2);
    }

    #[test]
    fn 式が途中で終わるとき末尾の位置を示す() {
        for source in ["", "1+", "2*", "x^", "-"] {
            let error = error_of(source);
            assert_eq!(error.kind, ErrorKind::UnexpectedEnd, "{source}");
            let length = source.chars().count();
            assert_eq!(error.span, length..length, "{source}");
        }
    }

    #[test]
    fn 余分な記号の位置を示す() {
        let error = error_of("1 + * 2");
        assert_eq!(error.kind, ErrorKind::UnexpectedToken("*".to_string()));
        assert_eq!(error.span, 4..5);
        let error = error_of("(1))");
        assert_eq!(error.kind, ErrorKind::UnexpectedToken(")".to_string()));
        assert_eq!(error.span, 3..4);
    }

    #[test]
    fn 入れ子が深すぎる入力を誤りにする() {
        let source = format!("{}1{}", "(".repeat(300), ")".repeat(300));
        assert_eq!(error_of(&source).kind, ErrorKind::TooDeep);
        let source = format!("{}1", "-".repeat(300));
        assert_eq!(error_of(&source).kind, ErrorKind::TooDeep);
    }

    #[test]
    fn 長すぎる入力を誤りにする() {
        let source = "1+".repeat(MAX_INPUT_CHARS);
        assert_eq!(error_of(&source).kind, ErrorKind::InputTooLong);
    }
}
