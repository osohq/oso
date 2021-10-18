import pytest

from polar import Variable
from django_oso.oso import Oso, reset_oso
from django_oso.partial import partial_to_query_filter
from test_app2.models import Post, User


@pytest.fixture(autouse=True)
def reset():
    reset_oso()


@pytest.mark.xfail(reason="a bug in partial_to_query_filter")
@pytest.mark.django_db
def test_another_one(load_additional_str):
    load_additional_str(
        """
        allow(actor: test_app2::User, "get", post: test_app2::Post) if
            post.created_by.is_banned = false and
            post.created_by = actor;
        allow(actor: test_app2::User, "put", post: test_app2::Post) if
            post.created_by = actor and
            post.created_by.is_banned = false;
    """
    )

    gwen = User(username="gwen")
    gwen.save()
    steve = User(username="steve")
    steve.save()
    gwens_post = Post(created_by=gwen)
    gwens_post.save()
    steves_post = Post(created_by=steve)
    steves_post.save()

    def authzd_posts(user, action):
        result = Oso.query_rule(
            "allow", user, action, Variable("post"), accept_expression=True
        )
        partial = next(result)["bindings"]["post"]
        filter = partial_to_query_filter(partial, Post)
        return list(Post.objects.filter(filter))

    assert authzd_posts(gwen, "put") == [gwens_post]
    assert authzd_posts(steve, "put") == [steves_post]

    assert authzd_posts(gwen, "get") == [gwens_post]
    assert authzd_posts(steve, "get") == [steves_post]


@pytest.mark.django_db
def test_partial_to_query_filter(load_additional_str):
    load_additional_str('ok(_: test_app2::User{username:"gwen"});')
    gwen = User(username="gwen")
    gwen.save()
    steve = User(username="steve")
    steve.save()
    result = Oso.query_rule("ok", Variable("actor"), accept_expression=True)

    partial = next(result)["bindings"]["actor"]
    filter = partial_to_query_filter(partial, User)
    q = list(User.objects.filter(filter))
    assert q == [gwen]
