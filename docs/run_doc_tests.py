import argparse
from pathlib import Path
import importlib
import sys

from polar import Polar


def path_to_module_name(filename):
    # TODO (dhatch) Make this include the relative directory and some random stuff to not conflict.
    return Path(filename).stem


def load_python(filename, polar):
    """Load a python file into the knowledge base.

    Imports the file, and calls the load function from the file
    with the knowledge base.
    """
    module_name_tail = path_to_module_name(filename)
    module_name = f"polar.user.loaded.{module_name_tail}"
    spec = importlib.util.spec_from_file_location(module_name, filename)
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)

    module.load(polar)


def load(filename, polar):
    polar.load_file(filename)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("filename", nargs="+")

    args = parser.parse_args()
    polar = Polar()

    for filename in args.filename:
        if filename.endswith(".py"):
            load_python(filename, polar)

    for filename in args.filename:
        if filename.endswith(".polar"):
            load(filename, polar)


if __name__ == "__main__":
    main()
