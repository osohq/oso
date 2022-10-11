import pytest
from polar import Variable
from test_app.models import User

from django_oso.oso import Oso, reset_oso
from django_oso.partial import partial_to_query_filter


@pytest.fixture(autouse=True)
def reset():
    reset_oso()


@pytest.mark.django_db
def test_partial_to_query_filter(load_additional_str):
    load_additional_str('ok(_: test_app::User{name:"gwen"});')
    gwen = User(name="gwen")
    gwen.save()
    steve = User(name="steve")
    steve.save()
    result = Oso.query_rule("ok", Variable("actor"), accept_expression=True)

    partial = next(result)["bindings"]["actor"]
    filter = partial_to_query_filter(partial, User)
    q = list(User.objects.filter(filter))
    assert q == [gwen]
