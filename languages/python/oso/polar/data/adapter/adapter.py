class DataAdapter:
    def build_query(self, _filter):
        raise NotImplementedError

    def exec_query(self, _query):
        raise NotImplementedError
