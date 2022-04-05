import os
import ctypes
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


def clear_wigner_3j_cache():
    """Clear the LRU cache of Wigner 3j symbols"""
    return __lib.clear_wigner_3j_cache()
