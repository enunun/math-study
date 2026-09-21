//! 式の構文木．

use crate::error::Span;
use crate::rational::Rational;

/// 二項演算子．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// 足し算．
    Add,
    /// 引き算．
    Subtract,
    /// 掛け算．省略された掛け算(`2x`，`(x+1)(x-1)`)も含む．
    Multiply,
    /// 割り算．
    Divide,
    /// 累乗．
    Power,
}

/// 構文木の節の種類．
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    /// 数．
    Number(Rational),
    /// 変数．1文字である．
    Variable(char),
    /// 符号の反転．
    Negate(Box<Expr>),
    /// 二項演算．
    Binary {
        /// 演算子．
        operator: BinaryOperator,
        /// 左の項．
        left: Box<Expr>,
        /// 右の項．
        right: Box<Expr>,
    },
}

/// 入力の範囲を持つ，構文木の節．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    /// 節の種類．
    pub kind: ExprKind,
    /// 節に対応する，入力の範囲．
    pub span: Span,
}

impl Expr {
    /// 二項演算の節を作る．範囲は，左の項の始めから，右の項の終わりまでである．
    #[must_use]
    pub fn binary(operator: BinaryOperator, left: Self, right: Self) -> Self {
        let span = left.span.start..right.span.end;
        Self {
            kind: ExprKind::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            },
            span,
        }
    }
}
