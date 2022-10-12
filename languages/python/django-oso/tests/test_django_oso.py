from pathlib import Path

import pytest
from django.core.exceptions import EmptyResultSet, PermissionDenied
from oso import OsoError
from polar import Expression, Variable

from django_oso.auth import authorize, authorize_model
from django_oso.oso import Oso, reset_oso
from django_oso.partial import TRUE_FILTER

from .conftest import negated_condition


@pytest.fixture(autouse=True)
def reset():
    reset_oso()


@pytest.fixture
def simple_policy(load_additional_str):
    """Load simple authorization policy."""
    with open(Path(__file__).parent / "simple.polar", "rb") as f:
        contents = f.read().decode("utf-8")
        load_additional_str(contents)


@pytest.fixture
def partial_policy(load_additional_str):
    """Load partial authorization policy."""
    with open(Path(__file__).parent / "partial.polar", "rb") as f:
        contents = f.read().decode("utf-8")
        load_additional_str(contents)


def test_policy_autoload():
    """Test that policies are loaded from policy directory."""
    # These rules are added by the policies in the test app.
    assert next(Oso.query_rule("policy_load_test", 1))
    assert next(Oso.query_rule("policy_load_test", 2))


def test_model_registration():
    """Test that models are automatically registered with the policy."""
    from oso import Variable
    from test_app import models

    assert (
        next(Oso.query_rule("models", models.TestRegistration(), Variable("x")))[
            "bindings"
        ]["x"]
        == 1
    )
    assert (
        next(Oso.query_rule("models", models.TestRegistration2(), Variable("x")))[
            "bindings"
        ]["x"]
        == 2
    )


def test_authorize(rf, simple_policy):
    """Test that authorize function works."""
    request = rf.get("/")

    # No defaults
    authorize(request, actor="user", action="read", resource="resource")

    # Default action
    authorize(request, actor="user", resource="action_resource")

    # Default actor
    request.user = "user"
    authorize(request, resource="action_resource")

    # Not authorized
    with pytest.raises(PermissionDenied):
        authorize(request, "resource", actor="other", action="read")


def test_require_authorization(client, settings, simple_policy):
    """Test that require authorization middleware works."""
    settings.MIDDLEWARE.append("django_oso.middleware.RequireAuthorization")

    with pytest.raises(OsoError):
        response = client.get("/")

    response = client.get("/auth/")
    assert response.status_code == 200

    response = client.get("/auth_decorated_fail/")
    assert response.status_code == 403

    response = client.get("/auth_decorated/")
    assert response.status_code == 200

    # 404 gets through
    response = client.get("/notfound/")
    assert response.status_code == 404

    # 500 gets through
    response = client.get("/error/")
    assert response.status_code == 500


def test_route_authorization(client, settings, simple_policy, load_additional_str):
    """Test route authorization middleware"""
    settings.MIDDLEWARE.append("django.contrib.sessions.middleware.SessionMiddleware")
    settings.MIDDLEWARE.append(
        "django.contrib.auth.middleware.AuthenticationMiddleware"
    )
    settings.MIDDLEWARE.append("django_oso.middleware.RouteAuthorization")

    response = client.get("/a/")
    assert response.status_code == 403

    response = client.get("/b/")
    assert response.status_code == 403

    load_additional_str('allow(_, "GET", _: HttpRequest{path: "/a/"});')
    response = client.get("/a/")
    assert response.status_code == 200

    response = client.post("/a/")
    assert response.status_code == 403

    # Django runs url resolving after middleware, so there is no way
    # for the route authorization middleware to determine if the response
    # would be a 404 and not apply authorization.
    response = client.get("/notfound/")
    assert response.status_code == 403


