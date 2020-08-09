import os
from oso import polar_class


@polar_class
class Env:
    @staticmethod
    def var(variable):
        yield os.environ[variable]
