"""Test hooks & SQLAlchemy API integrations."""
import pytest
from sqlalchemy.orm import aliased, joinedload

from sqlalchemy_oso.compat import USING_SQLAlchemy_v1_3
from sqlalchemy_oso.session import (
    AuthorizedSession,
    authorized_sessionmaker,
    scoped_session,
)

from .conftest import print_query
from .models import Post, User


def log_queries():
    import logging

    logging.basicConfig()
    logging.getLogger("sqlalchemy.engine").setLevel(logging.INFO)


def test_authorize_query_no_access(engine, oso, fixture_data):
    # No matching rules for Post.
    oso.load_str("allow(_, _, _: User);")
    session = AuthorizedSession(oso, "user", {Post: "action"}, bind=engine)
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
    oso.load_str(
        """allow("user", "read", post: Post) if post.access_level = "public";
           allow("user", "write", post: Post) if post.access_level = "private";
           allow("admin", "read", _post: Post);
           allow("moderator", "read", post: Post) if
             (post.access_level = "private" or post.access_level = "public") and
             post.needs_moderation = true;"""
    )

    session = AuthorizedSession(oso, "user", {Post: "read"}, bind=engine)
    authorized = query(session)

    assert authorized.count() == 5
    assert authorized.all()[0].contents == "foo public post"
    assert authorized.all()[0].id == 0

    session = AuthorizedSession(oso, "user", {Post: "write"}, bind=engine)
    posts = query(session)

    assert posts.count() == 4
    assert posts.all()[0].contents == "foo private post"
    assert posts.all()[1].contents == "foo private post 2"

    session = AuthorizedSession(oso, "admin", {Post: "read"}, bind=engine)
    posts = query(session)
    assert posts.count() == 9

    session = AuthorizedSession(oso, "moderator", {Post: "read"}, bind=engine)
    posts = query(session)
    print_query(posts)
    assert posts.all()[0].contents == "private for moderation"
    assert posts.all()[1].contents == "public for moderation"

    session = AuthorizedSession(oso, "guest", {Post: "read"}, bind=engine)
    posts = query(session)
    assert posts.count() == 0


def test_authorize_query_multiple_types(engine, oso, fixture_data):
    """Test a query involving multiple models."""
    oso.load_str(
        """allow("user", "read", post: Post) if post.id = 1;
           allow("user", "read", user: User) if user.id = 0;
           allow("user", "read", user: User) if user.id = 1;
           allow("all_posts", "read", _: Post);"""
    )

    # Query two models. Only return authorized objects from each (no join).
    session = AuthorizedSession(oso, "user", {Post: "read", User: "read"}, bind=engine)
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
    session = AuthorizedSession(oso, "all_posts", {Post: "read"}, bind=engine)
    authorized = session.query(Post).join(User).filter(User.username == "admin_user")
    print_query(authorized)
    assert authorized.count() == 2

    # TODO (dhatch): What happens for aggregations?


@pytest.mark.xfail(USING_SQLAlchemy_v1_3, reason="Not supported by 1.3 events API.")
def test_authorize_query_joined_load(engine, oso, fixture_data):
    """Test a query involving multiple models."""
    oso.load_str(
        """allow("user", "read", post: Post) if post.id = 1;
           allow("user", "read", user: User) if user.id = 0;
           allow("user", "read", user: User) if user.id = 1;
           allow("all_posts", "read", _: Post);"""
    )

    session = AuthorizedSession(oso, "user", {Post: "read", User: "read"}, bind=engine)
    authorized = session.query(User).options(joinedload(User.posts))
    print_query(authorized)
    print(authorized[0].posts)
    assert len(authorized[0].posts) == 1


def test_alias(engine, oso, fixture_data):
    oso.load_str('allow("user", "read", post: Post) if post.id = 1;')
    session = AuthorizedSession(
        oso, user="user", checked_permissions={Post: "read"}, bind=engine
    )

    post_alias = aliased(Post)

    query = session.query(post_alias)

    # Fine with hooks if you don't authorize it.
    assert query.count() == 1


