"""Test hooks & SQLAlchemy API integrations."""
import pytest

from sqlalchemy.orm import aliased

from sqlalchemy_oso.session import (
    authorized_sessionmaker,
    scoped_session,
    AuthorizedSession,
)

from .models import User, Post
from .conftest import print_query


def log_queries():
    import logging

    logging.basicConfig()
    logging.getLogger("sqlalchemy.engine").setLevel(logging.INFO)


def test_authorize_query_no_access(engine, oso, fixture_data):
    session = AuthorizedSession(oso, "user", "action", bind=engine)
    query = session.query(Post)

    assert query.count() == 0


@pytest.mark.parametrize(
    "query",
    [
        lambda session: session.query(Post),
        lambda session: session.query(Post.contents, Post.id),
    ],
)
def test_authorize_query_basic(engine, oso, fixture_data, query):
    # TODO: copied from test_authorize_model_basic
    oso.load_str('allow("user", "read", post: Post) if post.access_level = "public";')
    oso.load_str('allow("user", "write", post: Post) if post.access_level = "private";')
    oso.load_str('allow("admin", "read", post: Post);')
    oso.load_str(
        'allow("moderator", "read", post: Post) if '
        '(post.access_level = "private" or post.access_level = "public") and '
        "post.needs_moderation = true;"
    )

    session = AuthorizedSession(oso, "user", "read", bind=engine)
    authorized = query(session)

    assert authorized.count() == 5
    assert authorized.all()[0].contents == "foo public post"
    assert authorized.all()[0].id == 0

    session = AuthorizedSession(oso, "user", "write", bind=engine)
    posts = query(session)

    assert posts.count() == 4
    assert posts.all()[0].contents == "foo private post"
    assert posts.all()[1].contents == "foo private post 2"

    session = AuthorizedSession(oso, "admin", "read", bind=engine)
    posts = query(session)
    assert posts.count() == 9

    session = AuthorizedSession(oso, "moderator", "read", bind=engine)
    posts = query(session)
    print_query(posts)
    assert posts.all()[0].contents == "private for moderation"
    assert posts.all()[1].contents == "public for moderation"

    session = AuthorizedSession(oso, "guest", "read", bind=engine)
    posts = query(session)
    assert posts.count() == 0


def test_authorize_query_multiple_types(engine, oso, fixture_data):
    """Test a query involving multiple models."""
    oso.load_str('allow("user", "read", post: Post) if post.id = 1;')
    oso.load_str('allow("user", "read", user: User) if user.id = 0;')
    oso.load_str('allow("user", "read", user: User) if user.id = 1;')
    oso.load_str('allow("all_posts", "read", _: Post);')

    # Query two models. Only return authorized objects from each (no join).
    session = AuthorizedSession(oso, "user", "read", bind=engine)
    authorized = session.query(Post, User)
    print_query(authorized)
    assert authorized.count() == 2
    assert authorized[0][0].id == 1
    assert authorized[0][1].id == 0
    assert authorized[1][1].id == 1

    # Query two models, with a join condition. Only return authorized objects that meet the join
    # condition.
    authorized = session.query(Post, User.username).join(User)
    print_query(authorized)
    assert authorized.count() == 1
    assert authorized[0][0].id == 1
    assert authorized[0][1] == "foo"

    # Join, but only return fields from one model.
    authorized = session.query(Post).join(User)

    # This one is odd... we don't return any fields from the User model,
    # so no authorization is applied for users.
    print_query(authorized)
    assert authorized.count() == 1

    # Another odd one.  We are joining on user, and filtering on fields on user.
    # But, no authorization filter is applied for user because no fields of user
    # are returned.
    # Could this leak data somehow? Maybe if users are allowed to filter arbitrary
    # values and see a count, but not retrieve the objects?
    session = AuthorizedSession(oso, "all_posts", "read", bind=engine)
    authorized = session.query(Post).join(User).filter(User.username == "admin_user")
    print_query(authorized)
    assert authorized.count() == 2

    # TODO (dhatch): What happens for aggregations?


def test_alias(engine, oso, fixture_data):
    oso.load_str('allow("user", "read", post: Post) if post.id = 1;')
    session = AuthorizedSession(oso, user="user", action="read", bind=engine)

    post_alias = aliased(Post)

    query = session.query(post_alias)

    # Fine with hooks if you don't authorize it.
    assert query.count() == 1


