#![allow(clippy::needless_return, clippy::redundant_field_names)]

mod primes;
mod rational;

mod wigner_3j;
pub use self::wigner_3j::{wigner_3j, clebsch_gordan, clear_wigner_3j_cache};
mod wigner_3j_half_integer;
pub use self::wigner_3j_half_integer::{
    wigner_3j_half_integer,
    clebsch_gordan_half_integer,
    clear_wigner_3j_cache_half_integer,
};
