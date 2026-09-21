//! 式の解析と計算で起こる誤り．

use std::fmt;
use std::ops::Range;

/// 入力の中の範囲．バイトやUTF-16の単位ではなく，文字(Unicodeのスカラー値)の番号で数える．
pub type Span = Range<usize>;

/// 誤りの種類．
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// 使えない文字がある．
    UnexpectedCharacter(char),
    /// 置けない位置に，記号や数，文字がある．中身は，見つかったものの表示．
    UnexpectedToken(String),
    /// 式が途中で終わっている．
    UnexpectedEnd,
    /// 開き括弧に対応する閉じ括弧がない．
    MissingClosingParenthesis,
    /// 入力が長すぎる．
    InputTooLong,
    /// 括弧や記号の入れ子が深すぎる．
    TooDeep,
    /// 数が大きすぎて，正確に扱えない．
    Overflow,
    /// 結果の項が多すぎる．
    TooLarge,
    /// 0で割っている．
    DivisionByZero,
    /// 定数以外で割っている．多項式での割り算は扱えない．
    NonConstantDivisor,
    /// 指数が，0以上の整数でない，または大きすぎる．
    InvalidExponent,
}

/// 入力の中の位置を持つ誤り．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// 誤りの種類．
    pub kind: ErrorKind,
    /// 誤りのある範囲．
    pub span: Span,
}

impl Error {
    /// 誤りを作る．
    #[must_use]
    pub fn new(kind: ErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::UnexpectedCharacter(c) => write!(f, "使えない文字「{c}」がある．"),
            ErrorKind::UnexpectedToken(token) => {
                write!(
                    f,
                    "ここに「{token}」は置けない．演算子が足りないか，多い可能性がある．"
                )
            }
            ErrorKind::UnexpectedEnd => write!(f, "式が途中で終わっている．"),
            ErrorKind::MissingClosingParenthesis => {
                write!(f, "開き括弧に対応する閉じ括弧がない．")
            }
            ErrorKind::InputTooLong => write!(f, "入力が長すぎる．"),
            ErrorKind::TooDeep => write!(f, "括弧や記号の入れ子が深すぎる．"),
            ErrorKind::Overflow => write!(f, "数が大きすぎて，正確に扱えない．"),
            ErrorKind::TooLarge => write!(f, "結果の項が多すぎる．"),
            ErrorKind::DivisionByZero => write!(f, "0で割ることはできない．"),
            ErrorKind::NonConstantDivisor => {
                write!(f, "割る数は，定数にする．多項式での割り算は扱えない．")
            }
            ErrorKind::InvalidExponent => {
                write!(f, "指数は，0以上で，大きすぎない整数にする．")
            }
        }
    }
}

impl std::error::Error for Error {}
