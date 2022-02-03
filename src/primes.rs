use std::borrow::Borrow;
use std::cmp::Ordering;
use std::sync::RwLock;

use smallvec::SmallVec;

/// Store the first 1000 primes, to be used for fast prime decomposition of
/// factorial
static FIRST_PRIMES: [u32; 1000] = [
       2,    3,    5,    7,   11,   13,   17,   19,   23,   29,
      31,   37,   41,   43,   47,   53,   59,   61,   67,   71,
      73,   79,   83,   89,   97,  101,  103,  107,  109,  113,
     127,  131,  137,  139,  149,  151,  157,  163,  167,  173,
     179,  181,  191,  193,  197,  199,  211,  223,  227,  229,
     233,  239,  241,  251,  257,  263,  269,  271,  277,  281,
     283,  293,  307,  311,  313,  317,  331,  337,  347,  349,
     353,  359,  367,  373,  379,  383,  389,  397,  401,  409,
     419,  421,  431,  433,  439,  443,  449,  457,  461,  463,
     467,  479,  487,  491,  499,  503,  509,  521,  523,  541,
     547,  557,  563,  569,  571,  577,  587,  593,  599,  601,
     607,  613,  617,  619,  631,  641,  643,  647,  653,  659,
     661,  673,  677,  683,  691,  701,  709,  719,  727,  733,
     739,  743,  751,  757,  761,  769,  773,  787,  797,  809,
     811,  821,  823,  827,  829,  839,  853,  857,  859,  863,
     877,  881,  883,  887,  907,  911,  919,  929,  937,  941,
     947,  953,  967,  971,  977,  983,  991,  997, 1009, 1013,
    1019, 1021, 1031, 1033, 1039, 1049, 1051, 1061, 1063, 1069,
    1087, 1091, 1093, 1097, 1103, 1109, 1117, 1123, 1129, 1151,
    1153, 1163, 1171, 1181, 1187, 1193, 1201, 1213, 1217, 1223,
    1229, 1231, 1237, 1249, 1259, 1277, 1279, 1283, 1289, 1291,
    1297, 1301, 1303, 1307, 1319, 1321, 1327, 1361, 1367, 1373,
    1381, 1399, 1409, 1423, 1427, 1429, 1433, 1439, 1447, 1451,
    1453, 1459, 1471, 1481, 1483, 1487, 1489, 1493, 1499, 1511,
    1523, 1531, 1543, 1549, 1553, 1559, 1567, 1571, 1579, 1583,
    1597, 1601, 1607, 1609, 1613, 1619, 1621, 1627, 1637, 1657,
    1663, 1667, 1669, 1693, 1697, 1699, 1709, 1721, 1723, 1733,
    1741, 1747, 1753, 1759, 1777, 1783, 1787, 1789, 1801, 1811,
    1823, 1831, 1847, 1861, 1867, 1871, 1873, 1877, 1879, 1889,
    1901, 1907, 1913, 1931, 1933, 1949, 1951, 1973, 1979, 1987,
    1993, 1997, 1999, 2003, 2011, 2017, 2027, 2029, 2039, 2053,
    2063, 2069, 2081, 2083, 2087, 2089, 2099, 2111, 2113, 2129,
    2131, 2137, 2141, 2143, 2153, 2161, 2179, 2203, 2207, 2213,
    2221, 2237, 2239, 2243, 2251, 2267, 2269, 2273, 2281, 2287,
    2293, 2297, 2309, 2311, 2333, 2339, 2341, 2347, 2351, 2357,
    2371, 2377, 2381, 2383, 2389, 2393, 2399, 2411, 2417, 2423,
    2437, 2441, 2447, 2459, 2467, 2473, 2477, 2503, 2521, 2531,
    2539, 2543, 2549, 2551, 2557, 2579, 2591, 2593, 2609, 2617,
    2621, 2633, 2647, 2657, 2659, 2663, 2671, 2677, 2683, 2687,
    2689, 2693, 2699, 2707, 2711, 2713, 2719, 2729, 2731, 2741,
    2749, 2753, 2767, 2777, 2789, 2791, 2797, 2801, 2803, 2819,
    2833, 2837, 2843, 2851, 2857, 2861, 2879, 2887, 2897, 2903,
    2909, 2917, 2927, 2939, 2953, 2957, 2963, 2969, 2971, 2999,
    3001, 3011, 3019, 3023, 3037, 3041, 3049, 3061, 3067, 3079,
    3083, 3089, 3109, 3119, 3121, 3137, 3163, 3167, 3169, 3181,
    3187, 3191, 3203, 3209, 3217, 3221, 3229, 3251, 3253, 3257,
    3259, 3271, 3299, 3301, 3307, 3313, 3319, 3323, 3329, 3331,
    3343, 3347, 3359, 3361, 3371, 3373, 3389, 3391, 3407, 3413,
    3433, 3449, 3457, 3461, 3463, 3467, 3469, 3491, 3499, 3511,
    3517, 3527, 3529, 3533, 3539, 3541, 3547, 3557, 3559, 3571,
    3581, 3583, 3593, 3607, 3613, 3617, 3623, 3631, 3637, 3643,
    3659, 3671, 3673, 3677, 3691, 3697, 3701, 3709, 3719, 3727,
    3733, 3739, 3761, 3767, 3769, 3779, 3793, 3797, 3803, 3821,
    3823, 3833, 3847, 3851, 3853, 3863, 3877, 3881, 3889, 3907,
    3911, 3917, 3919, 3923, 3929, 3931, 3943, 3947, 3967, 3989,
    4001, 4003, 4007, 4013, 4019, 4021, 4027, 4049, 4051, 4057,
    4073, 4079, 4091, 4093, 4099, 4111, 4127, 4129, 4133, 4139,
    4153, 4157, 4159, 4177, 4201, 4211, 4217, 4219, 4229, 4231,
    4241, 4243, 4253, 4259, 4261, 4271, 4273, 4283, 4289, 4297,
    4327, 4337, 4339, 4349, 4357, 4363, 4373, 4391, 4397, 4409,
    4421, 4423, 4441, 4447, 4451, 4457, 4463, 4481, 4483, 4493,
    4507, 4513, 4517, 4519, 4523, 4547, 4549, 4561, 4567, 4583,
    4591, 4597, 4603, 4621, 4637, 4639, 4643, 4649, 4651, 4657,
    4663, 4673, 4679, 4691, 4703, 4721, 4723, 4729, 4733, 4751,
    4759, 4783, 4787, 4789, 4793, 4799, 4801, 4813, 4817, 4831,
    4861, 4871, 4877, 4889, 4903, 4909, 4919, 4931, 4933, 4937,
    4943, 4951, 4957, 4967, 4969, 4973, 4987, 4993, 4999, 5003,
    5009, 5011, 5021, 5023, 5039, 5051, 5059, 5077, 5081, 5087,
    5099, 5101, 5107, 5113, 5119, 5147, 5153, 5167, 5171, 5179,
    5189, 5197, 5209, 5227, 5231, 5233, 5237, 5261, 5273, 5279,
    5281, 5297, 5303, 5309, 5323, 5333, 5347, 5351, 5381, 5387,
    5393, 5399, 5407, 5413, 5417, 5419, 5431, 5437, 5441, 5443,
    5449, 5471, 5477, 5479, 5483, 5501, 5503, 5507, 5519, 5521,
    5527, 5531, 5557, 5563, 5569, 5573, 5581, 5591, 5623, 5639,
    5641, 5647, 5651, 5653, 5657, 5659, 5669, 5683, 5689, 5693,
    5701, 5711, 5717, 5737, 5741, 5743, 5749, 5779, 5783, 5791,
    5801, 5807, 5813, 5821, 5827, 5839, 5843, 5849, 5851, 5857,
    5861, 5867, 5869, 5879, 5881, 5897, 5903, 5923, 5927, 5939,
    5953, 5981, 5987, 6007, 6011, 6029, 6037, 6043, 6047, 6053,
    6067, 6073, 6079, 6089, 6091, 6101, 6113, 6121, 6131, 6133,
    6143, 6151, 6163, 6173, 6197, 6199, 6203, 6211, 6217, 6221,
    6229, 6247, 6257, 6263, 6269, 6271, 6277, 6287, 6299, 6301,
    6311, 6317, 6323, 6329, 6337, 6343, 6353, 6359, 6361, 6367,
    6373, 6379, 6389, 6397, 6421, 6427, 6449, 6451, 6469, 6473,
    6481, 6491, 6521, 6529, 6547, 6551, 6553, 6563, 6569, 6571,
    6577, 6581, 6599, 6607, 6619, 6637, 6653, 6659, 6661, 6673,
    6679, 6689, 6691, 6701, 6703, 6709, 6719, 6733, 6737, 6761,
    6763, 6779, 6781, 6791, 6793, 6803, 6823, 6827, 6829, 6833,
    6841, 6857, 6863, 6869, 6871, 6883, 6899, 6907, 6911, 6917,
    6947, 6949, 6959, 6961, 6967, 6971, 6977, 6983, 6991, 6997,
    7001, 7013, 7019, 7027, 7039, 7043, 7057, 7069, 7079, 7103,
    7109, 7121, 7127, 7129, 7151, 7159, 7177, 7187, 7193, 7207,
    7211, 7213, 7219, 7229, 7237, 7243, 7247, 7253, 7283, 7297,
    7307, 7309, 7321, 7331, 7333, 7349, 7351, 7369, 7393, 7411,
    7417, 7433, 7451, 7457, 7459, 7477, 7481, 7487, 7489, 7499,
    7507, 7517, 7523, 7529, 7537, 7541, 7547, 7549, 7559, 7561,
    7573, 7577, 7583, 7589, 7591, 7603, 7607, 7621, 7639, 7643,
    7649, 7669, 7673, 7681, 7687, 7691, 7699, 7703, 7717, 7723,
    7727, 7741, 7753, 7757, 7759, 7789, 7793, 7817, 7823, 7829,
    7841, 7853, 7867, 7873, 7877, 7879, 7883, 7901, 7907, 7919,
];

