//! Compute complex Wigner D matrices using the Risbo/Trapani-Navaza recurrence.
//!
//! The Wigner D matrix in the ZYZ Euler-angle convention is:
//!
//! ```text
//! D^j_{m,m'}(α, β, γ) = <j,m| exp(-iα J_z) exp(-iβ J_y) exp(-iγ J_z) |j,m'>
//!                      = exp(-i m α)  d^j_{m,m'}(β)  exp(-i m' γ)
//! ```
//!
//! where `d^j_{m,m'}(β)` is the real-valued small Wigner d-matrix.
//! The matrices are unitary: `D^j (D^j)† = I`, and satisfy
//! `D^j(0, 0, 0) = I`.
//!
//! # Algorithm
//!
//! The implementation follows the five-step recurrence of Risbo (1996) as
//! described in Trapani & Navaza (2006).  Instead of computing each `d^j_{m,m'}`
//! independently, the recurrence simultaneously fills all `(j, m, m')` triples up
//! to a requested maximum `j` by exploiting three-term recurrence relations that
//! couple neighbouring `j` and `m` values.
//!
//! The recurrence is seeded from `D⁰_{0,0} = 1` and proceeds in five phases:
//!
//! 1. **Step 1**: seed `D⁰_{0,0} = 1`.
//! 2. **Step 2**: fill the `m' = 0` column for all `j` using a recurrence in `m`
//!    that involves trigonometric factors `cos(β)` and `sin(β)`.
//! 3. **Step 3**: fill the `m' = 1` sub-diagonal by coupling `(j, m'=1)` values
//!    with previously computed `(j+1, m'=0)` values.
//! 4. **Step 4**: fill `m' ≥ 2` by stepping upward in `m'` using a recurrence
//!    that involves the precomputed `d` coefficients.
//! 5. **Step 5**: fill the `m' ≤ 0` half by applying symmetry relations.
//!
//! After these five steps the internal "H-wedge" array contains the real-valued
//! reduced matrix elements for all `(j, m, m')`.  The final complex D-matrix
//! elements are obtained by multiplying with the complex phases
//! `exp(-i m α) · exp(-i m' γ)` and the sign factors `ε(m)·ε(-m')`.
//!
//! Pre-computed scalar coefficients `a, b, d, g, h` that depend only on `(j, m)`
//! (not on the Euler angles) are re-used across all angle evaluations.
//!
//! # References
//!
//! - T. Risbo, "Fourier-transform summation of Legendre series by D-matrix
//!   transform", https://doi.org/10.1007/BF01090814
//! - S. Trapani & J. Navaza, "Calculation of spherical harmonics and Wigner d
//!   functions by FFT.", https://doi.org/10.1107/S0108767306017478

use num_complex::Complex;

/// Sign factor ε(m) used in the H-wedge normalisation.
///
/// Returns 1 if m ≤ 0, otherwise returns (-1)^m.
fn epsilon(m: i32) -> f64 {
    if m > 0 {
        if m % 2 == 0 {
            1.0
        } else {
            -1.0
        }
    } else {
        1.0
    }
}

/// Flat index for a `(n, m)` pair in the coefficient arrays `b, d, g, h`.
///
/// Layout: for each `n ≥ 0`, `m` runs from `-n` to `+n`.
/// The first index for level `n` is `n·(n+1)`.
fn nm_index(n: u32, m: i32) -> usize {
    (m + n as i32 * (n + 1) as i32) as usize
}

/// Flat index for a `(n, absm)` pair in the coefficient array `a`.
///
/// Layout: for each `n ≥ 0`, `absm` runs from 0 to `n`.
/// The first index for level `n` is `n·(n+1)/2`.
fn nabsm_index(n: u32, absm: u32) -> usize {
    (absm + n * (n + 1) / 2) as usize
}

