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