/// A container for prime numbers. The first 1000 primes are stored in a static
/// table, extra values are computed on the fly when required.
struct Primes {
    extra_primes: RwLock<Vec<u32>>,
}

impl Primes {
    /// Create a new `Primes` instance. It is able to give any prime below 7919
    /// (1000-th prime) in O(1) time, and computes & cache higher values
    fn new() -> Primes {
        return Primes {
            extra_primes: RwLock::new(vec![7927]),
        }
    }

    /// Get the `nth` prime
    fn get(&self, nth: usize) -> u32 {
        if nth < FIRST_PRIMES.len() {
            return FIRST_PRIMES[nth]
        }

        let nth = nth - FIRST_PRIMES.len();
        let data = self.extra_primes.read().expect("poisoned lock");
        if let Some(&value) = data.get(nth) {
            return value;
        }

        std::mem::drop(data);
        // get exclusive access to the data
        let mut data = self.extra_primes.write().expect("poisoned lock");

        while data.len() < (nth + 1) {
            let is_prime = |value| {
                for known_prime in FIRST_PRIMES.iter().chain(data.iter()) {
                    if value % known_prime == 0 {
                        return false;
                    }
                }
                return true;
            };

            let mut p = data.last().expect("empty prime table") + 2;
            while !is_prime(p) {
                p += 2;
            }
            data.push(p);
        }

        return *data.get(nth).expect("missing last prime");
    }
}

