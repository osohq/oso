class DataAdapter:
    def build_query(self, _filter):
        raise NotImplementedError

    def execute_query(self, _query):
        raise NotImplementedError
