//! 有理数．分子と分母は，128ビットの整数で持つ．あふれる演算は，停止せずにNoneを返す．

use std::fmt;

/// 既約分数で持つ有理数．分母は正で，0は`0/1`である．
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rational {
    numerator: i128,
    denominator: i128,
}

/// 最大公約数．
fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a.checked_rem(b).unwrap_or(0));
    }
    a
}

impl Rational {
    /// 0．
    pub const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };
    /// 1．
    pub const ONE: Self = Self {
        numerator: 1,
        denominator: 1,
    };

    /// 整数から作る．
    #[must_use]
    pub fn from_integer(value: i128) -> Self {
        Self {
            numerator: value,
            denominator: 1,
        }
    }

    /// 分子と分母から，既約分数を作る．分母が0のとき，または，表せないほど大きいとき，Noneを返す．
    #[must_use]
    pub fn new(numerator: i128, denominator: i128) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        let divisor =
            i128::try_from(gcd(numerator.unsigned_abs(), denominator.unsigned_abs())).ok()?;
        let mut numerator = numerator.checked_div(divisor)?;
        let mut denominator = denominator.checked_div(divisor)?;
        if denominator < 0 {
            numerator = numerator.checked_neg()?;
            denominator = denominator.checked_neg()?;
        }
        Some(Self {
            numerator,
            denominator,
        })
    }

    /// 分子．
    #[must_use]
    pub fn numerator(&self) -> i128 {
        self.numerator
    }

    /// 分母．常に正である．
    #[must_use]
    pub fn denominator(&self) -> i128 {
        self.denominator
    }

    /// 0かどうか．
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.numerator == 0
    }

    /// 負かどうか．
    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.numerator < 0
    }

    /// 整数かどうか．
    #[must_use]
    pub fn is_integer(&self) -> bool {
        self.denominator == 1
    }

    /// 0以上の整数のとき，`u32`で返す．
    #[must_use]
    pub fn to_non_negative_u32(&self) -> Option<u32> {
        if self.is_integer() {
            u32::try_from(self.numerator).ok()
        } else {
            None
        }
    }

    /// 符号を変える．
    #[must_use]
    pub fn checked_neg(&self) -> Option<Self> {
        Some(Self {
            numerator: self.numerator.checked_neg()?,
            denominator: self.denominator,
        })
    }

    /// 和．
    #[must_use]
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        let divisor = i128::try_from(gcd(
            self.denominator.unsigned_abs(),
            other.denominator.unsigned_abs(),
        ))
        .ok()?;
        let left_scale = other.denominator.checked_div(divisor)?;
        let right_scale = self.denominator.checked_div(divisor)?;
        let numerator = self
            .numerator
            .checked_mul(left_scale)?
            .checked_add(other.numerator.checked_mul(right_scale)?)?;
        Self::new(numerator, self.denominator.checked_mul(left_scale)?)
    }

    /// 差．
    #[must_use]
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.checked_add(&other.checked_neg()?)
    }

    /// 積．約分してから掛けて，途中の値があふれにくくする．
    #[must_use]
    pub fn checked_mul(&self, other: &Self) -> Option<Self> {
        let left_divisor = i128::try_from(gcd(
            self.numerator.unsigned_abs(),
            other.denominator.unsigned_abs(),
        ))
        .ok()?;
        let right_divisor = i128::try_from(gcd(
            other.numerator.unsigned_abs(),
            self.denominator.unsigned_abs(),
        ))
        .ok()?;
        // 分母は正だから，最大公約数は1以上で，割り算で0除算にならない．
        let numerator = self
            .numerator
            .checked_div(left_divisor)?
            .checked_mul(other.numerator.checked_div(right_divisor)?)?;
        let denominator = self
            .denominator
            .checked_div(right_divisor)?
            .checked_mul(other.denominator.checked_div(left_divisor)?)?;
        Self::new(numerator, denominator)
    }

    /// 逆数．0のときはNone．
    #[must_use]
    pub fn checked_recip(&self) -> Option<Self> {
        Self::new(self.denominator, self.numerator)
    }

    /// 商．0で割るときはNone．
    #[must_use]
    pub fn checked_div(&self, other: &Self) -> Option<Self> {
        self.checked_mul(&other.checked_recip()?)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_integer() {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rational(numerator: i128, denominator: i128) -> Rational {
        Rational::new(numerator, denominator).expect("有効な有理数")
    }

    #[test]
    fn 約分して分母を正にする() {
        assert_eq!(rational(2, 4), rational(1, 2));
        assert_eq!(rational(1, -2), rational(-1, 2));
        assert_eq!(rational(0, -5), Rational::ZERO);
        assert_eq!(rational(6, 3).to_string(), "2");
        assert_eq!(rational(-3, 6).to_string(), "-1/2");
    }

    #[test]
    fn 分母が0のときは作れない() {
        assert_eq!(Rational::new(1, 0), None);
    }

    #[test]
    fn 四則演算をする() {
        let a = rational(1, 2);
        let b = rational(1, 3);
        assert_eq!(a.checked_add(&b), Some(rational(5, 6)));
        assert_eq!(a.checked_sub(&b), Some(rational(1, 6)));
        assert_eq!(a.checked_mul(&b), Some(rational(1, 6)));
        assert_eq!(a.checked_div(&b), Some(rational(3, 2)));
        assert_eq!(a.checked_div(&Rational::ZERO), None);
    }

    #[test]
    fn ゼロとの積はゼロになる() {
        assert_eq!(
            rational(3, 7).checked_mul(&Rational::ZERO),
            Some(Rational::ZERO)
        );
        assert_eq!(
            Rational::ZERO.checked_mul(&rational(3, 7)),
            Some(Rational::ZERO)
        );
    }

    #[test]
    fn あふれるときは値を返さない() {
        let big = Rational::from_integer(i128::MAX);
        assert_eq!(big.checked_add(&Rational::ONE), None);
        assert_eq!(big.checked_mul(&Rational::from_integer(2)), None);
        assert_eq!(Rational::from_integer(i128::MIN).checked_neg(), None);
    }

    #[test]
    fn 指数に使える整数か調べる() {
        assert_eq!(rational(3, 1).to_non_negative_u32(), Some(3));
        assert_eq!(rational(-1, 1).to_non_negative_u32(), None);
        assert_eq!(rational(1, 2).to_non_negative_u32(), None);
    }
}
