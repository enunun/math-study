//! 計算機の入口(`calculate`)を通して，式の意味と，出力，誤りの位置を確かめる．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use polynomial::eval::evaluate;
use polynomial::parser::parse;
use polynomial::polynomial::Polynomial;
use polynomial::{ErrorKind, calculate};

fn expand(source: &str) -> Polynomial {
    evaluate(&parse(source).expect("構文解析に成功する")).expect("評価に成功する")
}

fn text(source: &str) -> String {
    calculate(source).expect("計算に成功する").expanded.text
}

fn tex(source: &str) -> String {
    calculate(source).expect("計算に成功する").expanded.tex
}

#[test]
fn 和の二乗を展開する() {
    assert_eq!(text("(x+1)^2"), "x^2 + 2*x + 1");
    assert_eq!(tex("(x+1)^2"), "x^2 + 2x + 1");
}

#[test]
fn 和と差の積を展開する() {
    assert_eq!(text("(x+1)(x-1)"), "x^2 - 1");
    assert_eq!(text("(a+b)(a-b)"), "a^2 - b^2");
}

#[test]
fn 同類項をまとめ項を全次数の大きい順に並べる() {
    assert_eq!(text("1 + 2x + x^3 - x + 4x^2"), "x^3 + 4*x^2 + x + 1");
    assert_eq!(text("y + x^2 + x*y"), "x^2 + x*y + y");
}

#[test]
fn 複数の変数の項は変数の名前の順に並べる() {
    assert_eq!(text("(a+b+c)^2"), "a^2 + 2*a*b + 2*a*c + b^2 + 2*b*c + c^2");
    assert_eq!(tex("x^2y + xy^2"), "x^2y + xy^2");
}

#[test]
fn 打ち消し合って0になる() {
    assert_eq!(text("(x+1)^2 - x^2 - 2x - 1"), "0");
    assert_eq!(tex("x - x"), "0");
}

#[test]
fn 分数の係数を正確に扱う() {
    assert_eq!(text("x/2 + x/3"), "5/6*x");
    assert_eq!(tex("x/2 + x/3"), r"\frac{5}{6}x");
    assert_eq!(tex("-x/2 + 1/4"), r"-\frac{1}{2}x + \frac{1}{4}");
    assert_eq!(text("0.1 + 0.2"), "3/10");
}

#[test]
fn 係数の1と符号を整える() {
    assert_eq!(tex("-x^2 + x - 1"), "-x^2 + x - 1");
    assert_eq!(tex("x^10 + x^2"), "x^{10} + x^2");
    assert_eq!(text("-x"), "-x");
}

#[test]
fn 累乗の指数は0以上の整数に限る() {
    assert_eq!(text("x^0"), "1");
    assert_eq!(text("0^0"), "1");
    assert_eq!(text("2^10"), "1024");
    for source in ["x^-1", "x^(1/2)", "x^y", "x^101"] {
        let error = calculate(source).expect_err("指数の誤り");
        assert_eq!(error.kind, ErrorKind::InvalidExponent, "{source}");
    }
}

#[test]
fn 割る数は0でない定数に限る() {
    assert_eq!(text("(2x+4)/2"), "x + 2");
    assert_eq!(
        calculate("x/0").expect_err("0で割る").kind,
        ErrorKind::DivisionByZero
    );
    assert_eq!(
        calculate("1/x").expect_err("多項式で割る").kind,
        ErrorKind::NonConstantDivisor
    );
    assert_eq!(
        calculate("x/(x-x)").expect_err("0で割る").kind,
        ErrorKind::DivisionByZero
    );
}

#[test]
fn 誤りの位置は割る数と指数を示す() {
    let error = calculate("1 + x/(y+1)").expect_err("多項式で割る");
    assert_eq!(error.span, 6..11);
    let error = calculate("x^(-1)").expect_err("指数の誤り");
    assert_eq!(error.span, 2..6);
}

