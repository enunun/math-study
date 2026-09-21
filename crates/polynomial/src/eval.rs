//! 構文木を，多項式に評価する．

use crate::ast::{BinaryOperator, Expr, ExprKind};
use crate::error::{Error, ErrorKind, Span};
use crate::polynomial::{ArithmeticError, MAX_EXPONENT, Polynomial};

fn arithmetic(error: ArithmeticError, span: &Span) -> Error {
    let kind = match error {
        ArithmeticError::Overflow => ErrorKind::Overflow,
        ArithmeticError::TooLarge => ErrorKind::TooLarge,
    };
    Error::new(kind, span.clone())
}

/// 割り算．割る数は，0でない定数に限る．
fn divide(dividend: &Polynomial, divisor: &Expr) -> Result<Polynomial, Error> {
    let constant = evaluate(divisor)?
        .as_constant()
        .ok_or_else(|| Error::new(ErrorKind::NonConstantDivisor, divisor.span.clone()))?;
    let reciprocal = constant
        .checked_recip()
        .ok_or_else(|| Error::new(ErrorKind::DivisionByZero, divisor.span.clone()))?;
    dividend
        .checked_scale(&reciprocal)
        .map_err(|error| arithmetic(error, &divisor.span))
}

/// 累乗．指数は，0以上で，上限以下の整数に限る．
fn power(base: &Polynomial, exponent: &Expr) -> Result<Polynomial, Error> {
    let invalid = || Error::new(ErrorKind::InvalidExponent, exponent.span.clone());
    let value = evaluate(exponent)?
        .as_constant()
        .and_then(|constant| constant.to_non_negative_u32())
        .filter(|&value| value <= MAX_EXPONENT)
        .ok_or_else(invalid)?;
    base.checked_pow(value)
        .map_err(|error| arithmetic(error, &exponent.span))
}

/// 構文木を，展開した多項式にする．
///
/// # Errors
///
/// 数があふれるとき，割り算や累乗の条件に合わないとき，結果の項が多すぎるときに，位置つきの誤りを返す．
pub fn evaluate(expr: &Expr) -> Result<Polynomial, Error> {
    match &expr.kind {
        ExprKind::Number(value) => Ok(Polynomial::constant(*value)),
        ExprKind::Variable(name) => Ok(Polynomial::variable(*name)),
        ExprKind::Negate(inner) => evaluate(inner)?
            .checked_neg()
            .map_err(|error| arithmetic(error, &expr.span)),
        ExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let left_value = evaluate(left)?;
            let combine = |result: Result<Polynomial, ArithmeticError>| {
                result.map_err(|error| arithmetic(error, &expr.span))
            };
            match operator {
                BinaryOperator::Divide => divide(&left_value, right),
                BinaryOperator::Power => power(&left_value, right),
                BinaryOperator::Add => combine(left_value.checked_add(&evaluate(right)?)),
                BinaryOperator::Subtract => combine(left_value.checked_sub(&evaluate(right)?)),
                BinaryOperator::Multiply => combine(left_value.checked_mul(&evaluate(right)?)),
            }
        }
    }
}
