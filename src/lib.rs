#![allow(clippy::needless_return, clippy::redundant_field_names)]

mod primes;
mod rational;

mod wigner_3j;
pub use self::wigner_3j::{clear_wigner_3j_cache, clebsch_gordan, wigner_3j};
