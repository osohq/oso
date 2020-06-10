import os
from polar import api


class Env:
    def var(self, variable):
        yield os.environ[variable]


api.register_class(Env)