/// Total number of elements in the H-wedge array for a given `mp_max` and `j_max`.
///
/// The H-wedge array stores values for `j ∈ [0, j_max]` and `|mp| ≤ mp_max`.
fn wigner_h_size(mp_max: u32, j_max: u32) -> usize {
    if mp_max >= j_max {
        return ((j_max + 1) * (j_max + 2) * (2 * j_max + 3) / 6) as usize;
    }
    let e = j_max as i64;
    let m = mp_max as i64;
    ((e + 1) * (e + 2) * (2 * e + 3) - 2 * (e - m) * (e - m + 1) * (e - m + 2)) as usize / 6
}

/// Index into the H-wedge array for a `(j, mp, m)` triple without symmetry reduction.
///
/// This is the "base" index that assumes `(j, mp, m)` is already in the
/// canonical form used by the recurrence.  The canonical region stores values
/// for `mp \in [-mp_max, mp_max]` and `m` in a range that depends on `mp`.
fn wigner_h_index_base(j: u32, mp: i32, m: i32, mp_max: u32) -> usize {
    let local_mp_max = if mp_max > j { j } else { mp_max };
    let mut idx = wigner_h_size(local_mp_max, j - 1);
    if mp < 1 {
        let local_mp_max_s = local_mp_max as i64;
        let j_s = j as i64;
        idx +=
            ((local_mp_max_s + mp as i64) * (2 * j_s - local_mp_max_s + mp as i64 + 1) / 2)
                as usize;
    } else {
        let local_mp_max_s = local_mp_max as i64;
        let j_s = j as i64;
        let mp_s = mp as i64;
        idx += ((local_mp_max_s + 1) * (2 * j_s - local_mp_max_s + 2) / 2) as usize;
        idx += ((mp_s - 1) * (2 * j_s - mp_s + 2) / 2) as usize;
    }
    idx += (m - mp.abs()) as usize;
    idx
}

/// Index into the H-wedge array using the full symmetry of the d-matrix.
///
/// Maps any `(j, mp, m)` to the canonical index using the symmetry:
///
/// ```text
/// d^j_{m,m'}(β) = d^j_{-m',-m}(β)
/// ```
///
/// This ensures the H-wedge array only stores values for a reduced triangular
/// region, minimising memory and computation.
fn wigner_h_index(j: u32, mp: i32, m: i32, mp_max: u32) -> usize {
    if j == 0 {
        return 0;
    }

    let local_mp_max = if mp_max > j { j } else { mp_max };

    if m < -mp {
        if m < mp {
            return wigner_h_index_base(j, -mp, -m, local_mp_max);
        }
        return wigner_h_index_base(j, -m, -mp, local_mp_max);
    }

    if m < mp {
        return wigner_h_index_base(j, m, mp, local_mp_max);
    }
    wigner_h_index_base(j, mp, m, local_mp_max)
}

#[derive(Debug, Clone)]
struct WignerDCoefficients {
    a: Vec<f64>,
    b: Vec<f64>,
    d: Vec<f64>,
    g: Vec<f64>,
    h: Vec<f64>,
}

