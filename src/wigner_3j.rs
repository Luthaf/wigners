use std::num::NonZeroUsize;

use parking_lot::Mutex;

use lru::LruCache;
use rayon::prelude::*;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::primes::{factorial, PrimeFactorization};
use crate::rational::Rational;


// cache up to that many wigner_3j symbols in a LRU cache. 200_000 entries is
// enough for our use case of computing all symbols up to `j_{1, 2, 3}=20`
const WIGNER_3J_CACHE_SIZE: usize = 200_000;

type Wigner3jCacheKey = (i32, i32, i32, i32, i32);
lazy_static::lazy_static!(
    static ref CACHED_WIGNER_3J: Mutex<LruCache<Wigner3jCacheKey, f64>> = Mutex::new(
        LruCache::new(NonZeroUsize::new(WIGNER_3J_CACHE_SIZE).expect("cache size is zero"))
    );
);

#[no_mangle]
pub extern "C" fn clear_wigner_3j_cache() {
    CACHED_WIGNER_3J.lock().clear();
}

/// Compute the Wigner 3j coefficient for the given `j1`, `j2`, `j2`, `m1`,
/// `m2`, `m3`.
#[no_mangle]
pub extern "C" fn wigner_3j(j1: u32, j2: u32, j3: u32, m1: i32, m2: i32, m3: i32) -> f64 {
    if m1.unsigned_abs() > j1 {
        panic!("invalid j1/m1 in wigner3j: {}/{}", j1, m1);
    } else if m2.unsigned_abs() > j2 {
        panic!("invalid j2/m2 in wigner3j: {}/{}", j2, m2);
    } else if m3.unsigned_abs() > j3 {
        panic!("invalid j3/m3 in wigner3j: {}/{}", j3, m3);
    }

    if !triangle_condition(j1, j2, j3) || m1 + m2 + m3 != 0 {
        return 0.0;
    }

    let (j1, j2, j3, m1, m2, _, mut sign) = reorder3j(j1, j2, j3, m1, m2, m3, 1.0);

    let total_j = j1 + j2 + j3;
    let alpha1 = j2 as i32 - m1 - j3 as i32;
    let alpha2 = j1 as i32 + m2 - j3 as i32;
    let beta1 = (j1 + j2 - j3) as i32;
    let beta2 = j1 as i32 - m1;
    let beta3 = j2 as i32 + m2;

    // extra sign in definition: alpha1 - alpha2 = j1 + m2 - j2 + m1 = j1 - j2 + m3
    if (alpha1 - alpha2) % 2 != 0 {
        sign = -sign;
    }

    {
        let mut cache = CACHED_WIGNER_3J.lock();
        if let Some(&cached_value) = cache.get(&(alpha1, alpha2, beta1, beta2, beta3)) {
            return sign * cached_value;
        }
    }

    let s1 = triangle_coefficient(j1, j2, j3);

    debug_assert!(beta2 >= 0);
    let mut s2 = factorial(beta2 as u32);

    debug_assert!((beta1 - alpha1) >= 0);
    s2 *= factorial((beta1 - alpha1) as u32);

    debug_assert!((beta1 - alpha2) >= 0);
    s2 *= factorial((beta1 - alpha2) as u32);

    debug_assert!(beta3 >= 0);
    s2 *= factorial(beta3 as u32);

    debug_assert!((beta3 - alpha1) >= 0);
    s2 *= factorial((beta3 - alpha1) as u32);

    debug_assert!((beta2 - alpha2) >= 0);
    s2 *= factorial((beta2 - alpha2) as u32);

    let (series_numerator, series_denominator) = compute_3j_series(total_j, beta1, beta2, beta3, alpha1, alpha2);

    let numerator = s1.numerator * s2;
    let mut s = Rational::new(numerator, s1.denominator);

    let series_denominator = Rational::new(PrimeFactorization::one(), series_denominator);

    // insert series denominator in the root, this improves precision compared
    // to immediately converting the full series to f64
    s *= &series_denominator;
    s *= &series_denominator;
    s.simplify();

    let result = series_numerator * s.signed_root();

    {
        let mut cache = CACHED_WIGNER_3J.lock();
        cache.put((alpha1, alpha2, beta1, beta2, beta3), result);
    }

    return sign * result;
}

