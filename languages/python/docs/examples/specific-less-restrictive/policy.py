import oso

class General:
    pass

class Specific(General):
    pass

def setup(oso):
    oso.register_class(General)
    oso.register_class(Specific)