/// Pre-compute the scalar recurrence coefficients `a, b, d, g, h`.
///
/// These coefficients depend only on `(n, m)` (with `n = j + 1` in some cases)
/// and are independent of the Euler angles.  They are used by the five
/// recurrence steps and can be re-used across multiple angle evaluations.
///
/// Arrays are sized for `n ∈ [0, j_max + 1]` (one extra level for the
/// boundary condition of the recurrence).
fn create_wigner_coefficients(j_max: u32) -> WignerDCoefficients {
    let n_range = (j_max + 2) as usize;
    let nm_total = n_range * n_range;
    let nabsm_total = n_range * (n_range + 1) / 2;

    let mut a = vec![0.0; nabsm_total];
    let mut b = vec![0.0; nm_total];
    let mut d = vec![0.0; nm_total];
    let mut g = vec![0.0; nm_total];
    let mut h = vec![0.0; nm_total];

    for n in 0..n_range {
        let nu = n as u32;
        for m in -(n as i32)..=(n as i32) {
            let ni = nm_index(nu, m);
            let nf = n as f64;
            let mf = m as f64;

            b[ni] =
                ((nf - mf - 1.0) * (nf - mf) / ((2.0 * nf - 1.0) * (2.0 * nf + 1.0))).sqrt();
            if m < 0 {
                b[ni] = -b[ni];
            }

            d[ni] = 0.5 * ((nf - mf) * (nf + mf + 1.0)).sqrt();
            if m < 0 {
                d[ni] = -d[ni];
            }

            if nf - mf != 0.0 && nf + mf + 1.0 != 0.0 {
                g[ni] = 2.0 * (mf + 1.0) / ((nf - mf) * (nf + mf + 1.0)).sqrt();
                h[ni] = ((nf + mf + 2.0) * (nf - mf - 1.0)
                    / ((nf - mf) * (nf + mf + 1.0)))
                .sqrt();
            } else {
                g[ni] = 0.0;
                h[ni] = 0.0;
            }
        }
    }

    for n in 0..n_range {
        let nu = n as u32;
        for absm in 0..=n {
            let ai = nabsm_index(nu, absm as u32);
            let nf = n as f64;
            let absmf = absm as f64;
            let num = (nf + 1.0 + absmf) * (nf + 1.0 - absmf);
            let den = (2.0 * nf + 1.0) * (2.0 * nf + 3.0);
            a[ai] = (num / den).sqrt();
        }
    }

    WignerDCoefficients { a, b, d, g, h }
}

/// Step 1 of the Risbo recurrence: seed `D⁰_{0,0} = 1`.
fn step_1(hwedge: &mut [f64]) {
    hwedge[0] = 1.0;
}

/// Step 2 of the Risbo recurrence: fill the `mp = 0` column for all j.
///
/// This uses a recurrence in `m` that involves the `g` and `h` coefficients
/// and the trigonometric factors `cos(β)` and `sin(β)`.
#[allow(clippy::too_many_arguments)]
fn step_2(
    g: &[f64],
    h: &[f64],
    n_max: u32,
    mp_max: u32,
    hwedge: &mut [f64],
    hextra: &mut [f64],
    hv: &mut [f64],
    expi_beta: Complex<f64>,
) {
    let cos_beta = expi_beta.re;
    let sin_beta = expi_beta.im;
    let sqrt3 = (3.0_f64).sqrt();
    let inverse_sqrt2 = 1.0 / (2.0_f64).sqrt();

    if n_max > 0 {
        let n0n_index = wigner_h_index(1, 0, 1, mp_max);
        let nn_index = nm_index(1, 1);
        hwedge[n0n_index] = sqrt3;
        hwedge[n0n_index - 1] = g[nn_index - 1] * cos_beta * inverse_sqrt2;

        for n in 2..(n_max + 2) {
            let (n0n_index, use_hwedge): (usize, bool) = if n <= n_max {
                (wigner_h_index(n, 0, n as i32, mp_max), true)
            } else {
                (n as usize, false)
            };
            let prev_index = wigner_h_index(n - 1, 0, (n - 1) as i32, mp_max);
            let nn_index_val = nm_index(n, n as i32);

            let const_val = (1.0 + 0.5 / n as f64).sqrt();
            let g_i = g[nn_index_val - 1];

            if use_hwedge {
                hwedge[n0n_index] = const_val * hwedge[prev_index];
                hwedge[n0n_index - 1] = g_i * cos_beta * hwedge[n0n_index];

                for i in 2..n {
                    let g_i = g[nn_index_val - i as usize];
                    let h_i = h[nn_index_val - i as usize];
                    hwedge[n0n_index - i as usize] = g_i * cos_beta * hwedge[n0n_index - i as usize + 1]
                        - h_i * sin_beta * sin_beta * hwedge[n0n_index - i as usize + 2];
                }

                let const_val = 1.0 / (4.0 * n as f64 + 2.0).sqrt();
                let g_i = g[nn_index_val - n as usize];
                let h_i = h[nn_index_val - n as usize];
                hwedge[n0n_index - n as usize] = (g_i * cos_beta * hwedge[n0n_index - n as usize + 1]
                    - h_i * sin_beta * sin_beta * hwedge[n0n_index - n as usize + 2])
                    * const_val;

                let mut prefactor = const_val;
                for i in 1..n {
                    prefactor *= sin_beta;
                    hwedge[n0n_index - n as usize + i as usize] *= prefactor;
                }
            } else {
                hextra[n0n_index] = const_val * hwedge[prev_index];
                hextra[n0n_index - 1] = g_i * cos_beta * hextra[n0n_index];

                for i in 2..n {
                    let g_i = g[nn_index_val - i as usize];
                    let h_i = h[nn_index_val - i as usize];
                    hextra[n0n_index - i as usize] = g_i * cos_beta
                        * hextra[n0n_index - i as usize + 1]
                        - h_i * sin_beta * sin_beta * hextra[n0n_index - i as usize + 2];
                }

                let const_val = 1.0 / (4.0 * n as f64 + 2.0).sqrt();
                let g_i = g[nn_index_val - n as usize];
                let h_i = h[nn_index_val - n as usize];
                hextra[n0n_index - n as usize] = (g_i * cos_beta
                    * hextra[n0n_index - n as usize + 1]
                    - h_i * sin_beta * sin_beta * hextra[n0n_index - n as usize + 2])
                    * const_val;

                let mut prefactor = const_val;
                for i in 1..n {
                    prefactor *= sin_beta;
                    hextra[n0n_index - n as usize + i as usize] *= prefactor;
                }
            }

            if n <= n_max {
                let n_idx = nm_index(n, 1);
                let h_idx = wigner_h_index(n, 0, 1, mp_max);
                hv[n_idx] = hwedge[h_idx];
                hv[n_idx - 1] = hwedge[h_idx];
            }
        }

        let mut prefactor = 1.0;
        for n in 1..=n_max {
            prefactor *= sin_beta;
            let idx = wigner_h_index(n, 0, n as i32, mp_max);
            hwedge[idx] *= prefactor / (4.0 * n as f64 + 2.0).sqrt();
        }
        prefactor *= sin_beta;
        hextra[n_max as usize + 1] *= prefactor / (4.0 * (n_max as f64 + 1.0) + 2.0).sqrt();

        let idx_1 = nm_index(1, 1);
        let h_idx = wigner_h_index(1, 0, 1, mp_max);
        hv[idx_1] = hwedge[h_idx];
        hv[idx_1 - 1] = hwedge[h_idx];
    }
}

