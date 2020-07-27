import os
from oso import polar_class


@polar_class
class Env:
    def var(self, variable):
        yield os.environ[variable]
