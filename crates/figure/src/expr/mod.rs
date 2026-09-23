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
    /// ガンマ関数．階乗の一般化(`gamma(n + 1)`が`n!`)．
    Gamma,
    /// ガンマ関数の自然対数．ガンマ関数自体が大きくなりすぎる引数でも使える．
    LnGamma,
    /// 誤差関数．
    Erf,
    /// 相補誤差関数(`1 - erf(x)`)．
    Erfc,
    /// ドーソン関数．
    Dawson,
    /// ランベルトのW関数の主枝(`w * exp(w) = x`を満たす`w`のうち，`x >= -1/e`で定まる方)．
    LambertW,
    /// 0次の第1種ベッセル関数．
    BesselJ0,
    /// 1次の第1種ベッセル関数．
    BesselJ1,
    /// 0次の第2種ベッセル関数．
    BesselY0,
    /// 1次の第2種ベッセル関数．
    BesselY1,
    /// 0次の第1種変形ベッセル関数．
    BesselI0,
    /// 1次の第1種変形ベッセル関数．
    BesselI1,
    /// 0次の第2種変形ベッセル関数．
    BesselK0,
    /// 1次の第2種変形ベッセル関数．
    BesselK1,
}

impl Function {
    /// 名前から関数を探す．`log`と`ln`，`loggamma`と`lgamma`は，それぞれ同じ関数である．
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
            "gamma" => Self::Gamma,
            "loggamma" | "lgamma" => Self::LnGamma,
            "erf" => Self::Erf,
            "erfc" => Self::Erfc,
            "dawson" => Self::Dawson,
            "lambertw" => Self::LambertW,
            "besselj0" => Self::BesselJ0,
            "besselj1" => Self::BesselJ1,
            "bessely0" => Self::BesselY0,
            "bessely1" => Self::BesselY1,
            "besseli0" => Self::BesselI0,
            "besseli1" => Self::BesselI1,
            "besselk0" => Self::BesselK0,
            "besselk1" => Self::BesselK1,
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
            Self::Gamma => puruspe::gamma(x),
            Self::LnGamma => puruspe::ln_gamma(x),
            Self::Erf => puruspe::erf(x),
            Self::Erfc => puruspe::erfc(x),
            Self::Dawson => puruspe::dawson(x),
            Self::LambertW => puruspe::lambert_w0(x),
            Self::BesselJ0 => puruspe::Jn(0, x),
            Self::BesselJ1 => puruspe::Jn(1, x),
            Self::BesselY0 => puruspe::Yn(0, x),
            Self::BesselY1 => puruspe::Yn(1, x),
            Self::BesselI0 => puruspe::In(0, x),
            Self::BesselI1 => puruspe::In(1, x),
            Self::BesselK0 => puruspe::Kn(0, x),
            Self::BesselK1 => puruspe::Kn(1, x),
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
    fn point_kind(&self, is_point: &dyn Fn(usize) -> bool) -> Option<bool> {
        match self {
            Self::Number(_) => Some(false),
            Self::Variable(index) => Some(is_point(*index)),
            Self::Neg(operand) => operand.point_kind(is_point),
            Self::Binary(op, left, right) => {
                let (left, right) = (left.point_kind(is_point)?, right.point_kind(is_point)?);
                match op {
                    BinaryOp::Add | BinaryOp::Sub => (left == right).then_some(left),
                    BinaryOp::Mul => (!(left && right)).then_some(left || right),
                    BinaryOp::Div => (!right).then_some(left),
                    BinaryOp::Pow => (!(left || right)).then_some(false),
                }
            }
            Self::Call(_, argument) => (!argument.point_kind(is_point)?).then_some(false),
        }
    }

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

    /// 式が，点(ベクトル)の式として正しいかを調べ，値が点なら`Some(true)`，数なら`Some(false)`を返す．
    /// `is_point`は，`compile`に渡した名前の番号が，点の名前かを答える．
    ///
    /// 点は，和と差(点どうし，数どうしだけ)，数の倍(点×数，数×点)，数での割り算(点÷数)ができる．
    /// 点どうしの積，点への数の足し引き，点を関数やべき乗や割る側に使う式は，正しくない(`None`)．
    #[must_use]
    pub fn point_kind(&self, is_point: &dyn Fn(usize) -> bool) -> Option<bool> {
        self.root.point_kind(is_point)
    }

    /// 変数に値を入れて，式を評価する．`values`は，`compile`に渡した`names`と同じ順に並べる．
    /// 定義域の外では，無限大や非数(NaN)になる．値が足りない変数は，非数として扱う．
    #[must_use]
    pub fn eval(&self, values: &[f64]) -> f64 {
        self.root.eval(values)
    }
}