/// Step 3 of the Risbo recurrence: fill the `mp = 1` sub-diagonal.
///
/// This couples `(j, mp=1)` values with previously computed `(j+1, mp=0)`
/// values using the `a` and `b` coefficients and trigonometric factors.
fn step_3(
    a: &[f64],
    b: &[f64],
    n_max: u32,
    mp_max: u32,
    hwedge: &mut [f64],
    hextra: &[f64],
    expi_beta: Complex<f64>,
) {
    let cos_beta = expi_beta.re;
    let sin_beta = expi_beta.im;
    if n_max > 0 && mp_max > 0 {
        for n in 1..=n_max {
            let i1 = wigner_h_index(n, 1, 1, mp_max);
            let use_hextra = n + 1 > n_max;
            let i2 = if use_hextra {
                0
            } else {
                wigner_h_index(n + 1, 0, 0, mp_max)
            };
            let i3 = nm_index(n + 1, 0);
            let i4 = nabsm_index(n, 1);
            let inverse_b5 = 1.0 / b[i3];

            for i in 0..n {
                let b6 = b[(i3 as isize - i as isize - 2) as usize];
                let b7 = b[i3 + i as usize];
                let a8 = a[i4 + i as usize];

                let (v0, v1, v2) = if use_hextra {
                    (
                        hextra[i as usize + 2],
                        hextra[i as usize + 1],
                        hextra[i as usize],
                    )
                } else {
                    (
                        hwedge[i2 + i as usize + 2],
                        hwedge[i2 + i as usize + 1],
                        hwedge[i2 + i as usize],
                    )
                };

                hwedge[i1 + i as usize] = inverse_b5
                    * (0.5 * (b6 * (1.0 - cos_beta) * v0 - b7 * (1.0 + cos_beta) * v2)
                        - a8 * sin_beta * v1);
            }
        }
    }
}

