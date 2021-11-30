class DataAdapter():
    def build_query(self, _filter):
        raise NotImplemented

    def exec_query(self, _query):
        raise NotImplemented
