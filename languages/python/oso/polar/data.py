# New interface for an adapter.
# Only does things needed for authorized_resources and authorized_query
# TODO: Authorized session/database

# todo: build a filter object instead of passing the json through
# that way we can parse everything to python.
class Filter:
    pass

class Adapter:
    # host is only needed to call topolar so maybe we just handle that
    # as a preprocess step for the filter instead of passing it in here.
    def build_query(self, host, types, filter):
        """
        Takes in a Filter and produces a query. What the query object is would depend on the adapter
        but examples could include a sql query or a django Filter.
        """
        raise NotImplementedError

    def execute_query(self, host, query):
        raise NotImplementedError

# Todo: These maybe go in other packages or are included as optional features.
class ArrayAdapter(Adapter):
    """
    This is a simple built in adapter for filtering arrays of data. It assumes that there is
    an array for each type and takes a dict from type name to array in the constructor.
    It requires objects in the arrays to be hashable.
    """
    def __init__(self, type_arrays):
        self.type_arrays = type_arrays

    def get_val(self, host, typ, obj, val):
        kind = next(iter(val))
        if kind == 'Field':
            (t, f) = val[kind]
            assert t == typ
            v = getattr(obj, f)
            return v
        elif kind == 'Imm':
            v = val[kind]
            return host.to_python({'value': v})


    def build_query(self, host, types, filter):
        assert filter['root'] in types
        typ = filter['root']

        # todo: joins
        assert filter['relations'] == []

        array = self.type_arrays[typ]
        results = set()
        for conditions in filter['conditions']:
            for obj in array:
                test = True
                for (lhs, op, rhs) in conditions:
                    l = self.get_val(host, typ, obj, lhs)
                    r = self.get_val(host, typ, obj, rhs)
                    if op == 'Eq':
                        test &= l == r
                    else:
                        raise NotImplementedError
                if test:
                    results.add(obj)
        return results
                    
    
    def execute_query(self, host, query):
        return list(query)


class SqlalchemyAdapter(Adapter):
    def build_query(self, types, filter):
        pass
    
    def execute_query(self, query):
        pass