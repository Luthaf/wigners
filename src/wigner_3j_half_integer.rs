use std::num::NonZeroUsize;

use parking_lot::Mutex;

use lru::LruCache;
use num_traits::{PrimInt, CheckedRem};
use rayon::prelude::*;

use crate::primes::{factorial, PrimeFactorization};
use crate::rational::{factorial_half_integer, Rational};
use crate::wigner_3j::{
    triangle_condition,
    compute_3j_series,
};


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
pub extern "C" fn clear_wigner_3j_cache_half_integer() {
    CACHED_WIGNER_3J.lock().clear();
}

/// Compute the Wigner 3j coefficient for the given `j1`, `j2`, `j2`, `m1`,
/// `m2`, `m3`.
#[no_mangle]
pub extern "C" fn wigner_3j_half_integer(j1: u32, j2: u32, j3: u32, m1: i32, m2: i32, m3: i32) -> f64 {
    if !check_jm(j1, m1) {
        return 0.0;
        //panic!("invalid j1/m1 in wigner3j: {} -- {}, |m|<=j j±m should be integer",
            //print_half_integer(j1), print_half_integer(m1));
    } else if !check_jm(j2, m2) {
        return 0.0;
        //panic!("invalid j2/m2 in wigner3j: {} -- {}, |m|<=j j±m should be integer",
            //print_half_integer(j2), print_half_integer(m2));
    } else if !check_jm(j3, m3) {
        return 0.0;
        //panic!("invalid j3/m3 in wigner3j: {} -- {}, |m|<=j j±m should be integer",
            //print_half_integer(j3), print_half_integer(m3));
    }

    if !triangle_condition(j1, j2, j3) || m1 + m2 + m3 != 0 {
        return 0.0;
    }

    let (j1, j2, j3, m1, m2, _, mut sign) = reorder3j_half_integer(j1, j2, j3, m1, m2, m3, 1.0);

    let total_j = j1 + j2 + j3;

    // according to https://github.com/Jutho/WignerSymbols.jl/blob/master/src/WignerSymbols.jl#L93-L97
    // alpha1, alpha2, beta1, beta2, beta3 are guranteed to be integers
    let alpha1 = j2 as i32 - m1 - j3 as i32;
    assert_eq!(alpha1 % 2, 0);
    let alpha1 = alpha1 / 2;

    let alpha2 = j1 as i32 + m2 - j3 as i32;
    assert_eq!(alpha2 % 2, 0);
    let alpha2 = alpha2 / 2;

    let beta1 = (j1 + j2 - j3) as i32;
    assert_eq!(beta1 % 2, 0);
    let beta1 = beta1 / 2;

    let beta2 = j1 as i32 - m1;
    assert_eq!(beta2 % 2, 0);
    let beta2 = beta2 / 2;

    let beta3 = j2 as i32 + m2;
    assert_eq!(beta3 % 2, 0);
    let beta3 = beta3 / 2;


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

    let s1 = triangle_coefficient_half_integer(j1, j2, j3);

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

    let (series_numerator, series_denominator) = compute_3j_series(total_j/2, beta1, beta2, beta3, alpha1, alpha2);

    let mut s = Rational::new(s1.numerator * s2, s1.denominator);

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
pub extern "C" fn clebsch_gordan_half_integer(j1: u32, m1: i32, j2: u32, m2: i32, j3: u32, m3: i32) -> f64 {
    let mut w3j = wigner_3j_half_integer(j1, j2, j3, m1, m2, -m3);

    w3j *= f64::sqrt((j3 + 1) as f64);
    if (j1 as i32 - j2 as i32 + m3) % 4 != 0 {
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
pub fn clebsch_gordan_array_half_integer(j1: u32, j2: u32, j3: u32, output: &mut [f64]) {
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

        *o = clebsch_gordan_half_integer(j1, m1, j2, m2, j3, m3);
    })
}

/// Same function as `clebsch_gordan_array`, but can be called directly from C
#[no_mangle]
pub unsafe extern "C" fn clebsch_gordan_array_c_half_integer(j1: u32, j2: u32, j3: u32, data: *mut f64, len: u64) {
    let slice = std::slice::from_raw_parts_mut(data, len as usize);
    clebsch_gordan_array_half_integer(j1, j2, j3, slice);
}

// reorder j1/m1, j2/m2, j3/m3 such that j1 >= j2 >= j3 and m1 >= 0 or m1 == 0 && m2 >= 0
fn reorder3j_half_integer(j1: u32, j2: u32, j3: u32, m1: i32, m2: i32, m3: i32, mut sign: f64) -> (u32, u32, u32, i32, i32, i32, f64) {
    if j1 < j2 {
        return reorder3j_half_integer(j2, j1, j3, m2, m1, m3, -sign);
    } else if j2 < j3 {
        return reorder3j_half_integer(j1, j3, j2, m1, m3, m2, -sign);
    } else if m1 < 0 || (m1 == 0 && m2 < 0) {
        return reorder3j_half_integer(j1, j2, j3, -m1, -m2, -m3, -sign);
    } else {
        // sign doesn't matter if total J = j1 + j2 + j3 is even
        if (j1 + j2 + j3) % 4 == 0 {
            sign = 1.0;
        }
        return (j1, j2, j3, m1, m2, m3, sign);
    }
}

fn triangle_coefficient_half_integer(j1: u32, j2: u32, j3: u32) -> Rational {
    let n1 = factorial_half_integer(j1 + j2 - j3);
    let n2 = factorial_half_integer(j1 - j2 + j3);
    let n3 = factorial_half_integer(j2 + j3 - j1);
    let numerator = n1 * n2 * n3;
    let denominator = factorial_half_integer(j1 + j2 + j3 + 2);

    let mut result = numerator / denominator;
    result.simplify();
    return result;
}

fn check_jm(j: u32, m: i32) -> bool {
    return (m.unsigned_abs() <= j) &&
            isinteger(j as i32 - m) &&
            isinteger(j as i32 + m);

    fn isinteger(n: i32) -> bool {
        return n % 2 == 0;
    }
}

// fn print_half_integer<T>(n: T) -> String
// where T: PrimInt + CheckedRem + std::fmt::Display
// {
//     let two = T::one() + T::one();
//     if n % (T::one() + T::one()) == T::zero() {
//         return format!("{}", n / two);
//     } else {
//         return format!("{}/2", n);
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_ulps_eq;

    #[test]
    fn test_wigner3j_integer() {
        // checked against sympy
        //assert_ulps_eq!(wigner_4j_half_integer(4, 12, 8, 0, 0, 2), 0.0);  // check
        assert_ulps_eq!(wigner_3j_half_integer(4, 12, 8, 0, 0, 0), f64::sqrt(715.0) / 143.0);
        assert_ulps_eq!(wigner_3j_half_integer(10, 6, 4, -6, 6, 0), f64::sqrt(330.0) / 165.0);
        assert_ulps_eq!(wigner_3j_half_integer(10, 6, 4, -4, 6, -2), -f64::sqrt(330.0) / 330.0);
        assert_ulps_eq!(wigner_3j_half_integer(200, 200, 200, 200, -200, 0), 2.689688852311291e-13);

        assert_ulps_eq!(wigner_3j_half_integer(0, 2, 2, 0, 0, 0), -0.5773502691896257);

        // https://github.com/Luthaf/wigners/issues/7
        assert_ulps_eq!(wigner_3j_half_integer(200, 600, 570, 4, -4, 0), 0.001979165708981953);
    }

    #[test]
    fn test_wigner3j_half_integer() {
        // checked against sympy
        assert_ulps_eq!(wigner_3j_half_integer(2, 6, 4, 0, 0, 1), 0.0);
        assert_ulps_eq!(wigner_3j_half_integer(2, 6, 4, 0, 0, 0), -f64::sqrt(105.0) / 35.0);
        assert_ulps_eq!(wigner_3j_half_integer(5, 3, 2, -3, 3, 0), f64::sqrt(15.0) / 15.0);
        assert_ulps_eq!(wigner_3j_half_integer(5, 3, 2, -2, 3, -1), 0.0);
        assert_ulps_eq!(wigner_3j_half_integer(100, 100, 100, 100, -100, 0), 1.8219272830228477e-7);

        assert_ulps_eq!(wigner_3j_half_integer(0, 2, 2, 0, 0, 0), -0.5773502691896257);

        // https://github.com/Luthaf/wigners/issues/7
        assert_ulps_eq!(wigner_3j_half_integer(100, 300, 285, 2, -2, 0), 0.0);
        // Here I updated the test, since w3j(50, 150, 285/2, 1, -1, 0) is invalid.
        assert_ulps_eq!(wigner_3j_half_integer(101, 300, 285, 1, -2, 1), -0.0028951194712330303);
    }

    #[test]
    fn test_clebsch_gordan_integer() {
        // checked against sympy
        assert_ulps_eq!(clebsch_gordan_half_integer(4, 0, 12, 0, 8, 2), 0.0);
        assert_ulps_eq!(clebsch_gordan_half_integer(2, 2, 2, 2, 4, 4), 1.0);
        assert_ulps_eq!(clebsch_gordan_half_integer(4, 4, 2, -2, 6, 2), f64::sqrt(1.0 / 15.0));
    }

    #[test]
    fn test_clebsch_gordan_half_integer() {
        // checked against sympy
        assert_ulps_eq!(clebsch_gordan_half_integer(2, 0, 6, 0, 4, 1), 0.0);
        assert_ulps_eq!(clebsch_gordan_half_integer(1, 1, 1, 1, 2, 2), 1.0);
        assert_ulps_eq!(clebsch_gordan_half_integer(2, 2, 1, -1, 3, 1), f64::sqrt(3.0)/3.0);
    }
}