lazy_static::lazy_static!(
    /// Single global instance of the prime list
    static ref PRIMES: Primes = Primes::new();
);

/// Iterator over the global prime list
struct PrimeIter {
    next: usize
}

/// Get an iterator over prime numbers. This iterator have infinite size!
fn primes() -> PrimeIter {
    return PrimeIter { next: 0 }
}

impl std::iter::Iterator for PrimeIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let prime = PRIMES.get(self.next);
        self.next += 1;
        return Some(prime);
    }
}

/// `PrimeFactorization` represents the prime factorization of an arbitrary
/// large integer, even if this integer would overflow native machine integers
/// (i32/i64).
#[derive(Clone, PartialEq)]
pub struct PrimeFactorization {
    /// sign of the integer, stored as 1/0/-1
    pub(crate) sign: i8,
    /// factors of the factorization. The represented integer is `self.sign *
    /// \Pi_i prime_i ^ self.factors[i]`
    pub(crate) factors: SmallVec<[u16; 16]>,
}

impl std::fmt::Debug for PrimeFactorization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.sign == 0 {
            return write!(f, "0");
        } else if self.sign == 1 {
            write!(f, "+ ")?;
        } else {
            debug_assert_eq!(self.sign, -1);
            write!(f, "- ")?;
        }

        // special case ±1
        if self.factors.len() == 1 && self.factors[0] == 0 {
            return write!(f, "1");
        }

        let mut factors = Vec::new();
        for (prime, factor) in primes().zip(self.factors.iter().cloned()) {
            if factor != 0 {
                factors.push(format!("{}^{}", prime, factor));
            }
        }

        return write!(f, "{}", factors.join(" x "));
    }
}

