//! 式を入力して，展開と微分の結果を得る，計算機の入口．

use crate::error::{Error, ErrorKind};
use crate::eval::evaluate;
use crate::parser::parse;
use crate::polynomial::{ArithmeticError, Polynomial};

/// 1つの多項式の，2通りの表記．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    /// `MathJax`で描画する`TeX`．
    pub tex: String,
    /// 平文．
    pub text: String,
}

impl Rendered {
    fn of(polynomial: &Polynomial) -> Self {
        Self {
            tex: polynomial.to_tex(),
            text: polynomial.to_text(),
        }
    }
}

/// 1つの変数による偏微分．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Derivative {
    /// 微分する変数．
    pub variable: char,
    /// 微分の結果．
    pub result: Rendered,
}

/// 計算の結果．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calculation {
    /// 展開して整理した式．
    pub expanded: Rendered,
    /// 展開した式に含まれる変数．名前の順である．
    pub variables: Vec<char>,
    /// 変数ごとの偏微分．
    pub derivatives: Vec<Derivative>,
}

/// 式を展開し，含まれる変数ごとに偏微分する．
///
/// # Errors
///
/// 式に誤りがあるとき，または，計算が扱える大きさを超えるときに，位置つきの誤りを返す．
pub fn calculate(source: &str) -> Result<Calculation, Error> {
    let expanded = evaluate(&parse(source)?)?;
    let whole = 0..source.chars().count();
    let variables = expanded.variables();
    let derivatives = variables
        .iter()
        .map(|&variable| {
            let derivative = expanded.derivative(variable).map_err(|error| {
                let kind = match error {
                    ArithmeticError::Overflow => ErrorKind::Overflow,
                    ArithmeticError::TooLarge => ErrorKind::TooLarge,
                };
                Error::new(kind, whole.clone())
            })?;
            Ok(Derivative {
                variable,
                result: Rendered::of(&derivative),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(Calculation {
        expanded: Rendered::of(&expanded),
        variables,
        derivatives,
    })
}