#[test]
fn 変数ごとに偏微分する() {
    let calculation = calculate("x^3 y + 2x").expect("計算に成功する");
    assert_eq!(calculation.variables, vec!['x', 'y']);
    let derivatives: Vec<(char, String)> = calculation
        .derivatives
        .iter()
        .map(|derivative| (derivative.variable, derivative.result.text.clone()))
        .collect();
    assert_eq!(
        derivatives,
        vec![('x', "3*x^2*y + 2".to_string()), ('y', "x^3".to_string())]
    );
}

#[test]
fn 定数の微分は0で変数は持たない() {
    let calculation = calculate("(x+1)^2 - x^2 - 2x").expect("計算に成功する");
    assert_eq!(calculation.expanded.text, "1");
    assert!(calculation.variables.is_empty());
    assert!(calculation.derivatives.is_empty());
}

#[test]
fn 全角で書いた式も同じ結果になる() {
    assert_eq!(text("（ｘ＋１）＾２"), "x^2 + 2*x + 1");
    assert_eq!(text("２×ｘ÷４"), "1/2*x");
}

#[test]
fn 二項定理と一致する() {
    // (x+1)^n の係数は，二項係数である．
    let expected: Vec<i128> = (0..=20)
        .fold((1_i128, Vec::new()), |(coefficient, mut all), k| {
            all.push(coefficient);
            (coefficient * (20 - k) / (k + 1), all)
        })
        .1;
    let polynomial = expand("(x+1)^20");
    let coefficients: Vec<i128> = polynomial
        .descending_terms()
        .into_iter()
        .rev()
        .map(|(_, coefficient)| coefficient.numerator())
        .collect();
    assert_eq!(coefficients, expected);
}

#[test]
fn 恒等式が成り立つ() {
    let identities = [
        ("(a+b)^2", "a^2 + 2ab + b^2"),
        ("(a+b)^3", "a^3 + 3a^2b + 3ab^2 + b^3"),
        ("(a-b)(a^2+ab+b^2)", "a^3 - b^3"),
        ("(x+1)(x+2)(x+3)", "x^3 + 6x^2 + 11x + 6"),
        ("((x+1)^2)^2", "(x+1)^4"),
        ("(x+y)^2 - (x-y)^2", "4xy"),
    ];
    for (left, right) in identities {
        assert_eq!(expand(left), expand(right), "{left} = {right}");
    }
}

/// 平文で出力した式は，読み戻すと，同じ多項式になる．
#[test]
fn 平文の出力を読み戻すと同じ多項式になる() {
    // 擬似乱数(線形合同法)で，式を作る．外部のクレートを使わない．
    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    let mut next = |bound: u64| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) % bound
    };
    for _ in 0..200 {
        let mut source = String::new();
        for term in 0..=next(4) {
            if term > 0 {
                source.push_str(if next(2) == 0 { " + " } else { " - " });
            }
            source.push_str(&(next(9) + 1).to_string());
            for _ in 0..next(3) {
                source.push(['x', 'y', 'z'][usize::try_from(next(3)).unwrap()]);
                source.push('^');
                source.push_str(&next(4).to_string());
            }
        }
        let polynomial = expand(&source);
        assert_eq!(expand(&polynomial.to_text()), polynomial, "{source}");
    }
}

#[test]
fn 巨大な結果を誤りにし停止しない() {
    assert_eq!(
        calculate("(x+y+z+1)^100").expect_err("項が多すぎる").kind,
        ErrorKind::TooLarge
    );
    assert_eq!(
        calculate("(2^100)^100").expect_err("あふれる").kind,
        ErrorKind::Overflow
    );
    assert_eq!(
        calculate("99999999999999999999999999^2")
            .expect_err("あふれる")
            .kind,
        ErrorKind::Overflow
    );
}

#[test]
fn 空の入力は式が途中で終わる誤りになる() {
    assert_eq!(
        calculate("").expect_err("空").kind,
        ErrorKind::UnexpectedEnd
    );
    assert_eq!(
        calculate("   ").expect_err("空白だけ").kind,
        ErrorKind::UnexpectedEnd
    );
}