@pytest.mark.django_db
def test_partial(rf, partial_policy):
    from test_app.models import Post

    posts = [
        Post(name="test", is_private=False, timestamp=1).save(),
        Post(name="test_past", is_private=False, timestamp=-1).save(),
        Post(name="test_public", is_private=False, timestamp=1).save(),
        Post(name="test_private", is_private=True, timestamp=1).save(),
        Post(name="test_private_2", is_private=True, timestamp=1).save(),
        Post(name="test_option", is_private=False, timestamp=1, option=True).save(),
    ]

    request = rf.get("/")
    request.user = "test_user"

    authorize_filter = authorize_model(request, action="get", model=Post)
    assert (
        str(authorize_filter)
        == "(AND: ('is_private', False), ('timestamp__gt', 0), ('option', None))"
    )

    q = Post.objects.filter(authorize_filter)
    bool_cond = negated_condition('"test_app_post"."is_private"')
    expected = f"""
        SELECT "test_app_post"."id", "test_app_post"."is_private", "test_app_post"."name",
               "test_app_post"."timestamp", "test_app_post"."option", "test_app_post"."created_by_id"
        FROM "test_app_post"
        WHERE ({bool_cond}
               AND "test_app_post"."timestamp" > 0
               AND "test_app_post"."option" IS NULL)
    """
    assert str(q.query) == " ".join(expected.split())
    assert q.count() == 2

    request = rf.get("/")
    request.user = "test_admin"

    authorize_filter = authorize_model(request, action="get", model=Post)
    assert str(authorize_filter) == str(TRUE_FILTER)

    q = Post.objects.filter(authorize_filter)
    expected = """
        SELECT "test_app_post"."id", "test_app_post"."is_private", "test_app_post"."name",
               "test_app_post"."timestamp", "test_app_post"."option", "test_app_post"."created_by_id"
        FROM "test_app_post"
    """
    assert str(q.query) == " ".join(expected.split())
    assert q.count() == len(posts)

    q = Post.objects.authorize(request, action="get")
    assert q.count() == len(posts)


@pytest.mark.django_db
def test_partial_isa_with_path(load_additional_str):
    from test_app.models import Post, User

    alice = User(name="alice")
    alice.save()
    not_alice = User(name="not alice")
    not_alice.save()

    Post(created_by=alice).save(),
    Post(created_by=not_alice).save(),
    Post(created_by=alice).save(),

    load_additional_str(
        """
            allow(_, _, post: test_app::Post) if check(post.created_by);
            check(user: test_app::User) if user.name = "alice";
            check(post: test_app::Post) if post.is_private = false;
        """
    )

    authorize_filter = authorize_model(None, Post, actor="foo", action="bar")
    assert str(authorize_filter) == "(AND: ('created_by__name', 'alice'))"
    authorized_posts = Post.objects.filter(authorize_filter)
    expected = """
        SELECT "test_app_post"."id", "test_app_post"."is_private", "test_app_post"."name",
               "test_app_post"."timestamp", "test_app_post"."option", "test_app_post"."created_by_id"
        FROM "test_app_post"
        INNER JOIN "test_app_user" ON ("test_app_post"."created_by_id" = "test_app_user"."id")
        WHERE "test_app_user"."name" = alice
    """
    assert str(authorized_posts.query) == " ".join(expected.split())
    assert authorized_posts.count() == 2


@pytest.mark.django_db
def test_authorize_query_no_access(rf, load_additional_str):
    from test_app.models import Post

    Post(name="test", is_private=False, timestamp=1).save()
    Post(name="test_past", is_private=False, timestamp=-1).save()
    Post(name="test_public", is_private=False, timestamp=1).save()
    Post(name="test_private", is_private=True, timestamp=1).save()
    Post(name="test_private_2", is_private=True, timestamp=1).save()

    request = rf.get("/")
    request.user = "test_user"

    # No matching rules for Post.
    load_additional_str("allow(_, _, _: test_app::User);")

    q = Post.objects.authorize(request, action="get")
    assert q.count() == 0


@pytest.mark.django_db
def test_null_with_partial(rf, load_additional_str):
    from test_app.models import Post

    Post(name="test", is_private=False, timestamp=1).save()
    load_additional_str("allow(_, _, post: test_app::Post) if post.option = nil;")
    request = rf.get("/")
    request.user = "test_user"

    authorize_filter = authorize_model(request, Post)
    assert str(authorize_filter) == "(AND: ('option', None))"
    authorized_posts = Post.objects.filter(authorize_filter)
    expected = """
        SELECT "test_app_post"."id", "test_app_post"."is_private", "test_app_post"."name",
               "test_app_post"."timestamp", "test_app_post"."option", "test_app_post"."created_by_id"
        FROM "test_app_post"
        WHERE "test_app_post"."option" IS NULL
    """
    assert str(authorized_posts.query) == " ".join(expected.split())
    assert authorized_posts.count() == 1


