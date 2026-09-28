//! 図の式(`sin(x - shift)`など)を，構文木にして，浮動小数点で評価する．

use std::ops::Range;

mod elliptic;
mod functions;
mod lexer;
mod parser;
mod taylor;
mod vector;

pub use functions::Functions;
pub use vector::Value;
use vector::{VECTOR_FUNCTIONS, VectorFunction};

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
    /// 利用者が定義した関数の，引数の数が合わない．
    ArgumentCount {
        /// 関数の名前．
        name: String,
        /// 定義の引数の数．
        expected: usize,
        /// 呼び出しの引数の数．
        found: usize,
    },
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
    /// Dawson関数．
    Dawson,
    /// Lambertの`W`関数の主枝(`w * exp(w) = x`を満たす`w`のうち，`x >= -1/e`で定まる方)．
    LambertW,
    /// 0次の第1種Bessel関数．
    BesselJ0,
    /// 1次の第1種Bessel関数．
    BesselJ1,
    /// 0次の第2種Bessel関数．
    BesselY0,
    /// 1次の第2種Bessel関数．
    BesselY1,
    /// 0次の第1種変形Bessel関数．
    BesselI0,
    /// 1次の第1種変形Bessel関数．
    BesselI1,
    /// 0次の第2種変形Bessel関数．
    BesselK0,
    /// 1次の第2種変形Bessel関数．
    BesselK1,
    /// 第1種完全楕円積分`K(k)`．
    EllipK,
    /// 第2種完全楕円積分`E(k)`．
    EllipE,
    /// 第1種不完全楕円積分`F(φ, k)`．
    EllipF,
    /// 第2種不完全楕円積分`E(φ, k)`．
    EllipEInc,
    /// Jacobiの振幅関数`am(u, k)`．
    Am,
    /// Jacobiの楕円関数`sn(u, k)`．
    Sn,
    /// Jacobiの楕円関数`cn(u, k)`．
    Cn,
    /// Jacobiの楕円関数`dn(u, k)`．
    Dn,
    /// Weierstrassの楕円関数`℘(z; g2, g3)`．
    Wp,
    /// Weierstrassの楕円関数の導関数`℘'(z; g2, g3)`．
    Wpd,
}

/// 関数の名前と関数．`log`と`ln`，`loggamma`と`lgamma`は，それぞれ同じ関数である．
const FUNCTIONS: [(&str, Function); 39] = [
    ("sin", Function::Sin),
    ("cos", Function::Cos),
    ("tan", Function::Tan),
    ("asin", Function::Asin),
    ("acos", Function::Acos),
    ("atan", Function::Atan),
    ("sinh", Function::Sinh),
    ("cosh", Function::Cosh),
    ("tanh", Function::Tanh),
    ("exp", Function::Exp),
    ("log", Function::Log),
    ("ln", Function::Log),
    ("sqrt", Function::Sqrt),
    ("abs", Function::Abs),
    ("gamma", Function::Gamma),
    ("loggamma", Function::LnGamma),
    ("lgamma", Function::LnGamma),
    ("erf", Function::Erf),
    ("erfc", Function::Erfc),
    ("dawson", Function::Dawson),
    ("lambertw", Function::LambertW),
    ("besselj0", Function::BesselJ0),
    ("besselj1", Function::BesselJ1),
    ("bessely0", Function::BesselY0),
    ("bessely1", Function::BesselY1),
    ("besseli0", Function::BesselI0),
    ("besseli1", Function::BesselI1),
    ("besselk0", Function::BesselK0),
    ("besselk1", Function::BesselK1),
    ("ellipk", Function::EllipK),
    ("ellipe", Function::EllipE),
    ("ellipf", Function::EllipF),
    ("ellipeinc", Function::EllipEInc),
    ("am", Function::Am),
    ("sn", Function::Sn),
    ("cn", Function::Cn),
    ("dn", Function::Dn),
    ("wp", Function::Wp),
    ("wpd", Function::Wpd),
];

impl Function {
    /// 名前から関数を探す．
    fn from_name(name: &str) -> Option<Self> {
        FUNCTIONS
            .iter()
            .find(|(function, _)| *function == name)
            .map(|(_, function)| *function)
    }

    /// 引数の数．
    const fn arity(self) -> usize {
        match self {
            Self::EllipF | Self::EllipEInc | Self::Am | Self::Sn | Self::Cn | Self::Dn => 2,
            Self::Wp | Self::Wpd => 3,
            _ => 1,
        }
    }

    /// 引数の値に施す．`arguments`の長さは`arity`である(読むときに確かめる)．足りない引数は非数とする．
    fn apply(self, arguments: &[f64]) -> f64 {
        let at = |index: usize| arguments.get(index).copied().unwrap_or(f64::NAN);
        let x = at(0);
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
            Self::EllipK => elliptic::ellipk(x),
            Self::EllipE => elliptic::ellipe(x),
            Self::EllipF => elliptic::ellipf(x, at(1)),
            Self::EllipEInc => elliptic::ellipeinc(x, at(1)),
            Self::Am => elliptic::jacobi(x, at(1)).am,
            Self::Sn => elliptic::jacobi(x, at(1)).sn,
            Self::Cn => elliptic::jacobi(x, at(1)).cn,
            Self::Dn => elliptic::jacobi(x, at(1)).dn,
            Self::Wp => elliptic::weierstrass(x, at(1), at(2)).0,
            Self::Wpd => elliptic::weierstrass(x, at(1), at(2)).1,
        }
    }
}

