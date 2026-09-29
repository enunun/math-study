//! 点の式の評価．点をベクトルのまま評価し，内積(`dot`)，外積(`cross`)，長さ(`norm`)を使えるようにする．
//! 値の種類(数かベクトルか)は，値によらず式の形で決まるので，評価と同時に，式が正しいかも確かめる．

use super::{BinaryOp, Node};

/// ベクトルの関数．点の式でだけ使える．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VectorFunction {
    /// 内積．2つのベクトルから数．
    Dot,
    /// 外積．空間では2つのベクトルからベクトル，平面では数(z成分)．
    Cross,
    /// 長さ．1つのベクトルから数．
    Norm,
}

/// ベクトルの関数の名前と関数．
pub(super) const VECTOR_FUNCTIONS: [(&str, VectorFunction); 3] = [
    ("dot", VectorFunction::Dot),
    ("cross", VectorFunction::Cross),
    ("norm", VectorFunction::Norm),
];

impl VectorFunction {
    pub(super) fn from_name(name: &str) -> Option<Self> {
        VECTOR_FUNCTIONS
            .iter()
            .find(|(function, _)| *function == name)
            .map(|(_, function)| *function)
    }

    /// 引数の数．
    pub(super) const fn arity(self) -> usize {
        match self {
            Self::Dot | Self::Cross => 2,
            Self::Norm => 1,
        }
    }

    fn apply(self, arguments: &[Value]) -> Result<Value, &'static str> {
        let vectors: Vec<&[f64]> = arguments
            .iter()
            .map(|argument| match argument {
                Value::Vector(vector) => Ok(vector.as_slice()),
                Value::Number(_) => {
                    Err("内積(`dot`)，外積(`cross`)，長さ(`norm`)の引数は，点(ベクトル)にする．")
                }
            })
            .collect::<Result<_, _>>()?;
        Ok(match (self, vectors.as_slice()) {
            (Self::Dot, [u, v]) => Value::Number(dot(u, v)),
            (Self::Norm, [u]) => Value::Number(dot(u, u).sqrt()),
            (Self::Cross, [[a, b], [c, d]]) => Value::Number(a * d - b * c),
            (Self::Cross, [[a1, a2, a3], [b1, b2, b3]]) => Value::Vector(vec![
                a2 * b3 - a3 * b2,
                a3 * b1 - a1 * b3,
                a1 * b2 - a2 * b1,
            ]),
            _ => return Err("外積(`cross`)は，平面か空間の点(2個か3個の座標)にだけ使える．"),
        })
    }
}

fn dot(u: &[f64], v: &[f64]) -> f64 {
    u.iter().zip(v).map(|(a, b)| a * b).sum()
}

/// 点の式の値．数かベクトル．
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 数．
    Number(f64),
    /// ベクトル(点の位置ベクトル)．
    Vector(Vec<f64>),
}

impl Value {
    fn scale(vector: &[f64], factor: f64) -> Self {
        Self::Vector(vector.iter().map(|c| c * factor).collect())
    }
}

/// 2項演算．数どうしは数の演算，ベクトルどうしは和と差，数とベクトルは倍と数での割り算だけができる．
fn binary(op: BinaryOp, left: Value, right: Value) -> Result<Value, &'static str> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(match op {
            BinaryOp::Add => a + b,
            BinaryOp::Sub => a - b,
            BinaryOp::Mul => a * b,
            BinaryOp::Div => a / b,
            BinaryOp::Pow => a.powf(b),
        })),
        (Value::Vector(u), Value::Vector(v)) => match op {
            BinaryOp::Add => Ok(Value::Vector(
                u.iter().zip(&v).map(|(a, b)| a + b).collect(),
            )),
            BinaryOp::Sub => Ok(Value::Vector(
                u.iter().zip(&v).map(|(a, b)| a - b).collect(),
            )),
            BinaryOp::Mul => {
                Err("点どうしの積は書けない．内積は`dot(A, B)`，外積は`cross(A, B)`で書く．")
            }
            BinaryOp::Div => Err("点で割ることはできない．"),
            BinaryOp::Pow => Err("点のべき乗は書けない．"),
        },
        (Value::Number(a), Value::Vector(v)) => match op {
            BinaryOp::Mul => Ok(Value::scale(&v, a)),
            BinaryOp::Add | BinaryOp::Sub => Err("点に数を足し引きすることはできない．"),
            BinaryOp::Div => Err("点で割ることはできない．"),
            BinaryOp::Pow => Err("点のべき乗は書けない．"),
        },
        (Value::Vector(u), Value::Number(b)) => match op {
            BinaryOp::Mul => Ok(Value::scale(&u, b)),
            BinaryOp::Div => Ok(Value::scale(&u, 1.0 / b)),
            BinaryOp::Add | BinaryOp::Sub => Err("点に数を足し引きすることはできない．"),
            BinaryOp::Pow => Err("点のべき乗は書けない．"),
        },
    }
}

impl Node {
    /// 点の式として評価する．`value_of`は，名前の番号から値(媒介変数なら数，点ならベクトル)を返す．
    /// 式が点の式として正しくなければ，理由を返す．
    pub(super) fn eval_vector(
        &self,
        value_of: &dyn Fn(usize) -> Value,
    ) -> Result<Value, &'static str> {
        match self {
            Self::Number(value) => Ok(Value::Number(*value)),
            Self::Imaginary => Err("虚数単位`i`は，複素数の式でだけ使える．"),
            Self::Variable(index) => Ok(value_of(*index)),
            Self::Neg(operand) => Ok(match operand.eval_vector(value_of)? {
                Value::Number(value) => Value::Number(-value),
                Value::Vector(vector) => Value::scale(&vector, -1.0),
            }),
            Self::Binary(op, left, right) => binary(
                *op,
                left.eval_vector(value_of)?,
                right.eval_vector(value_of)?,
            ),
            Self::Call(function, arguments) => {
                let mut values = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    match argument.eval_vector(value_of)? {
                        Value::Number(value) => values.push(value),
                        Value::Vector(_) => {
                            return Err(
                                "点を数の関数(`sin`など)に入れることはできない．長さは`norm(A)`で書く．",
                            );
                        }
                    }
                }
                Ok(Value::Number(function.apply(&values)))
            }
            Self::Vector(function, arguments) => {
                let values = arguments
                    .iter()
                    .map(|argument| argument.eval_vector(value_of))
                    .collect::<Result<Vec<_>, _>>()?;
                function.apply(&values)
            }
        }
    }

    /// ベクトルの関数を使っているか．
    pub(super) fn uses_vector_functions(&self) -> bool {
        match self {
            Self::Number(_) | Self::Imaginary | Self::Variable(_) => false,
            Self::Neg(operand) => operand.uses_vector_functions(),
            Self::Call(_, arguments) => arguments.iter().any(Self::uses_vector_functions),
            Self::Binary(_, left, right) => {
                left.uses_vector_functions() || right.uses_vector_functions()
            }
            Self::Vector(..) => true,
        }
    }
}
