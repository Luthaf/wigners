import os
import ctypes

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


def wigner_3j(j1: int, j2: int, j3: int, m1: int, m2: int, m3: int) -> float:
    return __lib.wigner_3j(j1, j2, j3, m1, m2, m3)


def clebsch_gordan(j1: int, m1: int, j2: int, m2: int, j3: int, m3: int) -> float:
    return __lib.clebsch_gordan(j1, m1, j2, m2, j3, m3)
