from cffi import FFI
from os import sys

ffibuilder = FFI()

ffibuilder.set_source(
    "_polar_lib",
    r"""
    #include "polar.h"
    """,
    library_dirs=["../../target/release"],
    include_dirs=["../../polar"],
    libraries=["polar", "rt"] if sys.platform.startswith("linux") else ["polar"],
)

with open("../../polar/polar.h") as f:
    header = f.read()
    ffibuilder.cdef(header)


if __name__ == "__main__":  # not when running with setuptools
    ffibuilder.compile(verbose=True)
