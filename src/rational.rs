use std::borrow::Borrow;

use crate::primes::PrimeFactorization;
use crate::primes::factorial;

lazy_static::lazy_static!(
    static ref SQRT_PI_F64: f64 = (std::f64::consts::PI).sqrt();
);

/// A rational number represented as two integer prime factorizations, one for
/// numerator and one for denominator. The sign of the fraction is stored in the
/// numerator.
#[derive(Debug, Clone)]
pub struct Rational {
    pub numerator: PrimeFactorization,
    pub denominator: PrimeFactorization,
    pub sqrt_pi: i32,
}

impl Rational {
    /// Create a new `Rational` with the given `numerator` and `denominator`
    pub fn new(mut numerator: PrimeFactorization, mut denominator: PrimeFactorization) -> Rational {
        numerator.sign *= denominator.sign;
        denominator.sign = 1;
        return Rational {
            numerator,
            denominator,
            sqrt_pi: 0,
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
        self.numerator.as_f64() / self.denominator.as_f64() * SQRT_PI_F64.powi(self.sqrt_pi)
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
        self.sqrt_pi += &rhs.sqrt_pi;
    }
}

impl std::ops::Mul for Rational {
    type Output = Rational;
    fn mul(mut self, rhs: Self) -> Self::Output {
        self *= &rhs;
        return self;
    }
}


impl<T> std::ops::DivAssign<T> for Rational where T: Borrow<Rational> {
    fn div_assign(&mut self, rhs: T) {
        let rhs = rhs.borrow();
        debug_assert_eq!(self.denominator.sign, 1);
        debug_assert_eq!(rhs.denominator.sign, 1);

        self.numerator.sign *= rhs.numerator.sign;

        self.numerator *= &rhs.denominator;
        self.denominator *= &rhs.numerator;
        self.denominator.sign = 1;
        self.sqrt_pi -= &rhs.sqrt_pi;
    }
}

impl std::ops::Div for Rational {
    type Output = Rational;
    fn div(mut self, rhs: Self) -> Self::Output {
        self /= &rhs;
        return self;
    }
}


/// Factorials of half-integer can be calculated by Gamma function,
/// ```text
/// n! = Gamma(n+1)
/// (n-1/2)! = Gamma(n-1+1+1/2) = Gamma(n+1/2) = (2n)! / (2^2n * n!) * sqrt(pi)
/// ```
pub fn factorial_half_integer(double_n: u32) -> Rational {
    if double_n % 2 == 0 {
        let n = double_n / 2;
        return Rational {
            numerator: factorial(n),
            denominator: PrimeFactorization::one(),
            sqrt_pi: 0,
        }
    }

    let n = double_n / 2 + 1;   // evaluate (n-1/2)!
    let numerator = factorial(2*n);           // (2n)!
    let mut denominator = factorial(n);       // n!
    if denominator.factors.is_empty() {
        denominator.factors.push(2 * (n as u16)); // special case for (1/2)!
    } else {
        denominator.factors[0] += 2 * (n as u16); // 2^2n
    }

    let mut ret = Rational {
        numerator,
        denominator,
        sqrt_pi: 1,
    };
    ret.simplify();
    return ret;
}


#[cfg(test)]
#[allow(clippy::redundant_clone)]
mod tests{
    use super::*;

    use approx::assert_ulps_eq;

    #[test]
    fn test_factorial_half_integer() {
        // integer case
        let n = 8;
        let ret = factorial_half_integer(n);
        let expect = factorial(n/2);
        assert_eq!(ret.numerator, expect);
        assert_eq!(ret.denominator, PrimeFactorization::one());
        assert_eq!(ret.sqrt_pi, 0);

        // half integer case, (1/2)! = sqrt_pi * 1/2
        let n = 1;
        let ret = factorial_half_integer(n).as_f64();
        let expect = *SQRT_PI_F64 / 2.0;
        assert_ulps_eq!(ret, expect);

        // half integer case, (3/2)! = sqrt_pi * 1/2 * 3/2 = sqrt_pi * 3 / 4
        let n = 3;
        let ret = factorial_half_integer(n).as_f64();
        let expect = *SQRT_PI_F64 * 3.0 / 4.0;
        assert_ulps_eq!(ret, expect);

        // half integer case, (3/2)! = sqrt_pi * 1/2 * 3/2 * 5/2 = sqrt_pi * 15 / 8
        let n = 5;
        let ret = factorial_half_integer(n).as_f64();
        let expect = *SQRT_PI_F64 * 15.0 / 8.0;
        assert_ulps_eq!(ret, expect);
    }
}