def test_authorized_sessionmaker_relationship(engine, oso, fixture_data):
    oso.load_str(
        """allow("user", "read", post: Post) if post.id = 1;
           # Post with creator id = 1
           allow("user", "read", post: Post) if post.id = 7;
           allow("user", "read", user: User) if user.id = 0;"""
    )

    Session = authorized_sessionmaker(
        get_oso=lambda: oso,
        get_user=lambda: "user",
        get_checked_permissions=lambda: {Post: "read", User: "read"},
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
    oso.load_str(
        """allow("user", "read", post: Post) if post.id = 1;
           # Post with creator id = 1
           allow("user", "read", post: Post) if post.id = 7;
           allow("user", "read", user: User) if user.id = 0;"""
    )

    session = AuthorizedSession(
        oso=oso,
        user="user",
        checked_permissions={Post: "read", User: "read"},
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


def test_scoped_session_with_no_checked_permissions(engine, oso, fixture_data):
    # the policy denies all requests
    oso.load_str("allow(_, _, _) if false;")
    # but passing None skips authorization
    session = scoped_session(lambda: oso, lambda: "user", lambda: None)
    session.configure(bind=engine)
    posts = session.query(Post)
    # check that any posts are allowed
    assert posts.count()


def test_scoped_session_relationship(engine, oso, fixture_data):
    oso.load_str(
        """allow("user", "read", post: Post) if post.id = 1;
           # Post with creator id = 1
           allow("user", "read", post: Post) if post.id = 7;
           allow("user", "read", user: User) if user.id = 0;
           allow("other", "read", post: Post) if post.id = 3;
           allow("other", "write", post: Post) if post.id = 4;"""
    )

    data = {"user": "user", "checked_permissions": {Post: "read", User: "read"}}
    session = scoped_session(
        lambda: oso, lambda: data["user"], lambda: data["checked_permissions"]
    )
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

    data["checked_permissions"] = {Post: "write", User: "write"}
    assert len(session.identity_map.values()) == 0
    posts = session.query(Post)
    assert posts.count() == 1
    posts = posts.all()
    assert posts[0].id == 4
    assert len(session.identity_map.values()) == 1

    # Change back to original.
    data = {"user": "user", "checked_permissions": {Post: "read", User: "read"}}
    assert len(session.identity_map.values()) == 3


@pytest.mark.xfail(reason="Implemented incorrectly initially. Fix")
def test_authorized_sessionmaker_user_change(engine, oso, fixture_data):
    """Ensure that query fails if the user changes."""
    oso.load_str('allow("user", "read", post: Post) if post.id = 1;')
    user = ["user"]

    Session = authorized_sessionmaker(
        get_oso=lambda: oso,
        get_user=lambda: user[0],
        get_checked_permissions=lambda: {Post: "read"},
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
        get_checked_permissions=lambda: {Post: "read"},
        bind=engine,
    )
    posts = Session().query(Post)

    if USING_SQLAlchemy_v1_3:
        where_clause = " \nWHERE posts.contents IS NULL"
    else:
        # NOTE(gj): In contrast to the `Query.before_compile` event we listen
        # for in 1.3, the `Session.do_orm_execute` event we listen for in
        # SQLAlchemy 1.4 (unsurprisingly) happens at ORM execution time. Because
        # of this, the WHERE clauses we add as authorization constraints are
        # not applied when we (compile and) print out the query in the below
        # assertion but *are* applied when we actually interact with the ORM
        # when executing the `count()` method.
        where_clause = ""

    assert str(posts) == (
        "SELECT posts.id AS posts_id, posts.contents AS posts_contents, posts.title AS posts_title, "
        + "posts.access_level AS posts_access_level, posts.created_by_id AS posts_created_by_id, "
        + f"posts.needs_moderation AS posts_needs_moderation \nFROM posts{where_clause}"
    )
    assert posts.count() == 0


def test_regular_session(engine, oso, fixture_data):
    """Test that a regular session doesn't apply authorization."""
    from sqlalchemy.orm import Session

    session = Session(bind=engine)
    posts = session.query(Post)

    # No posts would be returned if authorization was applied.
    assert posts.count() == 9


def test_unconditional_policy_has_no_filter(engine, oso, fixture_data):
    oso.load_str('allow("user", "read", post: Post) if post.id = 1; allow(_, _, _);')
    session = AuthorizedSession(
        oso, user="user", checked_permissions={Post: "read"}, bind=engine
    )

    query = session.query(Post)

    if USING_SQLAlchemy_v1_3:
        where_clause = " \nWHERE 1 = 1"
    else:
        # NOTE(gj): In contrast to the `Query.before_compile` event we listen
        # for in 1.3, the `Session.do_orm_execute` event we listen for in
        # SQLAlchemy 1.4 (unsurprisingly) happens at ORM execution time. Because
        # of this, the WHERE clauses we add as authorization constraints are
        # not applied when we (compile and) print out the query in the below
        # assertion but *are* applied when we actually interact with the ORM
        # when executing the `count()` method.
        where_clause = ""

    assert str(query) == (
        "SELECT posts.id AS posts_id, posts.contents AS posts_contents, posts.title AS posts_title, "
        + "posts.access_level AS posts_access_level, posts.created_by_id AS posts_created_by_id, "
        + f"posts.needs_moderation AS posts_needs_moderation \nFROM posts{where_clause}"
    )
    assert query.count() == 9


def test_bakery_caching_for_AuthorizedSession(engine, oso, fixture_data):
    """Test that baked relationship queries don't lead to authorization bypasses
    for AuthorizedSession."""
    from sqlalchemy.orm import Session

    basic_session = Session(bind=engine)
    all_posts = basic_session.query(Post)
    assert all_posts.count() == 9
    first_post = all_posts[0]
    # Add related model query to the bakery cache.
    assert first_post.created_by.id == 0

    oso.load_str('allow("user", "read", post: Post) if post.id = 0;')

    # Baked queries disabled for sqlalchemy_oso.session.AuthorizedSession.
    authorized_session = AuthorizedSession(
        oso, user="user", checked_permissions={Post: "read"}, bind=engine
    )

    assert authorized_session.query(User).count() == 0

    authorized_posts = authorized_session.query(Post)
    assert authorized_posts.count() == 1
    first_authorized_post = authorized_posts[0]
    assert first_post.id == first_authorized_post.id

    # Should not be able to view the post's creator because there's no rule
    # permitting access to "read" users.
    assert first_authorized_post.created_by is None


def test_bakery_caching_for_authorized_sessionmaker(engine, oso, fixture_data):
    """Test that baked relationship queries don't lead to authorization bypasses
    for authorized_sessionmaker."""
    from sqlalchemy.orm import Session

    basic_session = Session(bind=engine)
    all_posts = basic_session.query(Post)
    assert all_posts.count() == 9
    first_post = all_posts[0]
    # Add related model query to the bakery cache.
    assert first_post.created_by.id == 0

    oso.load_str('allow("user", "read", post: Post) if post.id = 0;')

    # Baked queries disabled for sqlalchemy_oso.session.authorized_sessionmaker.
    authorized_session = authorized_sessionmaker(
        get_oso=lambda: oso,
        get_user=lambda: "user",
        get_checked_permissions=lambda: {Post: "read"},
        bind=engine,
    )()

    assert authorized_session.query(User).count() == 0

    authorized_posts = authorized_session.query(Post)
    assert authorized_posts.count() == 1
    first_authorized_post = authorized_posts[0]
    assert first_post.id == first_authorized_post.id

    # Should not be able to view the post's creator because there's no rule
    # permitting access to "read" users.
    assert first_authorized_post.created_by is None


def test_bakery_caching_for_scoped_session(engine, oso, fixture_data):
    """Test that baked relationship queries don't lead to authorization bypasses
    for scoped_session."""
    from sqlalchemy.orm import Session

    basic_session = Session(bind=engine)
    all_posts = basic_session.query(Post)
    assert all_posts.count() == 9
    first_post = all_posts[0]
    # Add related model query to the bakery cache.
    assert first_post.created_by.id == 0

    oso.load_str('allow("user", "read", post: Post) if post.id = 0;')

    # Baked queries disabled for sqlalchemy_oso.session.scoped_session.
    authorized_session = scoped_session(
        lambda: oso, lambda: "user", lambda: {Post: "read"}
    )
    authorized_session.configure(bind=engine)

    assert authorized_session.query(User).count() == 0

    authorized_posts = authorized_session.query(Post)
    assert authorized_posts.count() == 1
    first_authorized_post = authorized_posts[0]
    assert first_post.id == first_authorized_post.id

    # Should not be able to view the post's creator because there's no rule
    # permitting access to "read" users.
    assert first_authorized_post.created_by is None


def test_checked_permissions(engine, oso, fixture_data):
    """Test a query involving multiple models."""
    oso.load_str(
        """allow("user", "read", post: Post) if post.id = 1;
           allow("user", "view", user: User) if user.id = 0;
           allow("user", "view", user: User) if user.id = 1;
           allow("all_posts", "read", _: Post);"""
    )

    # Not applying any authorization to this session.
    session1 = AuthorizedSession(oso, "user", checked_permissions=None, bind=engine)
    posts1 = session1.query(Post)
    assert posts1.count() == 9
    assert posts1[0].created_by_id == 0
    assert posts1[0].created_by.username == "foo"
    users1 = session1.query(User)
    assert users1.count() == 3

    # Deny access to every model for this session by omission.
    session2 = AuthorizedSession(oso, "user", checked_permissions={}, bind=engine)
    posts2 = session2.query(Post)
    assert posts2.count() == 0
    users2 = session2.query(User)
    assert users2.count() == 0

    # Deny access to specific models for this session by inclusion.
    session3 = AuthorizedSession(
        oso, "user", checked_permissions={Post: None, User: None}, bind=engine
    )
    posts3 = session3.query(Post)
    assert posts3.count() == 0
    users3 = session3.query(User)
    assert users3.count() == 0

    # Allow access to one model but not the other for this session.
    session4 = AuthorizedSession(
        oso, "user", checked_permissions={Post: "read"}, bind=engine
    )
    posts4 = session4.query(Post)
    assert posts4.count() == 1
    assert posts4[0].id == 1
    assert posts4[0].created_by_id == 0
    assert posts4[0].created_by is None
    users4 = session4.query(User)
    assert users4.count() == 0

    # Allow access to multiple models with multiple actions for this session.
    session4 = AuthorizedSession(
        oso, "user", checked_permissions={Post: "read", User: "view"}, bind=engine
    )
    posts4 = session4.query(Post)
    assert posts4.count() == 1
    assert posts4[0].id == 1
    assert posts4[0].created_by_id == 0
    assert posts4[0].created_by.username == "foo"
    users4 = session4.query(User)
    assert users4.count() == 2


def test_register_models_declarative_base():
    """Test that `register_models()` registers models."""
    from oso import Oso
    from polar.exceptions import DuplicateClassAliasError

    from sqlalchemy_oso.auth import register_models

    from .models import Category, ModelBase, Tag

    oso = Oso()
    register_models(oso, ModelBase)

    for m in [Category, Post, Tag, User]:
        with pytest.raises(DuplicateClassAliasError):
            oso.register_class(m)


@pytest.mark.skipif(
    USING_SQLAlchemy_v1_3, reason="testing SQLAlchemy 1.4 functionality"
)
def test_register_models_registry():
    """Test that `register_models()` works with a SQLAlchemy 1.4-style
    registry."""
    # TODO(gj): remove type ignore once we upgrade to 1.4-aware MyPy types.
    from oso import Oso
    from polar.exceptions import DuplicateClassAliasError
    from sqlalchemy import Column, Integer, Table
    from sqlalchemy.orm import registry  # type: ignore

    from sqlalchemy_oso.auth import register_models

    mapper_registry = registry()

    user_table = Table(
        "user",
        mapper_registry.metadata,
        Column("id", Integer, primary_key=True),
    )

    class User:
        pass

    mapper_registry.map_imperatively(User, user_table)

    post_table = Table(
        "post",
        mapper_registry.metadata,
        Column("id", Integer, primary_key=True),
    )

    class Post:
        pass

    mapper_registry.map_imperatively(Post, post_table)

    oso = Oso()
    register_models(oso, mapper_registry)

    for m in [Post, User]:
        with pytest.raises(DuplicateClassAliasError):
            oso.register_class(m)
