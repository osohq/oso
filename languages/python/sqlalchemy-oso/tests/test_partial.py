from polar import Variable
from sqlalchemy.orm import Session

from sqlalchemy_oso.partial import partial_to_filter

from .models import User


def test_partial_to_query_filter(oso, engine):
    oso.load_str('ok(_: User{username:"gwen"});')
    session = Session(bind=engine)
    gwen = User(username="gwen")
    session.add(gwen)
    steve = User(username="steve")
    session.add(steve)
    result = oso.query_rule("ok", Variable("actor"), accept_expression=True)

    partial = next(result)["bindings"]["actor"]
    filter = partial_to_filter(partial, session, User, oso.get_class)
    q = list(session.query(User).filter(filter))
    assert q == [gwen]
