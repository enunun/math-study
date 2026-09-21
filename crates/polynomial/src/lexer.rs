//! 字句解析．入力の文字列を，記号の列に分ける．

use crate::error::{Error, ErrorKind, Span};
use crate::rational::Rational;

/// 記号の種類．
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// 数．整数と小数を，正確な有理数にしたもの．
    Number(Rational),
    /// 変数．英字1文字である．
    Letter(char),
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
    LeftParen,
    /// `)`．
    RightParen,
}

/// 記号と，その入力の範囲，入力での表記．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// 種類．
    pub kind: TokenKind,
    /// 入力の中の範囲．
    pub span: Span,
    /// 入力での表記．誤りの表示に使う．
    pub text: String,
}

/// 全角の記号を，半角にそろえる．日本語入力のまま，式を書けるようにするためである．
fn normalize(c: char) -> char {
    match c {
        '\u{3000}' => ' ',
        '\u{2212}' => '-',
        '×' | '·' => '*',
        '÷' => '/',
        '\u{FF01}'..='\u{FF5E}' => u32::from(c)
            .checked_sub(0xFEE0)
            .and_then(char::from_u32)
            .unwrap_or(c),
        _ => c,
    }
}

fn next(index: usize) -> usize {
    index.saturating_add(1)
}

/// `start`から始まる数を読み，値と，終わりの位置を返す．小数は，正確な有理数にする．
fn scan_number(chars: &[char], start: usize) -> Result<(Rational, usize), Error> {
    let overflow = |end: usize| Error::new(ErrorKind::Overflow, start..end);
    let mut end = start;
    let mut digits: i128 = 0;
    let mut scale: i128 = 1;
    let mut seen_dot = false;
    let mut digits_after_dot = 0_usize;
    while let Some(&raw) = chars.get(end) {
        let c = normalize(raw);
        if let Some(digit) = c.to_digit(10) {
            digits = digits
                .checked_mul(10)
                .and_then(|value| value.checked_add(i128::from(digit)))
                .ok_or_else(|| overflow(next(end)))?;
            if seen_dot {
                scale = scale.checked_mul(10).ok_or_else(|| overflow(next(end)))?;
                digits_after_dot = next(digits_after_dot);
            }
        } else if c == '.' && !seen_dot {
            seen_dot = true;
        } else {
            break;
        }
        end = next(end);
    }
    if seen_dot && digits_after_dot == 0 {
        // 「1.」のように，小数点の後に数字がない．
        return Err(Error::new(
            ErrorKind::UnexpectedCharacter('.'),
            end.saturating_sub(1)..end,
        ));
    }
    let value = Rational::new(digits, scale).ok_or_else(|| overflow(end))?;
    Ok((value, end))
}

/// 入力を，記号の列にする．空白は読み飛ばす．
///
/// # Errors
///
/// 使えない文字があるとき，または，数が大きすぎるときに，位置つきの誤りを返す．
pub fn tokenize(source: &str) -> Result<Vec<Token>, Error> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;
    while let Some(&raw) = chars.get(index) {
        let c = normalize(raw);
        let start = index;
        let kind = match c {
            _ if c.is_whitespace() => {
                index = next(index);
                continue;
            }
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '^' => TokenKind::Caret,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            _ if c.is_ascii_alphabetic() => TokenKind::Letter(c),
            _ if c.is_ascii_digit()
                || (c == '.'
                    && chars
                        .get(next(index))
                        .is_some_and(|&d| normalize(d).is_ascii_digit())) =>
            {
                let (value, end) = scan_number(&chars, index)?;
                index = end;
                let text = chars.get(start..end).unwrap_or_default().iter().collect();
                tokens.push(Token {
                    kind: TokenKind::Number(value),
                    span: start..end,
                    text,
                });
                continue;
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::UnexpectedCharacter(raw),
                    start..next(start),
                ));
            }
        };
        index = next(index);
        tokens.push(Token {
            kind,
            span: start..index,
            text: raw.to_string(),
        });
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        tokenize(source)
            .expect("字句解析に成功する")
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    #[test]
    fn 記号と数と変数に分ける() {
        assert_eq!(
            kinds("2x^3 + (y-1)/4"),
            vec![
                TokenKind::Number(Rational::from_integer(2)),
                TokenKind::Letter('x'),
                TokenKind::Caret,
                TokenKind::Number(Rational::from_integer(3)),
                TokenKind::Plus,
                TokenKind::LeftParen,
                TokenKind::Letter('y'),
                TokenKind::Minus,
                TokenKind::Number(Rational::ONE),
                TokenKind::RightParen,
                TokenKind::Slash,
                TokenKind::Number(Rational::from_integer(4)),
            ]
        );
    }

    #[test]
    fn 小数を正確な有理数にする() {
        assert_eq!(
            kinds("0.25"),
            vec![TokenKind::Number(Rational::new(1, 4).expect("有理数"))]
        );
        assert_eq!(
            kinds(".5"),
            vec![TokenKind::Number(Rational::new(1, 2).expect("有理数"))]
        );
    }

    #[test]
    fn 全角の記号と数字を半角にそろえる() {
        assert_eq!(kinds("（ｘ＋１）＾２"), kinds("(x+1)^2"));
        assert_eq!(kinds("2×x÷3−1"), kinds("2*x/3-1"));
        assert_eq!(kinds("x\u{3000}+\u{3000}y"), kinds("x+y"));
    }

    #[test]
    fn 記号の範囲は文字の番号で数える() {
        let tokens = tokenize("あ").expect_err("使えない文字");
        assert_eq!(tokens.span, 0..1);
        let tokens = tokenize("x + 12").expect("字句解析に成功する");
        assert_eq!(tokens.last().map(|token| token.span.clone()), Some(4..6));
    }

    #[test]
    fn 使えない文字を位置つきで誤りにする() {
        let error = tokenize("x + @").expect_err("使えない文字");
        assert_eq!(error.kind, ErrorKind::UnexpectedCharacter('@'));
        assert_eq!(error.span, 4..5);
    }

    #[test]
    fn 小数点だけの入力や末尾の小数点を誤りにする() {
        assert_eq!(
            tokenize(".").expect_err("小数点だけ").kind,
            ErrorKind::UnexpectedCharacter('.')
        );
        assert_eq!(
            tokenize("1.").expect_err("末尾の小数点").kind,
            ErrorKind::UnexpectedCharacter('.')
        );
    }

    #[test]
    fn 大きすぎる数を誤りにする() {
        let error = tokenize(&"9".repeat(60)).expect_err("あふれる");
        assert_eq!(error.kind, ErrorKind::Overflow);
    }
}
