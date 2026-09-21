//! 多項式の式を解析し，展開と微分を計算する．
//!
//! 式の文字列を，字句解析(`lexer`)，構文解析(`parser`)で構文木(`ast`)にし，
//! 評価(`eval`)で多項式(`polynomial`)にして，TeXと平文(`format`)で出力する．
//! 入口は，`calculator::calculate`である．

#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::panic
    )
)]

pub mod ast;
pub mod calculator;
pub mod error;
pub mod eval;
mod format;
pub mod lexer;
pub mod parser;
pub mod polynomial;
pub mod rational;

pub use calculator::{Calculation, Derivative, Rendered, calculate};
pub use error::{Error, ErrorKind, Span};
