use std::f64::consts::FRAC_1_SQRT_2;

use ndarray::{Array3, ArrayView3, ArrayViewMut3};


// j1 + j2 + j3 % 2 == 0, keep the real part
fn convert_to_real_cg_even_jjj(complex_cg: ArrayView3<f64>, j1: i32, m1: i32, j2: i32, m2: i32, j3: i32, m3: i32) -> f64 {
    let i_m1 = (j1 + m1) as usize;
    let i_m2 = (j2 + m2) as usize;
    let i_m3 = (j3 + m3) as usize;

    // index of -m1
    let i_minus_m1 = (j1 - m1) as usize;
    // (-1)^m1
    let minus_1_pow_m1 = f64::powi(-1.0, m1);

    // index of -m2
    let i_minus_m2 = (j2 - m2) as usize;
    // (-1)^m2
    let minus_1_pow_m2 = f64::powi(-1.0, m2);

    // (-1)^m3
    let minus_1_pow_m3 = f64::powi(-1.0, m3);

    if m1 == 0 && m2 == 0 && m3 == 0 { // eq (1)
        return complex_cg[[i_m1, i_m2, i_m3]];
    } else if m1 == m2 && m3 == 0 { // eq (2)
        return minus_1_pow_m1 * complex_cg[[i_m1, i_minus_m2, i_m3]];
    } else if m1 == 0 && m2 == m3 { // eq (3)
        return complex_cg[[i_m1, i_m2, i_m3]];
    } else if m2 == 0 && m1 == m3 { // eq (4)
        return complex_cg[[i_m1, i_m2, i_m3]];
    } else if m1 > 0 && m2 > 0 && m3 > 0 { // eq (5)
        return FRAC_1_SQRT_2 * (
            complex_cg[[i_m1, i_m2, i_m3]]
            + minus_1_pow_m2 * complex_cg[[i_m1, i_minus_m2, i_m3]]
            + minus_1_pow_m1 * complex_cg[[i_minus_m1, i_m2, i_m3]]
            + minus_1_pow_m3 * complex_cg[[i_minus_m1, i_minus_m2, i_m3]]
        );
    } else if m1 > 0 && m2 < 0 && m3 < 0 { // eq (6)
        return FRAC_1_SQRT_2 * (
            minus_1_pow_m1 * complex_cg[[i_m1, i_m2, i_m3]]
            - minus_1_pow_m3 * complex_cg[[i_m1, i_minus_m2, i_m3]]
            + complex_cg[[i_minus_m1, i_m2, i_m3]]
            - minus_1_pow_m2 * complex_cg[[i_minus_m1, i_minus_m2, i_m3]]
        );
    } else if m1 < 0 && m2 > 0 && m3 < 0 { // eq (7)
        return FRAC_1_SQRT_2 * (
            minus_1_pow_m2 * complex_cg[[i_m1, i_m2, i_m3]]
            + complex_cg[[i_m1, i_minus_m2, i_m3]]
            - minus_1_pow_m3 * complex_cg[[i_minus_m1, i_m2, i_m3]]
            - minus_1_pow_m1 * complex_cg[[i_minus_m1, i_minus_m2, i_m3]]
        );
    } else if m1 < 0 && m2 < 0 && m3 > 0 { // eq (8)
        return - FRAC_1_SQRT_2 * (
            minus_1_pow_m3 * complex_cg[[i_m1, i_m2, i_m3]]
            - minus_1_pow_m1 * complex_cg[[i_m1, i_minus_m2, i_m3]]
            - minus_1_pow_m2 * complex_cg[[i_minus_m1, i_m2, i_m3]]
            + complex_cg[[i_minus_m1, i_minus_m2, i_m3]]
        );
    } else {
        return 0.0;
    }
}