impl PrimeFactorization {
    /// Compute the prime factorization of the integer `n`
    fn new(n: i32) -> PrimeFactorization {
        let sign = match n.cmp(&0) {
            Ordering::Equal => {
                return PrimeFactorization {
                    factors: SmallVec::new(),
                    sign: 0,
                }
            }
            Ordering::Greater => 1,
            Ordering::Less => -1,
        };

        let mut value = n.abs() as u32;
        let mut factors = SmallVec::new();
        for prime in primes() {
            let mut factor = 0;

            let mut value_next = value / prime;
            let mut remainder = value % prime;
            while remainder == 0 {
                factor += 1;
                value = value_next;

                value_next = value / prime;
                remainder = value % prime;
            }

            factors.push(factor);

            if value == 1 {
                break;
            }
        }

        return PrimeFactorization {
            sign, factors
        };
    }

    /// Get the prime factorization of 1
    pub fn one() -> PrimeFactorization {
        PrimeFactorization::new(1)
    }

    /// Get the prime factorization of -1
    pub fn minus_one() -> PrimeFactorization {
        PrimeFactorization::new(-1)
    }

    /// trim factors to contain no trailing zeros, except if we are representing
    /// the number -1 or 1, in which case the factors will contain exactly one
    /// zero.
    pub(crate) fn simplify_factors(&mut self) {
        if self.sign == 0 {
            return;
        }

        while let Some(0) = self.factors.last() {
            self.factors.pop();
        }

        if self.factors.is_empty() {
            self.factors.push(0);
        }
    }

    /// Set this PrimeFactorization to the least common multiple of itself and
    /// `other`
    pub fn least_common_multiple(&mut self, other: &PrimeFactorization) {
        self.sign *= other.sign;
        if other.factors.len() > self.factors.len() {
            self.factors.resize(other.factors.len(), 0);
        }

        for (self_factor, &other_factor) in self.factors.iter_mut().zip(&other.factors) {
            *self_factor = std::cmp::max(*self_factor, other_factor);
        }
    }

    /// Get the value of this prime factorization as a floating point number
    pub fn as_f64(&self) -> f64 {
        let mut result = self.sign as f64;
        for (prime, &power) in primes().map(|p| p as f64).zip(&self.factors) {
            result *= prime.powi(power as i32);
        }
        return result;
    }
}

impl<T> std::ops::MulAssign<T> for PrimeFactorization where T: Borrow<PrimeFactorization> {
    fn mul_assign(&mut self, rhs: T) {
        let rhs = rhs.borrow();
        self.sign *= rhs.sign;

        if self.sign == 0 {
            self.factors.clear();
            return;
        }

        if self.factors.len() < rhs.factors.len() {
            self.factors.resize(rhs.factors.len(), 0)
        }

        for (factor, &rhs_factor) in self.factors.iter_mut().zip(&rhs.factors) {
            *factor += rhs_factor;
        }
    }
}

