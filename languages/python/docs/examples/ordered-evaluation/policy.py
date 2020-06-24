import pathlib

import oso

class ComplicatedResource:
    def __init__(self, unrestricted=False):
        self.unrestricted = unrestricted

def setup(oso):
    oso.register_class(ComplicatedResource)
    oso.load_file(pathlib.Path(__file__).parent / 'policy.polar')
