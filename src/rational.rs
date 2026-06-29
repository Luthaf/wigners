use std::borrow::Borrow;

use num_bigint::BigInt;
use num_traits::Signed;
use num_traits::ToPrimitive;

use crate::primes::PrimeFactorization;

/// A rational number represented as two integer prime factorizations, one for
/// numerator and one for denominator. The sign of the fraction is stored in the
/// numerator.
#[derive(Debug, Clone)]
pub struct Rational {
    pub numerator: PrimeFactorization,
    pub denominator: PrimeFactorization,
}

impl Rational {
    /// Create a new `Rational` with the given `numerator` and `denominator`
    pub fn new(mut numerator: PrimeFactorization, mut denominator: PrimeFactorization) -> Rational {
        numerator.sign *= denominator.sign;
        denominator.sign = 1;
        return Rational {
            numerator,
            denominator,
        };
    }

    /// Divide both numerator and denominator by their greatest common divider
    /// in order to simplify the rational
    pub fn simplify(&mut self) {
        for (num_factor, den_factor) in self.numerator.factors.iter_mut().zip(self.denominator.factors.iter_mut()) {
            let gcd = std::cmp::min(*num_factor, *den_factor);
            *num_factor -= gcd;
            *den_factor -= gcd;
        }

        self.numerator.simplify_factors();
        self.denominator.simplify_factors();
    }

    /// Get the value of this `Rational` as a floating point value
    pub fn as_f64(&self) -> f64 {
        self.numerator.as_f64() / self.denominator.as_f64()
    }

    /// Get the signed root of this `Rational`, i.e. `sign(R) * sqrt(|R|)` where
    /// R is the rational.
    pub fn signed_root(&self) -> f64 {
        let value = self.as_f64();
        return value.signum() * value.abs().sqrt();
    }

    /// compute `factor * sqrt(|self|) * sign(self)` using log-space arithmetic
    /// to avoid overflow/underflow when factor and s individually exceed f64
    /// range.
    pub fn signed_sqrt(self, factor: &BigInt) -> f64 {
        if factor.sign() == num_bigint::Sign::NoSign {
            return 0.0;
        }

        let factor_sign = if factor.sign() == num_bigint::Sign::Minus { -1.0 } else { 1.0 };
        let s_sign = self.numerator.sign as f64;
        if s_sign == 0.0 {
            return 0.0;
        }

        // Compute ln(|factor|) using BigInt bits to avoid overflow
        let bits = factor.bits();
        let ln_factor = if bits <= 53 {
            (factor.to_f64().unwrap()).abs().ln()
        } else {
            let shift = bits - 53;
            let mantissa = (factor >> shift).abs().to_f64().unwrap();
            mantissa.ln() + (shift as f64) * std::f64::consts::LN_2
        };

        // Compute 0.5 * ln(|s|) from the prime factorization
        let mut ln_s = 0.0;
        let max_len = std::cmp::max(self.numerator.factors.len(), self.denominator.factors.len());
        for (i, prime) in crate::primes::primes().enumerate() {
            if i >= max_len {
                break;
            }
            let num_exp = self.numerator.factors.get(i).copied().unwrap_or(0) as f64;
            let den_exp = self.denominator.factors.get(i).copied().unwrap_or(0) as f64;
            ln_s += (num_exp - den_exp) * (prime as f64).ln();
        }

        let ln_result = ln_factor + 0.5 * ln_s;
        return factor_sign * s_sign * ln_result.exp();
    }
}

impl<T> std::ops::MulAssign<T> for Rational where T: Borrow<Rational> {
    fn mul_assign(&mut self, rhs: T) {
        let rhs = rhs.borrow();
        debug_assert_eq!(self.denominator.sign, 1);
        debug_assert_eq!(rhs.denominator.sign, 1);

        self.numerator.sign *= rhs.numerator.sign;

        self.numerator *= &rhs.numerator;
        self.denominator *= &rhs.denominator;
    }
}