impl std::ops::Mul for PrimeFactorization {
    type Output = PrimeFactorization;

    fn mul(mut self, rhs: Self) -> Self::Output {
        self *= &rhs;
        return self;
    }
}

impl<T> std::ops::DivAssign<T> for PrimeFactorization where T: Borrow<PrimeFactorization> {
    fn div_assign(&mut self, rhs: T) {
        let rhs = rhs.borrow();
        if rhs.sign == 0 {
            panic!("attempt to divide by zero")
        }

        if self.sign == 0 {
            return;
        }

        self.sign *= rhs.sign;
        if self.factors.len() < rhs.factors.len() {
            self.factors.resize(rhs.factors.len(), 0)
        }

        for (factor, &rhs_factor) in self.factors.iter_mut().zip(&rhs.factors) {
            if rhs_factor > *factor {
                panic!("can not divide if the factorization do not have common prime factor");
            }
            *factor -= rhs_factor;
        }

        self.simplify_factors();
    }
}

impl std::ops::Div for PrimeFactorization {
    type Output = PrimeFactorization;

    fn div(mut self, rhs: Self) -> Self::Output {
        self /= rhs;
        return self;
    }
}

/// How many pre-computed factorials should we cache
const FACTORIAL_CACHE_SIZE: usize = 100;

lazy_static::lazy_static! {
    static ref FACTORIAL_TABLE: Vec<PrimeFactorization> = {
        let mut table = Vec::new();
        for n in 0..FACTORIAL_CACHE_SIZE {
            table.push(compute_factorial(n as u32));
        }
        table
    };
}

/// Compute the factorial of the integer `n` as a prime factorization
pub fn factorial(n: u32) -> PrimeFactorization {
    if (n as usize) < FACTORIAL_CACHE_SIZE {
        return FACTORIAL_TABLE[n as usize].clone();
    } else {
        return compute_factorial(n);
    }
}

fn compute_factorial(n : u32) -> PrimeFactorization {
    // inspired by https://janmr.com/blog/2010/10/prime-factors-of-factorial-numbers/
    let mut factors = SmallVec::new();
    for prime in primes() {
        if prime > n {
            break;
        }

        let mut factor = 0;
        let mut prime_pow = prime;
        loop {
            if prime_pow > n {
                assert!(factor <= u16::MAX as u32, "factorial requires a prime factor too big for this implementation");
                // we will find no more factors for this prime
                factors.push(factor as u16);
                break;
            }

            factor += n / prime_pow;
            prime_pow *= prime;
        }
    }

    return PrimeFactorization {
        sign: 1,
        factors: factors
    };
}

#[cfg(test)]
#[allow(clippy::redundant_clone)]
mod tests {
    use super::*;

