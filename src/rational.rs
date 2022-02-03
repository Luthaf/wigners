use std::borrow::Borrow;

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
