import os

# context-start
class Env:
    @staticmethod
    def var(variable):
        return os.environ[variable]
        # context-end
