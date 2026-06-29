import ctypes
import os

import numpy as np

__lib = ctypes.CDLL(os.path.join(os.path.dirname(__file__), "_wigners.so"))

__lib.wigner_3j.argtypes = [
    ctypes.c_uint32,
    ctypes.c_uint32,
    ctypes.c_uint32,
    ctypes.c_int32,
    ctypes.c_int32,
    ctypes.c_int32,
]
__lib.wigner_3j.restype = ctypes.c_double

__lib.clebsch_gordan.argtypes = [
    ctypes.c_uint32,
    ctypes.c_int32,
    ctypes.c_uint32,
    ctypes.c_int32,
    ctypes.c_uint32,
    ctypes.c_int32,
]
__lib.clebsch_gordan.restype = ctypes.c_double


__lib.clebsch_gordan_array_c.argtypes = [
    ctypes.c_uint32,
    ctypes.c_uint32,
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.c_double),
    ctypes.c_uint64,
]
__lib.clebsch_gordan_array_c.restype = None


__lib.wigner_D_array_c.argtypes = [
    ctypes.c_uint32,
    ctypes.c_double,
    ctypes.c_double,
    ctypes.c_double,
    ctypes.POINTER(ctypes.c_double),
    ctypes.c_uint64,
]
__lib.wigner_D_array_c.restype = None


__lib.clear_wigner_3j_cache.argtypes = []
__lib.clear_wigner_3j_cache.restype = None


def wigner_3j(j1: int, j2: int, j3: int, m1: int, m2: int, m3: int) -> float:
    """
    Compute a single Wigner 3j coefficient, corresponding to:

    .. code-block::

        | j1  j2  j3 |
        | m1  m2  m3 |
    """
    return __lib.wigner_3j(j1, j2, j3, m1, m2, m3)


def clebsch_gordan(j1: int, m1: int, j2: int, m2: int, j3: int, m3: int) -> float:
    """
    Compute a single Clebsch-Gordan coefficient, corresponding to:

    .. code-block::

        <j1 m1  j2 m2 | j3 m3>
    """
    return __lib.clebsch_gordan(j1, m1, j2, m2, j3, m3)


def clebsch_gordan_array(j1: int, j2: int, j3: int) -> np.ndarray:
    """
    Compute a full array of Clebsch-Gordan coefficient for the three given
    ``j``.

    The result is a 3-dimensional array with shape ``(2 * j1 + 1, 2 * j2 + 1, 2
    * j3 + 1)``.
    """
    array = np.zeros((2 * j1 + 1, 2 * j2 + 1, 2 * j3 + 1), dtype=np.float64)
    ptr = array.ctypes.data_as(ctypes.POINTER(ctypes.c_double))
    __lib.clebsch_gordan_array_c(j1, j2, j3, ptr, array.size)
    return array


def _total_d_complex_count(max_j: int) -> int:
    """Total number of complex D-matrix elements for all j in [0, max_j]."""
    return (max_j + 1) * (2 * max_j + 1) * (2 * max_j + 3) // 3


def wigner_D_array(
    max_j: int, alpha: float, beta: float, gamma: float
) -> list[np.ndarray]:
    """
    Compute the full complex Wigner D matrices for all j in ``[0, max_j]``
    using ZYZ Euler angles.

    Returns a list of ``(2*j+1, 2*j+1)`` complex128 matrices, ordered by
    increasing j. Each matrix is indexed as ``D[mp + j, m + j]`` where
    ``mp`` is the row index (first Euler angle) and ``m`` is the column index
    (third Euler angle).

    The convention is:
        D^j_{mp,m}(alpha, beta, gamma) = <j, mp| exp(-i Jz alpha)
            exp(-i Jy beta) exp(-i Jz gamma) |j, m>
    """
    total_complex = _total_d_complex_count(max_j)
    total_doubles = 2 * total_complex

    out = np.zeros(total_doubles, dtype=np.float64)
    ptr = out.ctypes.data_as(ctypes.POINTER(ctypes.c_double))
    __lib.wigner_D_array_c(max_j, alpha, beta, gamma, ptr, total_doubles)

    # View as complex128 (interleaved real/imag)
    out = out.view(np.complex128)

    # Split into per-j matrices
    matrices = []
    idx = 0
    for j in range(max_j + 1):
        size = (2 * j + 1) ** 2
        matrices.append(out[idx : idx + size].reshape(2 * j + 1, 2 * j + 1))
        idx += size
    return matrices


def clear_wigner_3j_cache():
    """Clear the LRU cache of Wigner 3j symbols"""
    return __lib.clear_wigner_3j_cache()
