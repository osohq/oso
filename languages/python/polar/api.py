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
        val = {"Integer": v}
        term = {"id": 0, "offset": 0, "value": val}
        return term
    # TODO
    return None


class PolarException(Exception):
    pass


class Foo:
    def __init__(self, start=0):
        self.start = start

    def call_me(self, end):
        for i in range(self.start, end):
            yield i


class Polar:
    def __init__(self):
        self.polar = lib.polar_new()
        self.loaded_files = {}

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
        instances = {}
        calls = {}

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

            if kind == "MakeExternal":
                instance_id = data["instance_id"]
                instance = data["instance"]

                # assert instance_id not in instances
                cls = instance["tag"]

                term_fields = instance["fields"]["fields"]

                fields = {}
                for k, v in term_fields.items():
                    fields[k] = to_python(v)

                # construct an instance
                # @TODO: use class constructor
                assert cls == "Foo"
                instance = Foo(**fields)

                instances[instance_id] = instance

                # @Q: Do we say anything on an error or just handle here?

            if kind == "ExternalCall":
                call_id = data["call_id"]
                instance_id = data["instance_id"]
                attribute = data["attribute"]
                args = [to_python(arg) for arg in data["args"]]

                if call_id not in calls:
                    instance = instances[instance_id]
                    call = getattr(instance, attribute)(*args)
                    calls[call_id] = call

                try:
                    val = next(calls[call_id])
                    term = to_polar(val)
                    msg = json.dumps(term)
                    c_str = ffi.new("char[]", msg.encode())
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
