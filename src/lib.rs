#![allow(clippy::needless_return, clippy::redundant_field_names)]

mod primes;
mod rational;
mod real;

mod wigner_3j;
pub use self::wigner_3j::{wigner_3j, clear_wigner_3j_cache};
pub use self::wigner_3j::{clebsch_gordan, clebsch_gordan_array};


pub use self::real::clebsch_gordan_real_array;
