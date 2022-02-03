from sympy.physics.wigner import wigner_3j
import time


def format_time(duration):
    if duration > 1:
        return f"{duration:5.7}s"
    elif 1000 * duration > 1:
        return f"{1000 * duration:5.7} ms"


if __name__ == "__main__":
    for max_angular in [4, 8, 12]:
        start = time.time()
        for j1 in range(max_angular):
            for j2 in range(max_angular):
                for j3 in range(max_angular):
                    for m1 in range(-j1, j1 + 1):
                        for m2 in range(-j2, j2 + 1):
                            for m3 in range(-j3, j3 + 1):
                                c = wigner_3j(j1, j2, j3, m1, m2, m3)

        print(f"max_angular = {max_angular} took {format_time(time.time() - start)}")
