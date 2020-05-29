import os
from cffi import FFI

ffibuilder = FFI()

lib_dirs = {
    "DEVELOPMENT": "../../target/debug",
    "RELEASE": "../../target/release",
    "LINUX": "native",
}
include_dirs = {
    "DEVELOPMENT": "../../polar",
    "RELEASE": "../../polar",
    "LINUX": "native",
}
env = os.environ.get("ENV", "DEVELOPMENT")
lib_dir = lib_dirs[env]
include_dir = include_dirs[env]

ffibuilder.set_source(
    "_polar_lib",
    r"""
    #include "polar.h"
    """,
    library_dirs=[lib_dir],
    include_dirs=[include_dir],
    libraries=["polar"],
)

with open(include_dir + "/polar.h") as f:
    header = f.read()
    ffibuilder.cdef(header)


if __name__ == "__main__":  # not when running with setuptools
    ffibuilder.compile(verbose=True)
