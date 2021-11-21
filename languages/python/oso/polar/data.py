# New interface for an adapter.
# Only does things needed for authorized_resources and authorized_query
# TODO: Authorized session/database

class Adapter:
    def build_query(self, types, filter):
        """
        Takes in a Filter and produces a query. What the query object is would depend on the adapter
        but examples could include a sql query or a django Filter.
        """
        raise NotImplementedError

    def execute_query(self, query):
        raise NotImplementedError

# Todo: These maybe go in other packages or are included as optional features.
class ArrayAdapter(Adapter):
    """
    This is a simple built in adapter for filtering arrays of data. It assumes that there is
    an array for each type and takes a dict from class to array in the constructor.
    """
    def __init__(self, type_arrays):
        self.type_arrays = type_arrays

    def build_query(self, types, filter):
        pass
    
    def execute_query(self, query):
        pass


class SqlalchemyAdapter(Adapter):
    def build_query(self, types, filter):
        pass
    
    def execute_query(self, query):
        pass