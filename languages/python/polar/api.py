import json

from _polar_lib import ffi, lib

from pathlib import Path
from types import GeneratorType

# TODO: Better error types, not just PolarException
class PolarException(Exception):
    pass


class Polar:
    def __init__(self):
        self.polar = lib.polar_new()
        self.loaded_files = {}
        self.classes = {}
        self.class_constructors = {}

    def __del__(self):
        # Not usually needed but useful for tests since we make a lot of these.
        lib.polar_free(self.polar)

    def _raise_error(self):
        err = lib.polar_get_error()
        msg = ffi.string(err).decode()
        exception = PolarException(msg)
        lib.string_free(err)
        raise exception

    # @BreakingChange
    # These really need to be on the polar object, everything really needs to be on the polar object now.
    # Also can we just call this register_class and get rid of the other one?
    def register_python_class(self, cls, from_polar=None):
        class_name = cls.__name__
        self.classes[class_name] = cls
        self.class_constructors[class_name] = from_polar

    def register_class(self, spec, source_class: type):
        raise NotImplementedError()

    def load_str(self, src_str):
        c_str = ffi.new("char[]", src_str.encode())
        loaded = lib.polar_load_str(self.polar, c_str)
        if loaded == 0:
            self._raise_error()

    def query_str(self, query_str):
        instances_by_id = {}
        ids_by_instance = {}
        calls = {}

        def to_external_id(python_obj):
            """ Create or look up a polar external_instance for an object """
            if python_obj in ids_by_instance:
                instance_id = ids_by_instance[python_obj]
            else:
                instance_id = lib.polar_get_external_id(self.polar, query)
                if instance_id == 0:
                    self._raise_error()
                instances_by_id[instance_id] = python_obj
                ids_by_instance[python_obj] = instance_id
            return instance_id

        def from_external_id(external_id):
            """ Lookup python object by external_id """
            assert external_id in instances_by_id
            return instances_by_id[external_id]

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
            if tag == "Dictionary":
                return {k: to_python(v) for k, v in value[tag]["fields"].items()}
            if tag == "ExternalInstance":
                return from_external_id(value[tag]["instance_id"])
            return None

        def to_polar(v):
            """ Convert python values to polar terms """
            if isinstance(v, int):
                val = {"Integer": v}
            elif isinstance(v, str):
                val = {"String": v}
            elif isinstance(v, bool):
                val = {"Boolean": v}
            elif isinstance(v, list):
                val = {"List": [to_polar(i) for i in v]}
            elif isinstance(v, dict):
                val = {"Dictionary": {"fields": {k: to_polar(v) for k, v in v.items()}}}
            else:
                instance_id = to_external_id(v)
                val = {"ExternalInstance": {"instance_id": instance_id}}
            term = {"id": 0, "offset": 0, "value": val}
            return term

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

                assert instance_id not in instances_by_id

                class_name = instance["tag"]
                term_fields = instance["fields"]["fields"]

                fields = {}
                for k, v in term_fields.items():
                    fields[k] = to_python(v)

                if class_name not in self.classes:
                    raise PolarException(
                        f"Error creating instance of class {class_name}. Has not been registered."
                    )

                assert class_name in self.classes
                assert class_name in self.class_constructors

                cls = self.classes[class_name]
                constructor = self.class_constructors[class_name]
                if constructor:
                    instance = constructor(**fields)
                else:
                    instance = cls(**fields)

                instances_by_id[instance_id] = instance
                ids_by_instance[instance] = instance_id

            if kind == "ExternalCall":
                call_id = data["call_id"]

                if call_id not in calls:
                    # Create a new call if this is the first use of call_id.
                    instance_id = data["instance_id"]
                    attribute = data["attribute"]
                    args = [to_python(arg) for arg in data["args"]]

                    # Lookup the attribute on the instance.
                    instance = instances_by_id[instance_id]
                    attr = getattr(instance, attribute)

                    if callable(attr):
                        # If it's a function call it with the args.
                        result = getattr(instance, attribute)(*args)
                    else:
                        # If it's just an attribute, it's the result.
                        result = attr

                    # We now have either a generator, a list or a single item as result.
                    # Call must be a generator so we turn anything else into one.
                    if isinstance(result, GeneratorType):
                        call = result
                    else:
                        call = (i for i in [result])

                    calls[call_id] = call

                # Return the next result of the call.
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