/// Step 4 of the Risbo recurrence: fill `mp ≥ 2` by stepping up in `mp`.
///
/// Uses the `d` coefficient recurrence to progressively fill higher `mp`
/// values from previously computed lower `mp` values at the same `j`.
fn step_4(d: &[f64], n_max: u32, mp_max: u32, hwedge: &mut [f64], hv: &mut [f64]) {
    if n_max > 0 && mp_max > 0 {
        for n in 2..=n_max {
            for mp in 1..(if mp_max < n { mp_max } else { n }) {
                let i1 = wigner_h_index(n, mp as i32 + 1, mp as i32 + 1, mp_max) - 1;
                let i2 = wigner_h_index(n, mp as i32 - 1, mp as i32, mp_max);
                let i3 = wigner_h_index(n, mp as i32, mp as i32, mp_max) - 1;
                let i4 = wigner_h_index(n, mp as i32, mp as i32 + 1, mp_max);
                let i5 = nm_index(n, mp as i32);
                let i6 = nm_index(n, mp as i32 - 1);
                let inverse_d5 = 1.0 / d[i5];
                let d6 = d[i6];

                hv[nm_index(n, mp as i32 + 1)] = inverse_d5
                    * (d6 * hwedge[i2] - d[i6] * hv[nm_index(n, mp as i32)] + d[i5] * hwedge[i4]);

                for i in 1..(n - mp) {
                    let d7 = d[i6 + i as usize];
                    let d8 = d[i5 + i as usize];
                    hwedge[i1 + i as usize] = inverse_d5
                        * (d6 * hwedge[i2 + i as usize]
                            - d7 * hwedge[i3 + i as usize]
                            + d8 * hwedge[i4 + i as usize]);
                }

                let i = n - mp;
                hwedge[i1 + i as usize] = inverse_d5
                    * (d6 * hwedge[i2 + i as usize]
                        - d[i6 + i as usize] * hwedge[i3 + i as usize]);
            }
        }
    }
}

/// Step 5 of the Risbo recurrence: fill `mp ≤ 0` by stepping down in `mp`.
///
/// Applies symmetry relations using the `d` coefficient recurrence to fill
/// the negative `mp` values from the already-computed positive side.
fn step_5(d: &[f64], n_max: u32, mp_max: u32, hwedge: &mut [f64], hv: &mut [f64]) {
    if n_max > 0 && mp_max > 0 {
        for n in 0..=n_max {
            let limit = if mp_max < n { mp_max } else { n };
            for mp_i in 0..limit {
                let mp_val = -(mp_i as i32);

                let i1 = wigner_h_index(n, mp_val - 1, -mp_val + 1, mp_max) - 1;
                let i2 = wigner_h_index(n, mp_val + 1, -mp_val + 1, mp_max) - 1;
                let i3 = wigner_h_index(n, mp_val, -mp_val, mp_max) - 1;
                let i4 = wigner_h_index(n, mp_val, -mp_val + 1, mp_max);
                let i5 = nm_index(n, mp_val - 1);
                let i6 = nm_index(n, mp_val);
                let i7 = nm_index(n, -mp_val - 1);
                let i8 = nm_index(n, -mp_val);
                let inverse_d5 = 1.0 / d[i5];
                let d6 = d[i6];
                let d7 = d[i7];
                let d8 = d[i8];

                if mp_val == 0 {
                    hv[nm_index(n, mp_val - 1)] = inverse_d5
                        * (d6 * hv[nm_index(n, mp_val + 1)]
                            + d7 * hv[nm_index(n, mp_val)]
                            - d8 * hwedge[i4]);
                } else {
                    hv[nm_index(n, mp_val - 1)] = inverse_d5
                        * (d6 * hwedge[i2] + d7 * hv[nm_index(n, mp_val)] - d8 * hwedge[i4]);
                }

                let n_plus_mp = (n as i32 + mp_val) as usize;
                for i in 1..n_plus_mp {
                    let d7_i = d[i7 + i];
                    let d8_i = d[i8 + i];
                    hwedge[i1 + i] = inverse_d5
                        * (d6 * hwedge[i2 + i] + d7_i * hwedge[i3 + i] - d8_i * hwedge[i4 + i]);
                }

                if n_plus_mp > 0 {
                    hwedge[i1 + n_plus_mp] = inverse_d5
                        * (d6 * hwedge[i2 + n_plus_mp]
                            + d[i7 + n_plus_mp] * hwedge[i3 + n_plus_mp]);
                }
            }
        }
    }
}

