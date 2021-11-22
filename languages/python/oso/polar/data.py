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

    # todo: hash joins would be better for big arrays but this is mostly used for small tests.
    def build_query(self, host, types, filter):
        assert filter['root'] in types
        typ = filter['root']

        records = []
        def join(obj, record, relations):
            if relations == []:
                records.append(record)
                return
            relation, rest_relations = relations[0], relations[1:]
            _, name, other_typ = relation
            rel = types[typ][name]['Relation']
            rel_typ = rel['other_class_tag']
            other_array = self.type_arrays[rel_typ]
            for j, j_obj in enumerate(other_array):
                my_field = getattr(obj, rel['my_field'])
                other_field = getattr(j_obj, rel['other_field'])
                if my_field == other_field:
                    joined_record = dict(record)
                    joined_record[rel_typ] = j
                    join(obj, joined_record, rest_relations)
                    if rel['kind'] == 'one':
                        break
        
        array = self.type_arrays[typ]
        for i, obj in enumerate(array):
            record = {typ: i}
            relations = filter['relations']
            join(obj, record, relations)

        results = set()
        for conditions in filter['conditions']:
            for rec in records:
                test = True
                for (lhs, op, rhs) in conditions:
                    kind = next(iter(lhs))
                    if kind == 'Field':
                        (t, f) = lhs[kind]
                        i = rec[t]
                        obj = self.type_arrays[t][i]
                        v = getattr(obj, f)
                        l = v
                    elif kind == 'Imm':
                        v = lhs[kind]
                        l = host.to_python({'value': v})

                    kind = next(iter(rhs))
                    if kind == 'Field':
                        (t, f) = rhs[kind]
                        i = rec[t]
                        obj = self.type_arrays[t][i]
                        v = getattr(obj, f)
                        r = v
                    elif kind == 'Imm':
                        v = rhs[kind]
                        r = host.to_python({'value': v})

                    if op == 'Eq':
                        test &= l == r
                    else:
                        raise NotImplementedError
                if test:
                    results.add(self.type_arrays[typ][rec[typ]])
        return results
    
    def execute_query(self, host, query):
        return list(query)


class SqlalchemyAdapter(Adapter):
    def build_query(self, types, filter):
        pass
    
    def execute_query(self, query):
        pass