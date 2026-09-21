//! 多項式を，TeXと平文の文字列にする．

use crate::polynomial::{Monomial, Polynomial};
use crate::rational::Rational;

/// 出力の書式．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    /// `MathJax`で描画する`TeX`．
    Tex,
    /// 入力として読み戻せる平文(`x^2 + 2*x + 1`)．
    Text,
}

/// 指数つきの変数の表記．
fn power_text(name: char, exponent: u32, style: Style) -> String {
    match (exponent, style) {
        (1, _) => name.to_string(),
        (2..=9, _) | (_, Style::Text) => format!("{name}^{exponent}"),
        (_, Style::Tex) => format!("{name}^{{{exponent}}}"),
    }
}

fn monomial_text(monomial: &Monomial, style: Style) -> String {
    let powers = monomial
        .factors()
        .iter()
        .map(|&(name, exponent)| power_text(name, exponent, style));
    match style {
        Style::Tex => powers.collect(),
        Style::Text => powers.collect::<Vec<_>>().join("*"),
    }
}

/// 係数の絶対値の表記．
fn magnitude_text(coefficient: &Rational, style: Style) -> String {
    let numerator = coefficient.numerator().unsigned_abs();
    let denominator = coefficient.denominator().unsigned_abs();
    match (denominator, style) {
        (1, _) => numerator.to_string(),
        (_, Style::Tex) => format!("\\frac{{{numerator}}}{{{denominator}}}"),
        (_, Style::Text) => format!("{numerator}/{denominator}"),
    }
}

/// 1つの項の，符号を除いた表記．
fn term_text(monomial: &Monomial, coefficient: &Rational, style: Style) -> String {
    if monomial.is_one() {
        return magnitude_text(coefficient, style);
    }
    let variables = monomial_text(monomial, style);
    let is_unit = coefficient.numerator().unsigned_abs() == 1 && coefficient.is_integer();
    match (is_unit, style) {
        (true, _) => variables,
        (false, Style::Tex) => format!("{}{variables}", magnitude_text(coefficient, style)),
        (false, Style::Text) => format!("{}*{variables}", magnitude_text(coefficient, style)),
    }
}

fn render(polynomial: &Polynomial, style: Style) -> String {
    if polynomial.is_zero() {
        return "0".to_string();
    }
    let mut text = String::new();
    for (index, (monomial, coefficient)) in polynomial.descending_terms().into_iter().enumerate() {
        match (index, coefficient.is_negative()) {
            (0, false) => {}
            (0, true) => text.push('-'),
            (_, false) => text.push_str(" + "),
            (_, true) => text.push_str(" - "),
        }
        text.push_str(&term_text(monomial, coefficient, style));
    }
    text
}

impl Polynomial {
    /// MathJaxで描画するTeXの文字列．全次数の大きい順に，項を並べる．
    #[must_use]
    pub fn to_tex(&self) -> String {
        render(self, Style::Tex)
    }

    /// 平文の文字列．そのまま式として読み戻せる．
    #[must_use]
    pub fn to_text(&self) -> String {
        render(self, Style::Text)
    }
}