/// Total number of complex D-matrix elements for all j in [0, max_j].
///
/// Equivalently:
/// ```text
///     sum_{j=0}^{N} (2·j + 1)²  =  (N+1)(2N+1)(2N+3) / 3
/// ```
fn total_d_matrix_size(max_j: u32) -> usize {
    let n = max_j as u64;
    ((n + 1) * (2 * n + 1) * (2 * n + 3) / 3) as usize
}

/// Compute the full complex Wigner D matrices for all `j ∈ [0, max_j]`
/// at the given ZYZ Euler angles.
///
/// The Wigner D matrix is defined as:
///
/// ```text
/// D^j_{mp,m}(α, β, γ)  =  <j, mp| exp(-i α J_z) exp(-i β J_y) exp(-i γ J_z) |j, m>
///                         =  exp(-i mp α)  d^j_{mp,m}(β)  exp(-i m γ)
/// ```
///
/// where `d^j_{mp,m}(β)` is the real-valued small Wigner d-matrix and `J_z`,
/// `J_y` are angular-momentum operators.
///
/// # Output layout
///
/// The output is a flat array of interleaved `(real, imag)` `f64` pairs,
/// organized as contiguous per-`j` blocks:
///
/// ```text
/// [j=0 block] [j=1 block] ... [j=max_j block]
/// ```
///
/// Each `j` block contains `(2·j + 1)^2` complex values stored in
/// **row-major** order with `mp` as the row index and `m` as the column index.
/// Both indices run from `-j` to `+j`.
///
/// The total number of required doubles is
/// `2 · (max_j + 1)(2 · max_j + 1)(2 · max_j + 3) / 3`.
///
/// # Panics
///
/// Panics if `output.len()` is too small to hold the result.
///
/// # Example
///
/// ```rust
/// use wigners::wigner_d_array;
///
/// let mut buf = vec![0.0; 2 * 10]; // space for j=0 and j=1
/// wigner_d_array(1, 0.5, 1.0, 0.3, &mut buf);
/// ```
pub fn wigner_d_array(max_j: u32, alpha: f64, beta: f64, gamma: f64, output: &mut [f64]) {
    let mp_max = max_j;
    let hsize = wigner_h_size(mp_max, max_j);
    let total_complex = total_d_matrix_size(max_j);
    let required_len = 2 * total_complex;
    assert!(
        output.len() >= required_len,
        "output too small: need {} doubles, got {}",
        required_len,
        output.len()
    );

    let WignerDCoefficients { a, b, d, g, h } = create_wigner_coefficients(max_j);
    let mut hwedge = vec![0.0; hsize];
    let mut hv = vec![0.0; (max_j + 1) as usize * (max_j + 1) as usize];
    let mut hextra = vec![0.0; (max_j + 2) as usize];

    let expi_beta = Complex::new(beta.cos(), beta.sin());

    step_1(&mut hwedge);
    step_2(&g, &h, max_j, mp_max, &mut hwedge, &mut hextra, &mut hv, expi_beta);
    step_3(&a, &b, max_j, mp_max, &mut hwedge, &hextra, expi_beta);
    step_4(&d, max_j, mp_max, &mut hwedge, &mut hv);
    step_5(&d, max_j, mp_max, &mut hwedge, &mut hv);

    let z_alpha = Complex::new((-alpha).cos(), (-alpha).sin());
    let z_gamma = Complex::new((-gamma).cos(), (-gamma).sin());

    let mut alpha_powers = Vec::with_capacity((max_j + 1) as usize);
    let mut gamma_powers = Vec::with_capacity((max_j + 1) as usize);
    alpha_powers.push(Complex::new(1.0, 0.0));
    gamma_powers.push(Complex::new(1.0, 0.0));
    for _ in 1..=(max_j as usize) {
        alpha_powers.push(*alpha_powers.last().unwrap() * z_alpha);
        gamma_powers.push(*gamma_powers.last().unwrap() * z_gamma);
    }

    let mut offset = 0;
    for j in 0..=max_j {
        let dim = 2 * j + 1;
        let local_mp_max = if mp_max < j { mp_max } else { j };

        for mp in -(j as i32)..=(j as i32) {
            for m in -(j as i32)..=(j as i32) {
                let h_idx = wigner_h_index(j, mp, m, local_mp_max);
                let h_val = hwedge[h_idx];
                let eps = epsilon(mp) * epsilon(-m);

                let mp_abs = mp.unsigned_abs();
                let m_abs = m.unsigned_abs();
                let phase_alpha = if mp >= 0 {
                    alpha_powers[mp_abs as usize]
                } else {
                    alpha_powers[mp_abs as usize].conj()
                };
                let phase_gamma = if m >= 0 {
                    gamma_powers[m_abs as usize]
                } else {
                    gamma_powers[m_abs as usize].conj()
                };
                let d_val = Complex::new(h_val, 0.0) * eps * phase_alpha * phase_gamma;

                let idx = offset
                    + 2 * (((mp + j as i32) * dim as i32 + (m + j as i32)) as usize);
                output[idx] = d_val.re;
                output[idx + 1] = d_val.im;
            }
        }
        offset += 2 * (dim * dim) as usize;
    }
}

