from _polar_lib import ffi, lib
import json

class Polar:
    def __init__(self):
        self.polar = lib.polar_new()

    def destroy(self):
        # TODO
        pass
        # Not usually needed but useful for tests since we make a lot of these.
        # lib.polar_free(self.polar)

    def load_str(self, src_str):
        c_str = ffi.new("char[]", src_str.encode())
        lib.polar_load_str(self.polar, c_str)

    def query(self, query_str):
        query_pred = '{"name":"foo","args":[{"id":2,"value":{"Integer":0}}]}'
        c_pred = ffi.new("char[]", query_pred.encode())
        query = lib.query_new_from_pred(c_pred)

        while True:
            event_s = lib.polar_query(self.polar, query)
            event = json.loads(ffi.string(event_s).decode())
            if event == "Done":
                break

            kind = [*event][0]
            data = event[kind]

            if kind == "Result":
                yield data["environment"]["bindings"]

            # if kind == "ExternalCall":
            #     name = data["name"]
            #     args = data["args"]
            #
            #     results = self.external_functions[name](*args)
            #     for result in results:
            #         lib.polar_external_result(
            #             self.polar, query, ffi.new("char[]", result.encode())
            #         )
