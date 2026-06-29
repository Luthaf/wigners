import random
import time

import numpy as np

from sympy.physics.wigner import wigner_3j as sympy_wigner_3j

import wigners


def _get_m(j1, j2, j3):
    for _ in range(3):
        m1 = random.randint(-j1, j1)
        m2 = random.randint(-j2, j2)
        m3 = -(m1 + m2)
        if abs(m3) <= j3:
            return m1, m2, m3
    m3 = random.randint(-j3, j3)
    return m1, m2, m3


def _check_j(j1, j2, j3, m1, m2, m3):
    expected = float(sympy_wigner_3j(j1, j2, j3, m1, m2, m3))
    actual = wigners.wigner_3j(j1, j2, j3, m1, m2, m3)
    assert np.isclose(expected, actual, rtol=1e-6, atol=1e-16), (
        f"mismatch for j1={j1} j2={j2} j3={j3} m1={m1} m2={m2} m3={m3}: "
        f"expected {expected}, got {actual}"
    )


def test_all_small_j():
    max_angular = 15
    for j1 in range(max_angular):
        for j2 in range(max_angular):
            for j3 in range(max_angular):
                m1, m2, m3 = _get_m(j1, j2, j3)
                _check_j(j1, j2, j3, m1, m2, m3)


def test_random_medium_j():
    random.seed(time.time())
    n_combinations = 500
    for _ in range(n_combinations):
        j1 = random.randint(15, 100)
        j2 = random.randint(15, 100)
        j3 = random.randint(15, 100)
        m1, m2, m3 = _get_m(j1, j2, j3)
        _check_j(j1, j2, j3, m1, m2, m3)


def test_random_large_j():
    random.seed(time.time())
    n_combinations = 100
    for _ in range(n_combinations):
        j1 = random.randint(30, 500)
        j2 = random.randint(30, 500)
        j3 = random.randint(30, 500)
        m1, m2, m3 = _get_m(j1, j2, j3)
        _check_j(j1, j2, j3, m1, m2, m3)
