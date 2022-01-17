import math
import unittest

import wigners


class WignersTest(unittest.TestCase):
    def test_wigner_3j(self):
        self.assertAlmostEqual(
            wigners.wigner_3j(j1=2, j2=6, j3=4, m1=0, m2=0, m3=1), 0.0
        )

        self.assertAlmostEqual(
            wigners.wigner_3j(j1=5, j2=3, j3=2, m1=-2, m2=3, m3=-1),
            -math.sqrt(330.0) / 330.0,
        )

    def test_clebsch_gordan(self):
        self.assertAlmostEqual(
            wigners.clebsch_gordan(j1=2, m1=0, j2=6, m2=0, j3=4, m3=1), 0.0
        )

        self.assertAlmostEqual(
            wigners.clebsch_gordan(j1=1, m1=1, j2=1, m2=1, j3=2, m3=2),
            1.0,
        )

        self.assertAlmostEqual(
            wigners.clebsch_gordan(j1=1, m1=1, j2=1, m2=0, j3=2, m3=1),
            1 / math.sqrt(2.0),
        )