/// 式で使える関数の名前．点の式でだけ使えるベクトルの関数(`dot`，`cross`，`norm`)も含む．
pub fn function_names() -> impl Iterator<Item = &'static str> {
    FUNCTIONS
        .iter()
        .map(|(name, _)| *name)
        .chain(VECTOR_FUNCTIONS.iter().map(|(name, _)| *name))
}

/// 式で使える定数の名前．
pub fn constant_names() -> impl Iterator<Item = &'static str> {
    parser::constant_names()
}

/// 関数か定数の名前か．媒介変数の`id`や変数の名前には使えない．
#[must_use]
pub fn is_reserved_name(name: &str) -> bool {
    Function::from_name(name).is_some()
        || VectorFunction::from_name(name).is_some()
        || parser::is_constant(name)
}

/// 構文木の節．
#[derive(Debug, Clone, PartialEq)]
enum Node {
    Number(f64),
    /// `compile`に渡した名前の，何番目か．
    Variable(usize),
    Neg(Box<Node>),
    Binary(BinaryOp, Box<Node>, Box<Node>),
    Call(Function, Vec<Node>),
    /// ベクトルの関数の呼び出し．点の式でだけ値を持つ．
    Vector(VectorFunction, Vec<Node>),
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
            Self::Call(function, arguments) => {
                let values: Vec<f64> = arguments
                    .iter()
                    .map(|argument| argument.eval(values))
                    .collect();
                function.apply(&values)
            }
            // ベクトルの関数は，点の式でだけ使える(`uses_vector_functions`で断る)．
            Self::Vector(..) => f64::NAN,
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
        Self::compile_with(source, names, &Functions::default())
    }

    /// 式を読み，構文木にする．`functions`にある関数も呼べる．呼び出しは，関数の本体に引数を埋め込んで
    /// 展開するので，できた構文木は，組み込みの関数と四則演算だけでできている．
    ///
    /// # Errors
    ///
    /// 構文の誤り，使えない名前，引数の数の違い，長さや深さの上限の超過があると，誤りを返す．
    pub fn compile_with(
        source: &str,
        names: &[&str],
        functions: &Functions,
    ) -> Result<Self, ExprError> {
        let root = parse_source(source, names, functions)?;
        Ok(Self { root })
    }

    /// 定数だけの式．
    #[must_use]
    pub const fn constant(value: f64) -> Self {
        Self {
            root: Node::Number(value),
        }
    }

    /// 変数`var`(`compile`に渡した名前の番号)について，`values`の値のまわりのTaylor係数を，
    /// 0次から`order`次まで返す．`k`番目は，`k`次の導関数の値を`k!`で割ったものである．
    ///
    /// べき級数の四則演算と，初等関数の級数の漸化式で求める(数値微分はしない)．特殊関数(ガンマ関数など)と，
    /// 展開の中心で微分できない所(`log(0)`，`abs(0)`など)では，`None`を返す．
    #[must_use]
    pub fn taylor(&self, var: usize, values: &[f64], order: usize) -> Option<Vec<f64>> {
        taylor::coefficients(&self.root, var, values, order)
    }
}

/// 式の文字列を，構文木の根にする．関数の本体を読むときにも使う．
fn parse_source(source: &str, names: &[&str], functions: &Functions) -> Result<Node, ExprError> {
    let chars: Vec<char> = source.chars().collect();
    if chars.len() > MAX_INPUT_CHARS {
        return Err(ExprError {
            kind: ExprErrorKind::TooLong,
            span: 0..chars.len(),
        });
    }
    let tokens = lexer::tokenize(&chars)?;
    parser::parse(&tokens, names, functions)
}

impl Expr {
    /// 点の式として評価する．点はベクトルのまま扱い，内積(`dot`)，外積(`cross`)，長さ(`norm`)を使える．
    /// `value_of`は，`compile`に渡した名前の番号から，値(媒介変数なら数，点ならベクトル)を返す．
    ///
    /// # Errors
    ///
    /// 点どうしの積，点への数の足し引き，点を数の関数に入れる式など，点の式として正しくなければ，理由を返す．
    pub fn eval_vector(&self, value_of: &dyn Fn(usize) -> Value) -> Result<Value, &'static str> {
        self.root.eval_vector(value_of)
    }

    /// ベクトルの関数(`dot`，`cross`，`norm`)を使っているか．これらは点の式でだけ使える．
    #[must_use]
    pub fn uses_vector_functions(&self) -> bool {
        self.root.uses_vector_functions()
    }

    /// 変数に値を入れて，式を評価する．`values`は，`compile`に渡した`names`と同じ順に並べる．
    /// 定義域の外では，無限大や非数(NaN)になる．値が足りない変数は，非数として扱う．
    #[must_use]
    pub fn eval(&self, values: &[f64]) -> f64 {
        self.root.eval(values)
    }
}