/// Compute the Clebsch-Gordan coefficient <j1 m1 ; j2 m2 | j3 m3> using their
/// relation to Wigner 3j coefficients:
///
/// ```text
/// <j1 m1 ; j2 m2 | j3 m3> = (-1)^(j1 - j2 + m3) sqrt(2*j3 + 1) wigner_3j(j1, j2, j3, m1, m2, -m3)
/// ```
#[no_mangle]
pub extern "C" fn clebsch_gordan(j1: u32, m1: i32, j2: u32, m2: i32, j3: u32, m3: i32) -> f64 {
    let mut w3j = wigner_3j(j1, j2, j3, m1, m2, -m3);

    w3j *= f64::sqrt((2 * j3 + 1) as f64);
    if (j1 as i32 - j2 as i32 + m3) % 2 != 0 {
        return -w3j;
    } else {
        return w3j;
    }
}


/// Compute the full array of Clebsch-Gordan coefficients for the three given
/// `j`.
///
/// Data will be written to `output`, which can be interpreted as a row-major
/// 3-dimensional array with shape `(2 * j1 + 1, 2 * j2 + 1, 2 * j3 + 1)`.
pub fn clebsch_gordan_array(j1: u32, j2: u32, j3: u32, output: &mut [f64]) {
    let j1_size = 2 * j1 + 1;
    let j2_size = 2 * j2 + 1;
    let j3_size = 2 * j3 + 1;

    let size = (j1_size * j2_size * j3_size) as usize;
    if output.len() != size {
        panic!(
            "invalid output size, expected to have space for {} entries, but got {}",
            size, output.len()
        );
    }

    output.par_iter_mut().enumerate().for_each(|(i, o)| {
        let i = i as u32;
        let m1 = ((i / j3_size) / j2_size) as i32 - j1 as i32;
        let m2 = ((i / j3_size) % j2_size) as i32 - j2 as i32;
        let m3 = (i % j3_size) as i32 - j3 as i32;

        *o = clebsch_gordan(j1, m1, j2, m2, j3, m3);
    })
}

/// Same function as `clebsch_gordan_array`, but can be called directly from C
#[no_mangle]
pub unsafe extern "C" fn clebsch_gordan_array_c(j1: u32, j2: u32, j3: u32, data: *mut f64, len: u64) {
    let slice = std::slice::from_raw_parts_mut(data, len as usize);
    clebsch_gordan_array(j1, j2, j3, slice);
}

/// check the triangle condition on j1, j2, j3, i.e. `|j1 - j2| <= j3 <= j1 + j2`
pub(crate) fn triangle_condition(j1: u32, j2: u32, j3: u32) -> bool {
    return (j3 <= j1 + j2) && (j1 <= j2 + j3) && (j2 <= j3 + j1);
}

// reorder j1/m1, j2/m2, j3/m3 such that j1 >= j2 >= j3 and m1 >= 0 or m1 == 0 && m2 >= 0
fn reorder3j(j1: u32, j2: u32, j3: u32, m1: i32, m2: i32, m3: i32, mut sign: f64) -> (u32, u32, u32, i32, i32, i32, f64) {
    if j1 < j2 {
        return reorder3j(j2, j1, j3, m2, m1, m3, -sign);
    } else if j2 < j3 {
        return reorder3j(j1, j3, j2, m1, m3, m2, -sign);
    } else if m1 < 0 || (m1 == 0 && m2 < 0) {
        return reorder3j(j1, j2, j3, -m1, -m2, -m3, -sign);
    } else {
        // sign doesn't matter if total J = j1 + j2 + j3 is even
        if (j1 + j2 + j3) % 2 == 0 {
            sign = 1.0;
        }
        return (j1, j2, j3, m1, m2, m3, sign);
    }
}

fn triangle_coefficient(j1: u32, j2: u32, j3: u32) -> Rational {
    let n1 = factorial(j1 + j2 - j3);
    let n2 = factorial(j1 - j2 + j3);
    let n3 = factorial(j2 + j3 - j1);
    let numerator = n1 * n2 * n3;
    let denominator = factorial(j1 + j2 + j3 + 1);

    let mut result = Rational::new(numerator, denominator);
    result.simplify();
    return result;
}

pub(crate) fn max(a: i32, b: i32, c: i32) -> i32 {
    std::cmp::max(a, std::cmp::max(b, c))
}