def test_authorized_sessionmaker_relationship(engine, oso, fixture_data):
    oso.load_str('allow("user", "read", post: Post) if post.id = 1;')
    # Post with creator id = 1
    oso.load_str('allow("user", "read", post: Post) if post.id = 7;')
    oso.load_str('allow("user", "read", user: User) if user.id = 0;')

    Session = authorized_sessionmaker(
        get_oso=lambda: oso,
        get_user=lambda: "user",
        get_action=lambda: "read",
        bind=engine,
    )

    session = Session()

    posts = session.query(Post)
    assert posts.count() == 2

    users = session.query(User)
    assert users.count() == 1

    post_1 = posts.get(1)
    # Authorized created by field.
    assert post_1.created_by == users.get(0)

    post_7 = posts.get(7)
    # created_by isn't actually none, but we can't see it
    assert post_7.created_by is None


def test_authorized_session_relationship(engine, oso, fixture_data):
    oso.load_str('allow("user", "read", post: Post) if post.id = 1;')
    # Post with creator id = 1
    oso.load_str('allow("user", "read", post: Post) if post.id = 7;')
    oso.load_str('allow("user", "read", user: User) if user.id = 0;')

    session = AuthorizedSession(
        oso=oso,
        user="user",
        action="read",
        bind=engine,
    )

    posts = session.query(Post)
    assert posts.count() == 2

    users = session.query(User)
    assert users.count() == 1

    post_1 = posts.get(1)
    # Authorized created by field.
    assert post_1.created_by == users.get(0)

    post_7 = posts.get(7)
    # created_by isn't actually none, but we can't see it
    assert post_7.created_by is None


def test_scoped_session_relationship(engine, oso, fixture_data):
    oso.load_str('allow("user", "read", post: Post) if post.id = 1;')
    # Post with creator id = 1
    oso.load_str('allow("user", "read", post: Post) if post.id = 7;')
    oso.load_str('allow("user", "read", user: User) if user.id = 0;')
    oso.load_str('allow("other", "read", post: Post) if post.id = 3;')
    oso.load_str('allow("other", "write", post: Post) if post.id = 4;')

    data = {"user": "user", "action": "read"}
    session = scoped_session(lambda: oso, lambda: data["user"], lambda: data["action"])
    session.configure(bind=engine)

    posts = session.query(Post)
    assert posts.count() == 2

    users = session.query(User)
    assert users.count() == 1

    post_1 = posts.get(1)
    # Authorized created by field.
    assert post_1.created_by == users.get(0)

    post_7 = posts.get(7)
    # created_by isn't actually none, but we can't see it
    assert post_7.created_by is None
    assert len(session.identity_map.values()) == 3

    data["user"] = "other"

    # Ensure this changed the session.
    assert len(session.identity_map.values()) == 0
    posts = session.query(Post)
    assert posts.count() == 1
    posts = posts.all()
    assert posts[0].id == 3
    assert len(session.identity_map.values()) == 1

    data["action"] = "write"
    assert len(session.identity_map.values()) == 0
    posts = session.query(Post)
    assert posts.count() == 1
    posts = posts.all()
    assert posts[0].id == 4
    assert len(session.identity_map.values()) == 1

    # Change back to original.
    data = {"user": "user", "action": "read"}
    assert len(session.identity_map.values()) == 3


@pytest.mark.xfail(reason="Implemented incorrectly initially. Fix")
def test_authorized_sessionmaker_user_change(engine, oso, fixture_data):
    """Ensure that query fails if the user changes."""
    oso.load_str('allow("user", "read", post: Post) if post.id = 1;')
    user = ["user"]

    Session = authorized_sessionmaker(
        get_oso=lambda: oso,
        get_user=lambda: user[0],
        get_action=lambda: "read",
        bind=engine,
    )

    session = Session()

    posts = session.query(Post).count()
    assert posts == 1

    user[0] = "moderator"

    with pytest.raises(Exception, match="user"):
        posts = session.query(Post).count()


def test_null_with_partial(engine, oso):
    oso.load_str("allow(_, _, post: Post) if post.contents = nil;")
    Session = authorized_sessionmaker(
        get_oso=lambda: oso,
        get_user=lambda: "user",
        get_action=lambda: "read",
        bind=engine,
    )
    posts = Session().query(Post)

    assert str(posts) == (
        "SELECT posts.id AS posts_id, posts.contents AS posts_contents, "
        + "posts.access_level AS posts_access_level, posts.created_by_id AS posts_created_by_id, "
        + "posts.needs_moderation AS posts_needs_moderation \nFROM posts \nWHERE posts.contents IS NULL"
    )
    assert posts.count() == 0


def test_regular_session(engine, oso, fixture_data):
    """Test that a regular session doesn't apply authorization."""
    from sqlalchemy.orm import Session

    session = Session(bind=engine)
    posts = session.query(Post)

    # No posts would be returned if authorization was applied.
    assert posts.count() == 9
