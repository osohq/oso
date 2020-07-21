import os
from polar import polar_class

@polar_class
class Env:
    def var(self, variable):
        yield os.environ[variable]
