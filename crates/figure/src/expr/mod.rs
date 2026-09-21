//! 図の式(`sin(x - shift)`など)を，構文木にして，浮動小数点で評価する．

use std::ops::Range;

mod lexer;
mod parser;

/// 入力の中の範囲．バイトではなく，文字(Unicodeのスカラー値)の番号で数える．
pub type Span = Range<usize>;

/// 式の誤りの種類．
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprErrorKind {
    /// 使えない文字がある．
    UnexpectedCharacter(char),
    /// 置けない位置に，記号や数，名前がある．中身は，見つかったものの表示．
    UnexpectedToken(String),
    /// 式が途中で終わっている．
    UnexpectedEnd,
    /// 開き括弧に対応する閉じ括弧がない．
    MissingClosingParenthesis,
    /// 使える名前(変数，媒介変数，定数)にない名前である．
    UnknownName(String),
    /// 使える関数にない名前が，関数として呼ばれている．
    UnknownFunction(String),
    /// 関数の名前だけで，引数がない．
    FunctionNeedsArgument(String),
    /// 式が長すぎる．
    TooLong,
    /// 括弧や記号の入れ子が深すぎる．
    TooDeep,
}

/// 式の中の位置を持つ誤り．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprError {
    /// 誤りの種類．
    pub kind: ExprErrorKind,
    /// 誤りのある範囲．
    pub span: Span,
}

/// 入力の長さの上限(文字数)．
const MAX_INPUT_CHARS: usize = 2000;

/// 2項演算．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

/// 使える関数．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Function {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Exp,
    Log,
    Sqrt,
    Abs,
}

impl Function {
    /// 名前から関数を探す．`log`と`ln`は，どちらも自然対数である．
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "sin" => Self::Sin,
            "cos" => Self::Cos,
            "tan" => Self::Tan,
            "asin" => Self::Asin,
            "acos" => Self::Acos,
            "atan" => Self::Atan,
            "sinh" => Self::Sinh,
            "cosh" => Self::Cosh,
            "tanh" => Self::Tanh,
            "exp" => Self::Exp,
            "log" | "ln" => Self::Log,
            "sqrt" => Self::Sqrt,
            "abs" => Self::Abs,
            _ => return None,
        })
    }

    fn apply(self, x: f64) -> f64 {
        match self {
            Self::Sin => x.sin(),
            Self::Cos => x.cos(),
            Self::Tan => x.tan(),
            Self::Asin => x.asin(),
            Self::Acos => x.acos(),
            Self::Atan => x.atan(),
            Self::Sinh => x.sinh(),
            Self::Cosh => x.cosh(),
            Self::Tanh => x.tanh(),
            Self::Exp => x.exp(),
            Self::Log => x.ln(),
            Self::Sqrt => x.sqrt(),
            Self::Abs => x.abs(),
        }
    }
}

/// 関数か定数の名前か．媒介変数の`id`や変数の名前には使えない．
#[must_use]
pub fn is_reserved_name(name: &str) -> bool {
    Function::from_name(name).is_some() || parser::is_constant(name)
}

/// 構文木の節．
#[derive(Debug, Clone, PartialEq)]
enum Node {
    Number(f64),
    /// `compile`に渡した名前の，何番目か．
    Variable(usize),
    Neg(Box<Node>),
    Binary(BinaryOp, Box<Node>, Box<Node>),
    Call(Function, Box<Node>),
}

impl Node {
    fn eval(&self, values: &[f64]) -> f64 {
        match self {
            Self::Number(value) => *value,
            Self::Variable(index) => values.get(*index).copied().unwrap_or(f64::NAN),
            Self::Neg(operand) => -operand.eval(values),
            Self::Binary(op, left, right) => {
                let (left, right) = (left.eval(values), right.eval(values));
                match op {
                    BinaryOp::Add => left + right,
                    BinaryOp::Sub => left - right,
                    BinaryOp::Mul => left * right,
                    BinaryOp::Div => left / right,
                    BinaryOp::Pow => left.powf(right),
                }
            }
            Self::Call(function, argument) => function.apply(argument.eval(values)),
        }
    }
}

/// 構文木にした式．
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    root: Node,
}

impl Expr {
    /// 式を読み，構文木にする．`names`にある名前は，変数として使える．
    ///
    /// # Errors
    ///
    /// 構文の誤り，使えない名前，長さや深さの上限の超過があると，誤りを返す．
    pub fn compile(source: &str, names: &[&str]) -> Result<Self, ExprError> {
        let chars: Vec<char> = source.chars().collect();
        if chars.len() > MAX_INPUT_CHARS {
            return Err(ExprError {
                kind: ExprErrorKind::TooLong,
                span: 0..chars.len(),
            });
        }
        let tokens = lexer::tokenize(&chars)?;
        let root = parser::parse(&tokens, names)?;
        Ok(Self { root })
    }

    /// 変数に値を入れて，式を評価する．`values`は，`compile`に渡した`names`と同じ順に並べる．
    /// 定義域の外では，無限大や非数(NaN)になる．値が足りない変数は，非数として扱う．
    #[must_use]
    pub fn eval(&self, values: &[f64]) -> f64 {
        self.root.eval(values)
    }
}
