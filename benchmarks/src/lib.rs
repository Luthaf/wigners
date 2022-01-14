
pub mod wigxjpf {
    use std::os::raw::{c_int, c_void, c_double};

    #[link(name="wigxjpf", kind="static")]
    extern "C" {
        pub fn wig_table_init(max_two_j: c_int, wigner_type: c_int) -> c_void;
        pub fn wig_table_free() -> c_void;
        pub fn wig_temp_init(max_two_j: c_int) -> c_void;
        pub fn wig_temp_free() -> c_void;

        pub fn wig3jj(two_j1: c_int, two_j2: c_int, two_j3: c_int, two_m1: c_int, two_m2: c_int, two_m3: c_int) -> c_double;
        pub fn wig6jj(two_j1: c_int, two_j2: c_int, two_j3: c_int, two_j4: c_int, two_j5: c_int, two_j6: c_int) -> c_double;
        pub fn wig9jj(two_j1: c_int, two_j2: c_int, two_j3: c_int, two_j4: c_int, two_j5: c_int, two_j6: c_int, two_j7: c_int, two_j8: c_int, two_j9: c_int) -> c_double;
    }
}
