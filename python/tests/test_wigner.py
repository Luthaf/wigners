import math

import numpy as np
import wigners


def test_wigner_3j():
    assert wigners.wigner_3j(j1=2, j2=6, j3=4, m1=0, m2=0, m3=1) == 0.0

    assert (
        wigners.wigner_3j(j1=5, j2=3, j3=2, m1=-2, m2=3, m3=-1)
        == -math.sqrt(330.0) / 330.0
    )


def test_clebsch_gordan():
    cg = wigners.clebsch_gordan(j1=2, m1=0, j2=6, m2=0, j3=4, m3=1)
    assert cg == 0.0

    cg = wigners.clebsch_gordan(j1=1, m1=1, j2=1, m2=1, j3=2, m3=2)
    assert cg == 1.0

    cg = wigners.clebsch_gordan(j1=1, m1=1, j2=1, m2=0, j3=2, m3=1)
    assert np.isclose(cg, 1 / math.sqrt(2.0))


def test_clebsch_gordan_array():
    j1 = 12
    j2 = 15
    j3 = 8

    expected = np.zeros((2 * j1 + 1, 2 * j2 + 1, 2 * j3 + 1), dtype=np.float64)
    for m1 in range(-j1, j1 + 1):
        for m2 in range(-j2, j2 + 1):
            for m3 in range(-j3, j3 + 1):
                expected[j1 + m1, j2 + m2, j3 + m3] = wigners.clebsch_gordan(
                    j1, m1, j2, m2, j3, m3
                )

    assert np.allclose(wigners.clebsch_gordan_array(j1, j2, j3), expected)


def test_wigner_d_identity():
    """D(0,0,0) should be the identity matrix for all ell."""
    matrices = wigners.wigner_D_array(4, 0.0, 0.0, 0.0)
    for ell, matrix in enumerate(matrices):
        assert np.allclose(matrix, np.eye(2 * ell + 1, dtype=np.complex128))


def test_wigner_d_unitarity():
    """D * D^dagger = I for all ell."""
    matrices = wigners.wigner_D_array(4, 0.3, 0.7, 1.2)
    for ell, matrix in enumerate(matrices):
        identity = matrix @ matrix.conj().T
        assert np.allclose(
            identity, np.eye(2 * ell + 1, dtype=np.complex128), atol=1e-10
        )


def test_wigner_d_rotation_product():
    """The Wigner-D matrices represent the product of two rotations."""
    max_j = 8
    alpha = 0.3
    beta_1 = 0.4
    beta_2 = 0.9
    gamma = -0.7

    first = wigners.wigner_D_array(max_j, alpha, beta_1, 0.0)
    second = wigners.wigner_D_array(max_j, 0.0, beta_2, gamma)
    product = wigners.wigner_D_array(max_j, alpha, beta_1 + beta_2, gamma)

    for j in range(max_j + 1):
        np.testing.assert_allclose(
            product[j],
            first[j] @ second[j],
            rtol=0.0,
            atol=1e-12,
            err_msg=f"j={j}",
        )


def test_wigner_d_beta_zero():
    """D(alpha, 0, gamma) should be diagonal: exp(-i*m*(alpha+gamma))."""
    alpha, gamma = 0.3, 0.7
    matrices = wigners.wigner_D_array(3, alpha, 0.0, gamma)
    for ell, matrix in enumerate(matrices):
        for mp_idx in range(2 * ell + 1):
            for m_idx in range(2 * ell + 1):
                if mp_idx == m_idx:
                    m = mp_idx - ell
                    expected = np.exp(-1j * m * (alpha + gamma))
                    assert np.isclose(matrix[mp_idx, m_idx], expected, atol=1e-12)
                else:
                    assert np.isclose(matrix[mp_idx, m_idx], 0.0, atol=1e-12)


def test_wigner_d_alpha_gamma_zero():
    """D(0, beta, 0) should be real (the small d-matrix)."""
    matrices = wigners.wigner_D_array(3, 0.0, 0.5, 0.0)
    for matrix in matrices:
        assert np.allclose(matrix.imag, 0.0, atol=1e-14)


def test_wigner_d_j0():
    """D^0 is always 1."""
    for _ in range(5):
        import random

        matrices = wigners.wigner_D_array(
            0, random.random(), random.random(), random.random()
        )
        assert np.isclose(matrices[0], 1.0)


