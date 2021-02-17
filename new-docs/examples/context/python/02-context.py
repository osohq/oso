import os
from oso import polar_class


# context-start
@polar_class
class Env:
    @staticmethod
    def var(variable):
        return os.environ[variable]
        # context-end