@pytest.mark.django_db
def test_negated_matches_with_partial(rf, load_additional_str):
    from test_app.models import Post

    Post(name="test", is_private=False, timestamp=1).save()
    load_additional_str(
        """
        allow(1, _, post) if not post matches test_app::Post;
        allow(2, _, post) if not post matches test_app::User;
        allow(3, _, post) if not post.created_by matches test_app::User;
        allow(4, _, post) if not post.created_by matches test_app::Post;
        """
    )
    request = rf.get("/")

    request.user = 1
    authorize_filter = authorize_model(request, Post)
    assert str(authorize_filter) == (f"(NOT (AND: {str(TRUE_FILTER)}))")
    authorized_posts = Post.objects.filter(authorize_filter)
    # For some reason, this only seems to be raised when stringifying.
    with pytest.raises(EmptyResultSet):
        str(authorized_posts.query)
    assert authorized_posts.count() == 0

    request.user = 2
    authorize_filter = authorize_model(request, Post)
    assert str(authorize_filter) == str(TRUE_FILTER)
    authorized_posts = Post.objects.filter(authorize_filter)
    expected = """
        SELECT "test_app_post"."id", "test_app_post"."is_private", "test_app_post"."name",
               "test_app_post"."timestamp", "test_app_post"."option", "test_app_post"."created_by_id"
        FROM "test_app_post"
    """
    assert str(authorized_posts.query) == " ".join(expected.split())
    assert authorized_posts.count() == 1

    request.user = 3
    authorize_filter = authorize_model(request, Post)
    assert str(authorize_filter) == (f"(NOT (AND: {str(TRUE_FILTER)}))")
    authorized_posts = Post.objects.filter(authorize_filter)
    # For some reason, this only seems to be raised when stringifying.
    with pytest.raises(EmptyResultSet):
        str(authorized_posts.query)
    assert authorized_posts.count() == 0

    request.user = 4
    authorize_filter = authorize_model(request, Post)
    assert str(authorize_filter) == str(TRUE_FILTER)
    authorized_posts = Post.objects.filter(authorize_filter)
    expected = """
        SELECT "test_app_post"."id", "test_app_post"."is_private", "test_app_post"."name",
               "test_app_post"."timestamp", "test_app_post"."option", "test_app_post"."created_by_id"
        FROM "test_app_post"
    """
    assert str(authorized_posts.query) == " ".join(expected.split())
    assert authorized_posts.count() == 1


def test_partial_unification(load_additional_str):
    load_additional_str("f(x, y) if x = y and x = 1;")
    results = Oso.query_rule("f", Variable("x"), Variable("y"), accept_expression=True)
    first = next(results)["bindings"]
    assert first["x"] == 1
    assert first["y"] == 1

    with pytest.raises(StopIteration):
        next(results)

    load_additional_str("g(x, y) if x = y and y > 1;")
    results = Oso.query_rule("g", Variable("x"), Variable("y"), accept_expression=True)
    first = next(results)["bindings"]

    # TODO not ideal that these are swapped in order (y = x) not (x = y).
    # this is a hard case, we want the (y > 1) to be this in both cases AND keep the x = y.
    assert first["x"] == Expression(
        "And",
        [
            Expression("Unify", [Variable("y"), Variable("_this")]),
            Expression("Gt", [Variable("y"), 1]),
        ],
    )
    assert first["y"] == Expression(
        "And",
        [
            Expression("Unify", [Variable("_this"), Variable("x")]),
            Expression("Gt", [Variable("_this"), 1]),
        ],
    )


def test_rewrite_parameters(load_additional_str):
    from test_app.models import Post

    load_additional_str(
        """allow(_, _, resource) if g(resource.created_by);
           g(resource) if resource matches test_app::User;
        """
    )
    authorize_filter = authorize_model(None, Post, actor="foo", action="bar")
    assert str(authorize_filter) == str(TRUE_FILTER)


@pytest.mark.django_db
def test_partial_with_allow_all(rf, load_additional_str):
    from test_app.models import Post

    Post(name="test", is_private=False, timestamp=1).save()
    load_additional_str("allow(_, _, _);")
    request = rf.get("/")
    request.user = "test_user"

    authorize_filter = authorize_model(request, Post)
    assert str(authorize_filter) == str(TRUE_FILTER)
    authorized_posts = Post.objects.filter(authorize_filter)
    expected = """
        SELECT "test_app_post"."id", "test_app_post"."is_private", "test_app_post"."name",
               "test_app_post"."timestamp", "test_app_post"."option", "test_app_post"."created_by_id"
        FROM "test_app_post"
    """
    assert str(authorized_posts.query) == " ".join(expected.split())
    assert authorized_posts.count() == 1


def test_unconditional_policy_has_no_filter(load_additional_str):
    from test_app.models import Post

    load_additional_str(
        'allow("user", "read", post: test_app::Post) if post.id = 1; allow(_, _, _);'
    )
    authorize_filter = authorize_model(None, Post, actor="user", action="read")
    assert str(authorize_filter) == str(TRUE_FILTER)
    authorized_posts = Post.objects.filter(authorize_filter)
    expected = """
        SELECT "test_app_post"."id", "test_app_post"."is_private", "test_app_post"."name",
               "test_app_post"."timestamp", "test_app_post"."option", "test_app_post"."created_by_id"
        FROM "test_app_post"
    """
    assert str(authorized_posts.query) == " ".join(expected.split())