// j1 + j2 + j3 % 2 == 1, keep the imaginary part
fn convert_to_real_cg_odd_jjj(complex_cg: ArrayView3<f64>, j1: i32, m1: i32, j2: i32, m2: i32, j3: i32, m3: i32) -> f64 {
    let i_m1 = (j1 + m1) as usize;
    let i_m2 = (j2 + m2) as usize;
    let i_m3 = (j3 + m3) as usize;

    // index of -m1
    let i_minus_m1 = (j1 - m1) as usize;
    // (-1)^m1
    let minus_1_pow_m1 = f64::powi(-1.0, m1);

    // index of -m2
    let i_minus_m2 = (j2 - m2) as usize;
    // (-1)^m2
    let minus_1_pow_m2 = f64::powi(-1.0, m2);

    // (-1)^m3
    let minus_1_pow_m3 = f64::powi(-1.0, m3);

    if m1 == 0 && m2 == 0 && m3 == 0 { // eq (1)
        return complex_cg[[i_m1, i_m2, i_m3]];
    } else if m1 == -m2 && m3 == 0 { // eq (2)
        // FIXME: this is -minus_1_pow_m1 * b * complex_cg[[i_m1, i_m2, i_m3]];
        return -minus_1_pow_m1 * complex_cg[[i_m1, i_m2, i_m3]];
    } else if m1 == 0 && m2 == -m3 { // eq (3)
        return complex_cg[[i_m1, i_minus_m2, i_m3]];
    } else if m2 == 0 && m1 == -m3 { // eq (4)
        return complex_cg[[i_minus_m1, i_m2, i_m3]];
    } else if m1 > 0 && m2 > 0 && m3 < 0 { // eq (5)
        return FRAC_1_SQRT_2 * minus_1_pow_m3 * (
            complex_cg[[i_m1, i_m2, i_m3]]
            + minus_1_pow_m2 * complex_cg[[i_m1, i_minus_m2, i_m3]]
            + minus_1_pow_m1 * complex_cg[[i_minus_m1, i_m2, i_m3]]
            + minus_1_pow_m3 * complex_cg[[i_minus_m1, i_minus_m2, i_m3]]
        );
    } else if m1 > 0 && m2 < 0 && m3 > 0 { // eq (6)
        return - FRAC_1_SQRT_2 * minus_1_pow_m3 * (
            minus_1_pow_m1 * complex_cg[[i_m1, i_m2, i_m3]]
            - minus_1_pow_m3 * complex_cg[[i_m1, i_minus_m2, i_m3]]
            + complex_cg[[i_minus_m1, i_m2, i_m3]]
            - minus_1_pow_m2 * complex_cg[[i_minus_m1, i_minus_m2, i_m3]]
        );
    } else if m1 < 0 && m2 > 0 && m3 > 0 { // eq (7)
        return - FRAC_1_SQRT_2 * minus_1_pow_m3 * (
            minus_1_pow_m2 * complex_cg[[i_m1, i_m2, i_m3]]
            + complex_cg[[i_m1, i_minus_m2, i_m3]]
            - minus_1_pow_m3 * complex_cg[[i_minus_m1, i_m2, i_m3]]
            - minus_1_pow_m1 * complex_cg[[i_minus_m1, i_minus_m2, i_m3]]
        );
    } else if m1 < 0 && m2 < 0 && m3 < 0 { // eq (8)
        return - FRAC_1_SQRT_2 * minus_1_pow_m3 * (
            minus_1_pow_m3 * complex_cg[[i_m1, i_m2, i_m3]]
            - minus_1_pow_m1 * complex_cg[[i_m1, i_minus_m2, i_m3]]
            - minus_1_pow_m2 * complex_cg[[i_minus_m1, i_m2, i_m3]]
            + complex_cg[[i_minus_m1, i_minus_m2, i_m3]]
        );
    } else {
        return 0.0;
    }
}

/// Compute an array of Clebsch-Gordan coefficients that can be used together
/// with real spherical harmonics (the CG coefficients themself are always
/// real-valued.)
///
/// This assumes the following convention for real spherical harmonics:
///
/// ```text
/// Y_lm = i / sqrt(2) * (Y_l^m - (-1)^m Y_l^(-m))      if m < 0
///        Y_l^m                                        if m == 0
///        1 / sqrt(2) * (Y_l^(-m) + (-1)^m Y_l^m)      if m > 0
/// ```
///
/// which follows https://en.wikipedia.org/wiki/Spherical_harmonics#Real_form.

// TODO: add link to paper with Michelangelo once on Arvix or put the derivation
// in the docs.
pub fn clebsch_gordan_real_array(j1: u32, j2: u32, j3: u32, output: &mut [f64]) {
    let mut complex_cg = vec![0.0; output.len()];
    crate::clebsch_gordan_array(j1, j2, j3, &mut complex_cg);

    let j1_size = (2 * j1 + 1) as usize;
    let j2_size = (2 * j2 + 1) as usize;
    let j3_size = (2 * j3 + 1) as usize;
    let complex_cg = Array3::from_shape_vec((j1_size, j2_size, j3_size), complex_cg).expect("wrong shape");
    let complex_cg = complex_cg.view();

    let mut real_cg = ArrayViewMut3::from_shape((j1_size, j2_size, j3_size), output).expect("wrong shape");

    let j1 = j1 as i32;
    let j2 = j2 as i32;
    let j3 = j3 as i32;

    if (j1 + j2 + j3) % 2 == 0 {
        for (i_m1, m1) in (-j1..=j1).enumerate() {
            for (i_m2, m2) in (-j2..=j2).enumerate() {
                for (i_m3, m3) in (-j3..=j3).enumerate() {
                    real_cg[(i_m1, i_m2, i_m3)] = convert_to_real_cg_even_jjj(complex_cg, j1, m1, j2, m2, j3, m3);
                }
            }
        }
    } else {
        for (i_m1, m1) in (-j1..=j1).enumerate() {
            for (i_m2, m2) in (-j2..=j2).enumerate() {
                for (i_m3, m3) in (-j3..=j3).enumerate() {
                    real_cg[(i_m1, i_m2, i_m3)] = convert_to_real_cg_odd_jjj(complex_cg, j1, m1, j2, m2, j3, m3);
                }
            }
        }
    }
}

#[no_mangle]
pub unsafe extern fn clebsch_gordan_real_array_c(j1: u32, j2: u32, j3: u32, data: *mut f64, len: u64) {
    let slice = std::slice::from_raw_parts_mut(data, len as usize);
    clebsch_gordan_real_array(j1, j2, j3, slice);
}
