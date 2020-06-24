import oso

from pathlib import Path

class OsoModel:
    pass

class OsoModel2:
    pass

class NotOsoModel:
    pass

def setup(oso):
    oso.load_file(Path(__file__).parent / 'policy.polar')

    oso.register_class(OsoModel)
    oso.register_class(OsoModel2)
    oso.register_class(NotOsoModel)

def auth(oso, actor, action, resource, next):
    if not oso.query_predicate("is_checked", resource).success:
        return next(actor, action, resource)
    else:
        return oso.allow(actor, action, resource)
