//! 式の字句解析．文字を，数，名前，記号に分ける．

use super::{ExprError, ExprErrorKind, Span};

/// 字句の種類．
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// 数．
    Number(f64),
    /// 名前(変数，定数，関数)．
    Name(String),
    /// `+`．
    Plus,
    /// `-`．
    Minus,
    /// `*`．
    Star,
    /// `/`．
    Slash,
    /// `^`．
    Caret,
    /// `(`．
    OpenParen,
    /// `)`．
    CloseParen,
    /// 式の終わり．
    End,
}

/// 入力の中の位置を持つ字句．
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// 種類．
    pub kind: TokenKind,
    /// 位置．
    pub span: Span,
    /// 入力での表記．誤りの説明に使う．
    pub text: String,
}

/// 式を字句に分ける．最後に，式の終わりの字句を置く．
///
/// # Errors
///
/// 使えない文字があると，誤りを返す．
pub fn tokenize(chars: &[char]) -> Result<Vec<Token>, ExprError> {
    let mut tokens = Vec::new();
    let mut position = 0;
    while let Some(&c) = chars.get(position) {
        if c.is_whitespace() {
            position = position.saturating_add(1);
        } else if c.is_ascii_digit() {
            position = read_number(chars, position, &mut tokens);
        } else if c.is_ascii_alphabetic() {
            position = read_name(chars, position, &mut tokens);
        } else {
            tokens.push(symbol(c, position)?);
            position = position.saturating_add(1);
        }
    }
    tokens.push(Token {
        kind: TokenKind::End,
        span: chars.len()..chars.len(),
        text: String::new(),
    });
    Ok(tokens)
}

fn symbol(c: char, position: usize) -> Result<Token, ExprError> {
    let kind = match c {
        '+' => TokenKind::Plus,
        '-' => TokenKind::Minus,
        '*' => TokenKind::Star,
        '/' => TokenKind::Slash,
        '^' => TokenKind::Caret,
        '(' => TokenKind::OpenParen,
        ')' => TokenKind::CloseParen,
        other => {
            return Err(ExprError {
                kind: ExprErrorKind::UnexpectedCharacter(other),
                span: position..position.saturating_add(1),
            });
        }
    };
    Ok(Token {
        kind,
        span: position..position.saturating_add(1),
        text: c.to_string(),
    })
}

/// 数字の連続と，小数点に続く数字の連続を読む．小数点の後に数字がないときは，小数点を読まない．
fn read_number(chars: &[char], start: usize, tokens: &mut Vec<Token>) -> usize {
    let mut end = skip_while(chars, start, |c| c.is_ascii_digit());
    let has_fraction = chars.get(end) == Some(&'.')
        && chars
            .get(end.saturating_add(1))
            .is_some_and(char::is_ascii_digit);
    if has_fraction {
        end = skip_while(chars, end.saturating_add(1), |c| c.is_ascii_digit());
    }
    let text: String = chars.get(start..end).unwrap_or_default().iter().collect();
    tokens.push(Token {
        kind: TokenKind::Number(text.parse().unwrap_or(f64::NAN)),
        span: start..end,
        text,
    });
    end
}

fn read_name(chars: &[char], start: usize, tokens: &mut Vec<Token>) -> usize {
    let end = skip_while(chars, start, |c| c.is_ascii_alphanumeric() || c == '_');
    let text: String = chars.get(start..end).unwrap_or_default().iter().collect();
    tokens.push(Token {
        kind: TokenKind::Name(text.clone()),
        span: start..end,
        text,
    });
    end
}

fn skip_while(chars: &[char], start: usize, keep: impl Fn(char) -> bool) -> usize {
    let mut end = start;
    while chars.get(end).is_some_and(|&c| keep(c)) {
        end = end.saturating_add(1);
    }
    end
}
