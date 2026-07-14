import os
import glob
import subprocess

from setuptools import setup, Extension
from setuptools.command.build_ext import build_ext
from setuptools.command.bdist_wheel import bdist_wheel

ROOT = os.path.realpath(os.path.dirname(__file__))


class universal_wheel(bdist_wheel):
    # When building the wheel, the `wheel` package assumes that if we have a
    # binary extension then we are linking to `libpython.so`; and thus the wheel
    # is only usable with a single python version. This is not the case for
    # here, and the wheel will be compatible with any Python >=3.6. This is
    # tracked in https://github.com/pypa/wheel/issues/185, but until then we
    # manually override the wheel tag.
    def get_tag(self):
        tag = bdist_wheel.get_tag(self)
        # tag[2:] contains the os/arch tags, we want to keep them
        return ("py3", "none") + tag[2:]


class cargo_ext(build_ext):
    """
    Build the native library using cargo
    """

    def run(self):
        cargo_build = ["cargo", "build", "--release"]

        subprocess.run(
            cargo_build,
            cwd=ROOT,
            check=True,
            env={"CARGO_TARGET_DIR": self.build_temp, **os.environ},
        )

        file_found = False
        for filename in ["libwigners.so", "libwigners.dylib", "wigners.dll"]:
            lib_path = os.path.join(self.build_temp, "release", filename)

            candidates = glob.glob(
                # match either target/release or target/<triple>/release
                os.path.join(self.build_temp, "**", "release", filename),
                recursive=True,
            )
            if len(candidates) > 1:
                raise ValueError(
                    f"Found multiple candidates for {filename}: {candidates}"
                )

            if len(candidates) == 1:
                lib_path = candidates[0]
                file_found = True
                self.copy_file(
                    lib_path, os.path.join(self.build_lib, "wigners", "_wigners.so")
                )
                break

        if not file_found:
            raise Exception("could not find wigners library in " + self.build_temp)


# read version from Cargo.toml
with open("Cargo.toml") as fd:
    for line in fd:
        if line.startswith("version"):
            _, version = line.split(" = ")
            # remove quotes
            version = version[1:-2]
            # take the first version in the file, this should be the right one
            break

if __name__ == "__main__":
    setup(
        version=version,
        ext_modules=[
            # only declare the extension, it is built & copied as required in the
            # build_ext command
            Extension(name="wigners", sources=[]),
        ],
        cmdclass={
            "build_ext": cargo_ext,
            "bdist_wheel": universal_wheel,
        },
    )
