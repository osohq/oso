import os
import polar

class Env:
    def var(self, variable):
        yield os.environ[variable]

polar.register_python_class(Env)