/// C-compatible wrapper for `wigner_d_array`.
///
/// Parameters match the Rust version, except `output` is a raw pointer to a
/// pre-allocated buffer of `output_len` `f64` values (interleaved real/imag
/// pairs).  See [`wigner_d_array`] for the full documentation.
#[no_mangle]
pub unsafe extern "C" fn wigner_D_array_c(
    max_j: u32,
    alpha: f64,
    beta: f64,
    gamma: f64,
    output: *mut f64,
    output_len: u64,
) {
    let output = std::slice::from_raw_parts_mut(output, output_len as usize);
    wigner_d_array(max_j, alpha, beta, gamma, output);
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_ulps_eq;

    #[test]
    fn test_total_d_matrix_size() {
        assert_eq!(total_d_matrix_size(0), 1);
        assert_eq!(total_d_matrix_size(1), 1 + 9);
        assert_eq!(total_d_matrix_size(2), 1 + 9 + 25);
    }

    #[test]
    fn test_identity_at_zero_angles() {
        for max_j in 0..=5 {
            let total = total_d_matrix_size(max_j);
            let mut output = vec![0.0; 2 * total];
            wigner_d_array(max_j, 0.0, 0.0, 0.0, &mut output);

            let mut offset: usize = 0;
            for j in 0..=max_j {
                let dim_u = 2 * j + 1;
                let dim = dim_u as usize;
                for mp in 0..dim {
                    for m in 0..dim {
                        let idx = 2 * (mp * dim + m);
                        let re = output[offset + idx];
                        let im = output[offset + idx + 1];
                        if mp == m {
                            assert_ulps_eq!(re, 1.0, epsilon = 1e-12);
                        } else {
                            assert_ulps_eq!(re, 0.0, epsilon = 1e-12);
                        }
                        assert_ulps_eq!(im, 0.0, epsilon = 1e-12);
                    }
                }
                offset += 2 * dim * dim;
            }
        }
    }

    #[test]
    fn test_unitarity() {
        for max_j in 0..=4 {
            let alpha = 0.3;
            let beta = 0.7;
            let gamma = 1.2;

            let total = total_d_matrix_size(max_j);
            let mut output = vec![0.0; 2 * total];
            wigner_d_array(max_j, alpha, beta, gamma, &mut output);

            let mut offset: usize = 0;
            for j in 0..=max_j {
                let dim_u = 2 * j + 1;
                let dim = dim_u as usize;

                let mut d = vec![Complex::new(0.0, 0.0); dim * dim];
                for mp in 0..dim {
                    for m in 0..dim {
                        let idx = 2 * (mp * dim + m);
                        d[mp * dim + m] =
                            Complex::new(output[offset + idx], output[offset + idx + 1]);
                    }
                }

                for i in 0..dim {
                    for j in 0..dim {
                        let mut sum = Complex::new(0.0, 0.0);
                        for k in 0..dim {
                            sum += d[i * dim + k] * d[j * dim + k].conj();
                        }
                        if i == j {
                            assert_ulps_eq!(sum.re, 1.0, epsilon = 1e-10);
                        } else {
                            assert_ulps_eq!(sum.re, 0.0, epsilon = 1e-10);
                        }
                        assert_ulps_eq!(sum.im, 0.0, epsilon = 1e-10);
                    }
                }

                offset += 2 * dim * dim;
            }
        }
    }

    #[test]
    #[allow(clippy::erasing_op, clippy::identity_op)]
    fn test_j1_known_values() {
        let total = total_d_matrix_size(1);
        let mut output = vec![0.0; 2 * total];

        let alpha = 0.5;
        let beta = 1.0;
        let gamma = 0.3;

        wigner_d_array(1, alpha, beta, gamma, &mut output);

        let dim = 3;
        let offset_j1 = 2;

        let mut d = [Complex::new(0.0, 0.0); 9];
        for mp in 0..3 {
            for m in 0..3 {
                let idx = 2 * (mp * dim + m);
                d[mp * dim + m] = Complex::new(output[offset_j1 + idx], output[offset_j1 + idx + 1]);
            }
        }

        let expected_00 = Complex::new(beta.cos(), 0.0);
        assert_ulps_eq!(d[1 * 3 + 1].re, expected_00.re, epsilon = 1e-12);
        assert_ulps_eq!(d[1 * 3 + 1].im, expected_00.im, epsilon = 1e-12);

        let expected_11 = Complex::new(0.0, -alpha).exp() * (1.0 + beta.cos()) / 2.0
            * Complex::new(0.0, -gamma).exp();
        assert_ulps_eq!(d[2 * 3 + 2].re, expected_11.re, epsilon = 1e-12);
        assert_ulps_eq!(d[2 * 3 + 2].im, expected_11.im, epsilon = 1e-12);

        let expected_m1m1 = Complex::new(0.0, alpha).exp() * (1.0 + beta.cos()) / 2.0
            * Complex::new(0.0, gamma).exp();
        assert_ulps_eq!(d[0 * 3 + 0].re, expected_m1m1.re, epsilon = 1e-12);
        assert_ulps_eq!(d[0 * 3 + 0].im, expected_m1m1.im, epsilon = 1e-12);

        let expected_10 = -Complex::new(0.0, -alpha).exp() * beta.sin() / (2.0_f64).sqrt();
        assert_ulps_eq!(d[2 * 3 + 1].re, expected_10.re, epsilon = 1e-12);
        assert_ulps_eq!(d[2 * 3 + 1].im, expected_10.im, epsilon = 1e-12);
    }

    #[test]
    fn test_j2_known_value() {
        use std::f64::consts::PI;

        let total = total_d_matrix_size(2);
        let mut output = vec![0.0; 2 * total];
        wigner_d_array(2, 0.0, PI / 3.0, 0.0, &mut output);

        let j2_offset = 2 * (1 + 9);
        let dim = 5;
        let idx_00 = j2_offset + 2 * (2 * dim + 2);
        assert_ulps_eq!(output[idx_00], -0.125, epsilon = 1e-12);
        assert_ulps_eq!(output[idx_00 + 1], 0.0, epsilon = 1e-12);
    }
}