pub(crate) fn min(a: i32, b: i32, c: i32) -> i32 {
    std::cmp::min(a, std::cmp::min(b, c))
}

/// compute the sum appearing in the 3j symbol
pub(crate) fn compute_3j_series(total_j: u32, beta1: i32, beta2: i32, beta3: i32, alpha1: i32, alpha2: i32) -> (f64, PrimeFactorization) {
    let range = max(alpha1, alpha2, 0)..(min(beta1, beta2, beta3) + 1);

    let mut numerators = Vec::with_capacity(range.len());
    let mut denominators = Vec::with_capacity(range.len());
    for k in range {
        let numerator = if k % 2 == 0 {
            PrimeFactorization::one()
        } else {
            PrimeFactorization::minus_one()
        };
        numerators.push(numerator);

        debug_assert!(k >= 0);
        let mut denominator = factorial(k as u32);

        debug_assert!((k - alpha1) >= 0);
        denominator *= factorial((k - alpha1) as u32);

        debug_assert!((k - alpha2) >= 0);
        denominator *= factorial((k - alpha2) as u32);

        debug_assert!((beta1 - k) >= 0);
        denominator *= factorial((beta1 - k) as u32);

        debug_assert!((beta2 - k) >= 0);
        denominator *= factorial((beta2 - k) as u32);

        debug_assert!((beta3 - k) >= 0);
        denominator *= factorial((beta3 - k) as u32);

        denominators.push(denominator);
    }

    let denominator = common_denominator(&mut numerators, &denominators);

    let numerator = if total_j > 100 {
        // For large total J, we will overflow f64 in this sum, but performing
        // the sum with big integers is enough to recover the full precision
        let mut numerator = BigInt::from(0);
        for num in numerators {
            numerator += num.as_bigint();
        }
        numerator.to_f64().expect("not a f64")
    } else {
        let mut numerator = 0.0;
        for num in numerators {
            numerator += num.as_f64();
        }
        numerator
    };

    return (numerator, denominator);
}

/// Given a list of numerators and denominators, compute the common denominator
/// and the rescaled numerator, putting all fractions at the same common
/// denominator
pub(crate) fn common_denominator(
    numerators: &mut [PrimeFactorization],
    denominators: &[PrimeFactorization]
) -> PrimeFactorization {
    debug_assert_eq!(numerators.len(), denominators.len());
    if denominators.is_empty() {
        return PrimeFactorization::one()
    }

    let mut denominator = denominators[0].clone();
    for other in denominators.iter().skip(1) {
        denominator.least_common_multiple(other);
    }

    // rescale numerators
    for (num, den) in numerators.iter_mut().zip(denominators.iter()) {
        *num *= &denominator;
        *num /= den;
    }

    return denominator;
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_ulps_eq;

    #[test]
    fn test_wigner3j() {
        // checked against sympy
        assert_ulps_eq!(wigner_3j(2, 6, 4, 0, 0, 1), 0.0);
        assert_ulps_eq!(wigner_3j(2, 6, 4, 0, 0, 0), f64::sqrt(715.0) / 143.0);
        assert_ulps_eq!(wigner_3j(5, 3, 2, -3, 3, 0), f64::sqrt(330.0) / 165.0);
        assert_ulps_eq!(wigner_3j(5, 3, 2, -2, 3, -1), -f64::sqrt(330.0) / 330.0);
        assert_ulps_eq!(wigner_3j(100, 100, 100, 100, -100, 0), 2.689688852311291e-13);

        assert_ulps_eq!(wigner_3j(0, 1, 1, 0, 0, 0), -0.5773502691896257);

        // https://github.com/Luthaf/wigners/issues/7
        assert_ulps_eq!(wigner_3j(100, 300, 285, 2, -2, 0), 0.001979165708981953);
    }

    #[test]
    fn test_clebsch_gordan() {
        // checked against sympy
        assert_ulps_eq!(clebsch_gordan(2, 0, 6, 0, 4, 1), 0.0);
        assert_ulps_eq!(clebsch_gordan(1, 1, 1, 1, 2, 2), 1.0);
        assert_ulps_eq!(clebsch_gordan(2, 2, 1, -1, 3, 1), f64::sqrt(1.0 / 15.0));
    }
}
