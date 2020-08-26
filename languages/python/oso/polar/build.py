import os
from cffi import FFI
from os import sys

ffibuilder = FFI()

lib_dirs = {
    "DEVELOPMENT": "../../../target/debug",
    "RELEASE": "../../../target/release",
    "CI": "native",
}
include_dirs = {
    "DEVELOPMENT": "../../../polar-c-api",
    "RELEASE": "../../../polar-c-api",
    "CI": "native",
}
env = os.environ.get("OSO_ENV", "DEVELOPMENT")
libs = []
if sys.platform.startswith("win"):
    libs.append(lib_dirs[env] + "/polar.lib")
    libs.append("Ws2_32.lib")
    libs.append("advapi32.lib")
    libs.append("userenv.lib")
else:
    libs.append(lib_dirs[env] + "/libpolar.a")
include_dir = include_dirs[env]

ffibuilder.set_source(
    "_polar_lib",
    r"""
    #include "polar.h"
    """,
    library_dirs=[lib_dirs[env]],
    include_dirs=[include_dir],
    libraries=["rt"] if sys.platform.startswith("linux") else [],
    extra_link_args=libs,
)

with open(include_dir + "/polar.h") as f:
    header = f.read()
    ffibuilder.cdef(header)


if __name__ == "__main__":  # not when running with setuptools
    ffibuilder.compile(verbose=True)
