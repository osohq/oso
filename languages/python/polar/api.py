import json

from _polar_lib import ffi, lib

from pathlib import Path


def to_python(v):
    """ Convert polar terms to python values """
    # i = v['id']
    # offset = v['offset']
    value = v["value"]
    tag = [*value][0]
    if tag in ["Integer", "String", "Boolean"]:
        return value[tag]
    if tag == "List":
        return [to_python(e) for e in value[tag]]
    # TODO
    return None


def to_polar(v):
    """ Convert python values to polar terms """
    if isinstance(v, int):
        return {"Integer": v}
    # TODO
    return None


class PolarException(Exception):
    pass


def this_is_what_you_get():
    for i in range(0, 5):
        yield i


class Polar:
    def __init__(self):
        self.polar = lib.polar_new()
        self.loaded_files = {}
        self.next_instance_id = 1
        self.instances = {}
        self.calls = {}

    def __del__(self):
        # Not usually needed but useful for tests since we make a lot of these.
        lib.polar_free(self.polar)

    # TODO: Use error types, not just PolarException
    def _raise_error(self):
        err = lib.polar_get_error()
        msg = ffi.string(err).decode()
        exception = PolarException(msg)
        lib.string_free(err)
        raise exception

    # NEW METHOD
    def load_str(self, src_str):
        c_str = ffi.new("char[]", src_str.encode())
        loaded = lib.polar_load_str(self.polar, c_str)
        if loaded == 0:
            self._raise_error()

    # NEW METHOD
    def query_str(self, query_str):
        c_str = ffi.new("char[]", query_str.encode())
        query = lib.polar_new_query(self.polar, c_str)
        if query == ffi.NULL:
            self._raise_error()

        while True:
            event_s = lib.polar_query(self.polar, query)
            if event_s == ffi.NULL:
                lib.query_free(query)
                self._raise_error()

            event = json.loads(ffi.string(event_s).decode())
            lib.string_free(event_s)
            if event == "Done":
                break

            kind = [*event][0]
            data = event[kind]

            if kind == "ExternalConstructor":
                # instance = data["instance"]
                lib.polar_external_construct_result(
                    self.polar, query, self.next_instance_id
                )
                self.next_instance_id += 1

            if kind == "ExternalCall":
                call_id = data["call_id"]
                # instance_id = data["instance_id"]
                # attribute = data["attribute"]
                # args = [to_python(arg) for arg in data["args"]]

                if call_id not in self.calls:
                    self.calls[call_id] = this_is_what_you_get()

                try:
                    val = next(self.calls[call_id])
                    c_str = ffi.new("char[]", to_polar(val).encode())
                    result = lib.polar_external_call_result(
                        self.polar, query, call_id, c_str
                    )
                    if result == 0:
                        self._raise_error()
                except StopIteration:
                    result = lib.polar_external_call_result(
                        self.polar, query, call_id, ffi.NULL
                    )
                    if result == 0:
                        self._raise_error()

            if kind == "Result":
                yield {k: to_python(v) for k, v in data["bindings"].items()}

        lib.query_free(query)

    def query(self, query, debug):
        raise NotImplementedError("query not implemented")

    def import_builtin_module(self, name):
        """ Import a builtin polar module """
        raise PolarException("Unimplemented")

    def load(self, policy_file):
        """ Load in polar policy """
        policy_file = Path(policy_file)

        extension = policy_file.suffix
        if extension not in (".pol", ".polar"):
            raise PolarException(f"Policy names must have .pol or .polar extension")

        if not policy_file.exists():
            raise PolarException(f"Could not find file: {policy_file}")

        if policy_file not in self.loaded_files:
            with open(policy_file, "r") as f:
                contents = f.read()
            self.loaded_files[policy_file] = contents
            self.load_str(contents)

    def clear(self):
        raise NotImplementedError("Not implemented")


# STUBS (importable but don't do anything)


class PolarApiException(Exception):
    pass


class Query:
    def __init__(self, *args, **kwargs):
        raise NotImplementedError()


class QueryResult:
    def __init__(self, *args, **kwargs):
        raise NotImplementedError()


def register_python_class(cls, from_polar=None):
    raise NotImplementedError()


def register_class(spec, source_class: type):
    raise NotImplementedError()