    #[test]
    fn prime_factorize() {
        let zero = PrimeFactorization::new(0);
        assert_eq!(zero.sign, 0);
        assert_eq!(zero.factors.len(), 0);

        let one = PrimeFactorization::new(1);
        assert_eq!(one.sign, 1);
        assert_eq!(one.factors.len(), 1);
        assert_eq!(one.factors[0], 0);

        let m_one = PrimeFactorization::new(-1);
        assert_eq!(m_one.sign, -1);
        assert_eq!(m_one.factors[0], 0);

        let five = PrimeFactorization::new(5);
        assert_eq!(five.sign, 1);
        assert_eq!(five.factors.len(), 3);
        assert_eq!(five.factors.as_slice(), [0, 0, 1]);

        let m_twenty = PrimeFactorization::new(-20);
        assert_eq!(m_twenty.sign, -1);
        assert_eq!(m_twenty.factors.len(), 3);
        assert_eq!(m_twenty.factors.as_slice(), [2, 0, 1]);

        // outside of the initial range of primes
        assert_eq!(PRIMES.extra_primes.read().unwrap().len(), 1);
        let seventeen = PrimeFactorization::new(7949);
        assert_eq!(seventeen.sign, 1);
        assert_eq!(seventeen.factors.len(), 1004);
        // we now have more extra primes
        assert_eq!(PRIMES.extra_primes.read().unwrap().len(), 4);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn as_f64() {
        assert_eq!(PrimeFactorization::new(0).as_f64(), 0.0);
        assert_eq!(PrimeFactorization::new(1).as_f64(), 1.0);
        assert_eq!(PrimeFactorization::new(-1).as_f64(), -1.0);
        assert_eq!(PrimeFactorization::new(1020).as_f64(), 1020.0);
    }

    #[test]
    fn test_mul() {
        let zero = PrimeFactorization::new(0);
        let one = PrimeFactorization::new(1);
        let five = PrimeFactorization::new(5);
        let m_twenty = PrimeFactorization::new(-20);

        assert_eq!(five.clone() * m_twenty.clone(), PrimeFactorization::new(-100));
        assert_eq!(m_twenty.clone() * five.clone(), PrimeFactorization::new(-100));

        // 0 x whatever is zero
        assert_eq!(zero.clone() * one.clone(), zero);
        assert_eq!(zero.clone() * m_twenty.clone(), zero);
        assert_eq!(one.clone() * zero.clone(), zero);
        assert_eq!(m_twenty.clone() * zero.clone(), zero);

        // 1 x whatever is whatever
        assert_eq!(one.clone() * five.clone(), five);
        assert_eq!(one.clone() * m_twenty.clone(), m_twenty);
        assert_eq!(five.clone() * one.clone(), five);
        assert_eq!(m_twenty.clone() * one.clone(), m_twenty);

        // a few sign tests
        assert_eq!(PrimeFactorization::new(-2) * PrimeFactorization::new(-2), PrimeFactorization::new(4));
        assert_eq!(PrimeFactorization::new(-2) * PrimeFactorization::new(2), PrimeFactorization::new(-4));
        assert_eq!(PrimeFactorization::new(2) * PrimeFactorization::new(2), PrimeFactorization::new(4));
        assert_eq!(PrimeFactorization::new(2) * PrimeFactorization::new(-2), PrimeFactorization::new(-4));
    }

    #[test]
    fn test_div() {
        let zero = PrimeFactorization::new(0);
        let one = PrimeFactorization::new(1);
        let five = PrimeFactorization::new(5);
        let m_twenty = PrimeFactorization::new(-20);

        assert_eq!(m_twenty.clone() / five.clone(), PrimeFactorization::new(-4));

        // 0 / whatever is zero
        assert_eq!(zero.clone() / one.clone(), zero);
        assert_eq!(zero.clone() / m_twenty.clone(), zero);

        // whatever / 1 is whatever
        assert_eq!(five.clone() / one.clone(), five);
        assert_eq!(m_twenty.clone() / one.clone(), m_twenty);

        // a few sign tests
        assert_eq!(PrimeFactorization::new(-2) / PrimeFactorization::new(-2), PrimeFactorization::new(1));
        assert_eq!(PrimeFactorization::new(-2) / PrimeFactorization::new(2), PrimeFactorization::new(-1));
        assert_eq!(PrimeFactorization::new(2) / PrimeFactorization::new(2), PrimeFactorization::new(1));
        assert_eq!(PrimeFactorization::new(2) / PrimeFactorization::new(-2), PrimeFactorization::new(-1));
    }

    #[test]
    #[should_panic = "attempt to divide by zero"]
    fn test_div_by_zero() {
        let _ = PrimeFactorization::new(1) / PrimeFactorization::new(0);
    }

    #[test]
    #[should_panic = "can not divide if the factorization do not have common prime factor"]
    fn test_div_no_common_factors() {
        let _ = PrimeFactorization::new(5) / PrimeFactorization::new(7);
    }

    #[test]
    fn test_factorial() {
        let factorial_200 = factorial(200);
        // checked with wolfram alpha
        assert_eq!(factorial_200.factors.as_slice(), [
            197, 97, 49, 32, 19, 16, 11, 10, 8, 6, 6, 5, 4, 4, 4, 3, 3, 3, 2, 2,
            2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1
        ])
    }
}
