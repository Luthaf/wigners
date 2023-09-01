import random
import sys

import numpy as np
from sympy.physics.wigner import wigner_3j as sympy_wigner_3j

import wigners

N_ERRORS = 0


def error(j1, j2, j3, m1, m2, m3, expected, actual):
    global N_ERRORS
    N_ERRORS += 1
    print(
        f"error for j1={j1} j2={j2} j3={j3} m1={m1} m2={m2} m3={m3}:",
        f"expected {expected}, got {actual}",
    )


def get_m(j1, j2, j3):
    m1 = random.randint(-j1, j1)
    m2 = random.randint(-j2, j2)
    m3 = min(max(-(m1 + m2), -j3), j3)

    return m1, m2, m3


if __name__ == "__main__":
    max_angular = 30
    print(f"running all J combinations below J={max_angular}")
    for j1 in range(max_angular):
        for j2 in range(max_angular):
            for j3 in range(max_angular):
                m1, m2, m3 = get_m(j1, j2, j3)

                expected = float(sympy_wigner_3j(j1, j2, j3, m1, m2, m3))
                actual = wigners.wigner_3j(j1, j2, j3, m1, m2, m3)
                if not np.isclose(expected, actual, rtol=1e-6, atol=1e-16):
                    error(j1, j2, j2, m1, m2, m3, expected, actual)

    n_combinations = 500
    print(
        f"running {n_combinations} random J combinations for {max_angular} <= J <= 100"
    )
    for _ in range(n_combinations):
        j1 = random.randint(max_angular, 100)
        j2 = random.randint(max_angular, 100)
        j3 = random.randint(max_angular, 100)
        m1, m2, m3 = get_m(j1, j2, j3)

        expected = float(sympy_wigner_3j(j1, j2, j3, m1, m2, m3))
        actual = wigners.wigner_3j(j1, j2, j3, m1, m2, m3)
        if not np.isclose(expected, actual, rtol=1e-6, atol=1e-16):
            error(j1, j2, j2, m1, m2, m3, expected, actual)

    print(
        f"running {n_combinations} random J combinations for {max_angular} <= J <= 500"
    )
    for _ in range(n_combinations):
        j1 = random.randint(max_angular, 500)
        j2 = random.randint(max_angular, 500)
        j3 = random.randint(max_angular, 500)
        m1, m2, m3 = get_m(j1, j2, j3)

        expected = float(sympy_wigner_3j(j1, j2, j3, m1, m2, m3))
        actual = wigners.wigner_3j(j1, j2, j3, m1, m2, m3)
        if not np.isclose(expected, actual, rtol=1e-6, atol=1e-16):
            error(j1, j2, j2, m1, m2, m3, expected, actual)

    if N_ERRORS != 0:
        print(f"got {N_ERRORS} errors!")
        sys.exit(1)
    else:
        print("all good!")
