from _polar_lib import ffi, lib
import json
from pathlib import Path

def to_python(v):
    """ Convert Terms to python values"""
    # i = v['id']
    # offset = v['offset']
    value = v['value']
    tag = [*value][0]
    if tag == 'Integer':
        return value[tag]
    # TODO
    return None

class PolarException(Exception):
    pass

class Polar:
    def __init__(self):
        self.polar = lib.polar_new()
        self.loaded_files = {}

    def __del__(self):
        # Not usually needed but useful for tests since we make a lot of these.
        lib.polar_free(self.polar)

    def load_str(self, src_str):
        c_str = ffi.new("char[]", src_str.encode())
        lib.polar_load_str(self.polar, c_str)

    def query(self, query_str):
        c_str = ffi.new("char[]", query_str.encode())
        query = lib.polar_new_query(self.polar, c_str)

        while True:
            event_s = lib.polar_query(self.polar, query)
            event = json.loads(ffi.string(event_s).decode())
            lib.free_string(event_s)
            if event == "Done":
                break

            kind = [*event][0]
            data = event[kind]

            if kind == "Result":
                yield {k: to_python(v) for k,v in data['bindings'].items()}

        lib.query_free(query)


    def import_builtin_module(self, name):
        """ Import a builtin polar module """
        raise PolarException("Unimplemented")

    def load(self, policy_file):
        """ Load in polar policy """
        policy_file = Path(policy_file)

        extension = policy_file.suffix
        if extension != ".pol" and extension != ".polar":
            raise PolarException(f"Policy names must have .pol or .polar extension")

        if not policy_file.exists():
            raise PolarException(f"Could not find file: {policy_file}")

        if policy_file not in self.loaded_files:
            with open(policy_file, 'r') as f:
                contents = f.read()
            self.loaded_files[policy_file] = contents
            self.load_str(contents)
