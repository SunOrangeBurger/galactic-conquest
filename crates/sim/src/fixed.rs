//! Fixed-point values with three decimal places, as required by spec v0.4.
//! Arithmetic is integer-only; division truncates toward zero.

use serde::{Deserialize, Serialize};
use std::fmt;

pub const SCALE: i64 = 1_000;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Fixed(i64);

impl Fixed {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(SCALE);

    pub const fn from_milli(milli: i64) -> Self {
        Self(milli)
    }

    pub const fn milli(self) -> i64 {
        self.0
    }

    pub fn from_integer(value: i64) -> Option<Self> {
        value.checked_mul(SCALE).map(Self)
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    /// Multiplies two fixed-point values, truncating once at milli precision.
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        let result = i128::from(self.0) * i128::from(other.0) / i128::from(SCALE);
        i64::try_from(result).ok().map(Self)
    }

    /// Computes `(self * numerator) / denominator`, truncating toward zero.
    pub fn checked_scale_ratio(self, numerator: i64, denominator: i64) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        let result = i128::from(self.0) * i128::from(numerator) / i128::from(denominator);
        i64::try_from(result).ok().map(Self)
    }
}

impl fmt::Display for Fixed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let magnitude = self.0.unsigned_abs();
        let sign = if self.0 < 0 { "-" } else { "" };
        let whole = magnitude / SCALE as u64;
        let fraction = magnitude % SCALE as u64;
        write!(formatter, "{sign}{whole}.{fraction:03}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_multiply_and_ratio_truncate_toward_zero() {
        let third = Fixed::from_milli(333);
        assert_eq!(third.checked_mul(third).unwrap().milli(), 110);
        assert_eq!(
            Fixed::from_milli(-5)
                .checked_scale_ratio(1, 2)
                .unwrap()
                .milli(),
            -2
        );
    }

    #[test]
    fn fixed_arithmetic_reports_overflow_and_division_by_zero() {
        assert!(Fixed::from_milli(i64::MAX)
            .checked_add(Fixed::ONE)
            .is_none());
        assert!(Fixed::ONE.checked_scale_ratio(1, 0).is_none());
        assert!(Fixed::from_integer(i64::MAX).is_none());
    }

    #[test]
    fn display_is_fixed_to_three_decimal_places() {
        assert_eq!(Fixed::from_milli(12_034).to_string(), "12.034");
        assert_eq!(Fixed::from_milli(-12_034).to_string(), "-12.034");
    }
}
