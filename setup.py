import os
import sys
import shutil
import subprocess

from setuptools import setup, Extension
from wheel.bdist_wheel import bdist_wheel
from distutils.command.build_ext import build_ext  # type: ignore

ROOT = os.path.realpath(os.path.dirname(__file__))

if sys.version_info < (3, 6):
    sys.exit("Sorry, Python < 3.6 is not supported")


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
        subprocess.run(
            ["cargo", "build", "--release"],
            cwd=ROOT,
            check=True,
        )

        for filename in ["libwigners.so", "libwigners.dylib", "wigners.dll"]:
            lib_path = os.path.join(ROOT, "target", "release", filename)
            if os.path.exists(lib_path):
                shutil.copy(
                    lib_path, os.path.join(self.build_lib, "wigners", "_wigners.so")
                )


# read version from Cargo.toml
with open("Cargo.toml") as fd:
    for line in fd:
        if line.startswith("version"):
            _, version = line.split(" = ")
            # remove quotes
            version = version[1:-2]
            # take the first version in the file, this should be the right one
            break

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
    package_data={"wigners": ["wigners.so"]},
)
