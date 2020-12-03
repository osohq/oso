"""Standardized tests for adapters based on the Post model.

Tests come from the relationship document & operations laid out there
https://www.notion.so/osohq/Relationships-621b884edbc6423f93d29e6066e58d16.
"""
import pytest

from sqlalchemy_oso.auth import authorize_model

from .models import Post, Tag, User
from .conftest import print_query


def test_authorize_model_basic(session, oso, fixture_data):
    """Test that a simple policy with checks on non-relationship attributes is correct."""
    oso.load_str('allow("user", "read", post: Post) if post.access_level = "public";')
    oso.load_str('allow("user", "write", post: Post) if post.access_level = "private";')
    oso.load_str('allow("admin", "read", post: Post);')
    oso.load_str(
        'allow("moderator", "read", post: Post) if '
        '(post.access_level = "private" or post.access_level = "public") and '
        "post.needs_moderation = true;"
    )

    posts = authorize_model(oso, "user", "read", session, Post)

    assert posts.count() == 5
    assert posts.all()[0].contents == "foo public post"
    assert posts.all()[0].id == 0

    posts = authorize_model(oso, "user", "write", session, Post)

    assert posts.count() == 4
    assert posts.all()[0].contents == "foo private post"
    assert posts.all()[1].contents == "foo private post 2"

    posts = authorize_model(oso, "admin", "read", session, Post)
    assert posts.count() == 9

    posts = authorize_model(oso, "moderator", "read", session, Post)
    print_query(posts)
    assert posts.all()[0].contents == "private for moderation"
    assert posts.all()[1].contents == "public for moderation"

    posts = authorize_model(oso, "guest", "read", session, Post)
    assert posts.count() == 0


def test_authorize_scalar_attribute_eq(session, oso, fixture_data):
    """Test authorization rules on a relationship with one object equaling another."""
    # Object equals another object
    oso.load_str(
        'allow(actor: User, "read", post: Post) if post.created_by = actor and '
        'post.access_level = "private";'
    )
    oso.load_str(
        'allow(actor: User, "read", post: Post) if ' 'post.access_level = "public";'
    )
    oso.load_str(
        'allow(actor: User{is_moderator: true}, "read", post: Post) if '
        'post.access_level = "public";'
    )

    foo = session.query(User).filter(User.username == "foo").first()

    posts = authorize_model(oso, foo, "read", session, Post)
    print_query(posts)

    def allowed(post):
        return (
            post.access_level == "public"
            or post.access_level == "private"
            and post.created_by == foo
        )

    assert posts.count() == 8
    assert all(allowed(post) for post in posts)


def test_authorize_scalar_attribute_condition(session, oso, fixture_data):
    """Scalar attribute condition checks."""
    # Object equals another object

    oso.load_str(
        'allow(actor: User, "read", post: Post) if post.created_by.is_banned = false and '
        'post.created_by.username = actor.username and post.access_level = "private";'
    )

    oso.load_str(
        'allow(actor: User, "read", post: Post) if post.created_by.is_banned = false and '
        'post.access_level = "public";'
    )

    # moderator can see posts made by banned users.
    oso.load_str(
        """allow(actor: User, "read", post: Post) if
                actor.is_moderator = true
                and post.created_by.is_banned = true;"""
    )

    foo = session.query(User).filter(User.username == "foo").first()

    posts = authorize_model(oso, foo, "read", session, Post)

    def allowed(post, user):
        return (
            (post.access_level == "public" and not post.created_by.is_banned)
            or post.access_level == "private"
            and post.created_by == user
        )

    assert posts.count() == 7
    assert all(allowed(post, foo) for post in posts)

    admin = session.query(User).filter(User.username == "admin_user").first()
    posts = authorize_model(oso, admin, "read", session, Post)

    def allowed_admin(post):
        return post.created_by.is_banned

    assert posts.count() == 6
    for post in posts:
        assert allowed(post, admin) or allowed_admin(post)


@pytest.fixture
def tag_test_fixture(session):
    """Test data for tests with tags."""
    user = User(username="user")
    other_user = User(username="other_user")
    moderator = User(username="moderator", is_moderator=True)

    eng = Tag(name="eng")
    foo = Tag(name="foo")
    random = Tag(name="random", is_public=True)

    user_public_post = Post(
        contents="public post", created_by=user, access_level="public"
    )
    user_private_post = Post(
        contents="private user post", created_by=user, access_level="private"
    )

    other_user_public_post = Post(
        contents="other user public", created_by=other_user, access_level="public"
    )
    other_user_private_post = Post(
        contents="other user private", created_by=other_user, access_level="private"
    )
    other_user_random_post = Post(
        contents="other user random",
        created_by=other_user,
        access_level="private",
        tags=[random],
    )
    other_user_foo_post = Post(
        contents="other user foo",
        created_by=other_user,
        access_level="private",
        tags=[foo],
    )

    # HACK!
    objects = {}
    for (name, local) in locals().items():
        if name != "session" and name != "objects":
            session.add(local)

        objects[name] = local

    session.commit()

    return objects


def test_in_multiple_attribute_relationship(session, oso, tag_test_fixture):
    oso.load_str(
        """
        allow(user, "read", post: Post) if post.access_level = "public";
        allow(user, "read", post: Post) if post.access_level = "private" and post.created_by = user;
        allow(user, "read", post: Post) if
            tag in post.tags and
            0 < post.id and
            (tag.is_public = true or tag.name = "foo");
    """
    )

    posts = authorize_model(oso, tag_test_fixture["user"], "read", session, Post)

    assert tag_test_fixture["user_public_post"] in posts
    assert tag_test_fixture["user_private_post"] in posts
    assert tag_test_fixture["other_user_public_post"] in posts
    assert not tag_test_fixture["other_user_private_post"] in posts
    assert tag_test_fixture["other_user_random_post"] in posts
    assert tag_test_fixture["other_user_foo_post"] in posts
    assert posts.count() == 5


@pytest.fixture
def tag_nested_test_fixture(session):
    user = User(username="user")
    other_user = User(username="other_user")
    moderator = User(username="moderator", is_moderator=True)

    eng = Tag(name="eng", created_by=user)
    user_posts = Tag(name="user_posts", created_by=user)
    random = Tag(name="random", is_public=True, created_by=other_user)

    user_eng_post = Post(
        contents="user eng post", access_level="public", created_by=user, tags=[eng]
    )
    user_user_post = Post(
        contents="user eng post",
        access_level="public",
        created_by=user,
        tags=[user_posts],
    )

    random_post = Post(
        contents="other random post",
        access_level="public",
        created_by=other_user,
        tags=[random],
    )

    not_tagged_post = Post(
        contents="not tagged post", access_level="public", created_by=user, tags=[]
    )

    all_tagged_post = Post(
        contents="not tagged post",
        access_level="public",
        created_by=user,
        tags=[eng, user_posts, random],
    )

    # HACK!
    objects = {}
    for (name, local) in locals().items():
        if name != "session" and name != "objects":
            session.add(local)

        objects[name] = local

    session.commit()

    return objects


# TODO (dhatch): This doesn't actually exercise nested attribute code, because
# the nested piece is in a sub expression.
def test_nested_relationship_many_single(session, oso, tag_nested_test_fixture):
    """Test that nested relationships work.

    post - (many) -> tags - (single) -> User

    A user can read a post with a tag if the tag's creator is the user.
    """
    oso.load_str(
        """
        allow(user, "read", post: Post) if tag in post.tags and tag.created_by = user;
    """
    )

    posts = authorize_model(oso, tag_nested_test_fixture["user"], "read", session, Post)
    assert tag_nested_test_fixture["user_eng_post"] in posts
    assert tag_nested_test_fixture["user_user_post"] in posts
    assert not tag_nested_test_fixture["random_post"] in posts
    assert not tag_nested_test_fixture["not_tagged_post"] in posts
    assert tag_nested_test_fixture["all_tagged_post"] in posts
    assert posts.count() == 3

    posts = authorize_model(
        oso, tag_nested_test_fixture["other_user"], "read", session, Post
    )
    assert not tag_nested_test_fixture["user_eng_post"] in posts
    assert not tag_nested_test_fixture["user_user_post"] in posts
    assert tag_nested_test_fixture["random_post"] in posts
    assert not tag_nested_test_fixture["not_tagged_post"] in posts
    assert tag_nested_test_fixture["all_tagged_post"] in posts
    assert posts.count() == 2


@pytest.fixture
def tag_nested_many_many_test_fixture(session):
    eng = Tag(name="eng")
    user_posts = Tag(name="user_posts")
    random = Tag(name="random", is_public=True)

    user = User(username="user", tags=[eng, user_posts])
    other_user = User(username="other_user", tags=[random])
    moderator = User(username="moderator", tags=[random, user_posts, eng])

    user_eng_post = Post(
        contents="user eng post", access_level="public", created_by=user, tags=[eng]
    )
    user_user_post = Post(
        contents="user eng post",
        access_level="public",
        created_by=user,
        tags=[user_posts],
    )

    random_post = Post(
        contents="other random post",
        access_level="public",
        created_by=other_user,
        tags=[random],
    )

    not_tagged_post = Post(
        contents="not tagged post", access_level="public", created_by=user, tags=[]
    )

    all_tagged_post = Post(
        contents="not tagged post",
        access_level="public",
        created_by=user,
        tags=[eng, user_posts, random],
    )

    # HACK!
    objects = {}
    for (name, local) in locals().items():
        if name != "session" and name != "objects":
            session.add(local)

        objects[name] = local

    session.commit()

    return objects


def test_nested_relationship_many_many(session, oso, tag_nested_many_many_test_fixture):
    """Test that nested relationships work.

    post - (many) -> tags - (many) -> User

    A user can read a post with a tag if the tag's creator is the user.
    """
    # TODO This direction doesn't work, because tag in user.tags is a concrete object.
    # allow(user, "read", post: Post) if tag in post.tags and tag in user.tags;
    oso.load_str(
        """
    allow(user, "read", post: Post) if tag in post.tags and user in tag.users;
    """
    )

    posts = authorize_model(
        oso, tag_nested_many_many_test_fixture["user"], "read", session, Post
    )
    # TODO (dhatch): Check that this SQL query is correct, seems right from results.
    print_query(posts)
    assert tag_nested_many_many_test_fixture["user_eng_post"] in posts
    assert tag_nested_many_many_test_fixture["user_user_post"] in posts
    assert not tag_nested_many_many_test_fixture["random_post"] in posts
    assert not tag_nested_many_many_test_fixture["not_tagged_post"] in posts
    assert tag_nested_many_many_test_fixture["all_tagged_post"] in posts

    posts = authorize_model(
        oso, tag_nested_many_many_test_fixture["other_user"], "read", session, Post
    )
    assert not tag_nested_many_many_test_fixture["user_eng_post"] in posts
    assert not tag_nested_many_many_test_fixture["user_user_post"] in posts
    assert tag_nested_many_many_test_fixture["random_post"] in posts
    assert not tag_nested_many_many_test_fixture["not_tagged_post"] in posts
    assert tag_nested_many_many_test_fixture["all_tagged_post"] in posts


def test_partial_in_collection(session, oso, tag_nested_many_many_test_fixture):
    oso.load_str(
        """
        allow(user, "read", post: Post) if post in user.posts;
    """
    )

    user = tag_nested_many_many_test_fixture["user"]
    posts = authorize_model(oso, user, "read", session, Post)
    print_query(posts)
    posts = posts.all()

    assert tag_nested_many_many_test_fixture["user_eng_post"] in posts
    assert tag_nested_many_many_test_fixture["user_user_post"] in posts
    assert tag_nested_many_many_test_fixture["random_post"] not in posts
    assert tag_nested_many_many_test_fixture["not_tagged_post"] in posts
    assert tag_nested_many_many_test_fixture["all_tagged_post"] in posts
    assert len(posts) == 4

    user = tag_nested_many_many_test_fixture["other_user"]
    posts = authorize_model(oso, user, "read", session, Post).all()
    assert tag_nested_many_many_test_fixture["user_eng_post"] not in posts
    assert tag_nested_many_many_test_fixture["user_user_post"] not in posts
    assert tag_nested_many_many_test_fixture["random_post"] in posts
    assert tag_nested_many_many_test_fixture["not_tagged_post"] not in posts
    assert tag_nested_many_many_test_fixture["all_tagged_post"] not in posts
    assert len(posts) == 1


def test_empty_constraints_in(session, oso, tag_nested_many_many_test_fixture):
    oso.load_str("""allow(_, "read", post: Post) if _tag in post.tags;""")
    user = tag_nested_many_many_test_fixture["user"]
    posts = authorize_model(oso, user, "read", session, Post)
    assert str(posts) == (
        "SELECT posts.id AS posts_id, posts.contents AS posts_contents, posts.access_level AS posts_access_level,"
        + " posts.created_by_id AS posts_created_by_id, posts.needs_moderation AS posts_needs_moderation"
        + " \nFROM posts"
        + " \nWHERE (EXISTS (SELECT 1"
        + " \nFROM post_tags, tags"
        + " \nWHERE posts.id = post_tags.post_id AND tags.name = post_tags.tag_id))"
    )
    posts = posts.all()
    assert len(posts) == 4
    assert tag_nested_many_many_test_fixture["not_tagged_post"] not in posts


def test_in_with_constraints_but_no_matching_objects(
    session, oso, tag_nested_many_many_test_fixture
):
    oso.load_str(
        """
        allow(_, "read", post: Post) if
            tag in post.tags and
            tag.name = "bloop";
    """
    )
    user = tag_nested_many_many_test_fixture["user"]
    posts = authorize_model(oso, user, "read", session, Post)
    assert str(posts) == (
        "SELECT posts.id AS posts_id, posts.contents AS posts_contents, posts.access_level AS posts_access_level,"
        + " posts.created_by_id AS posts_created_by_id, posts.needs_moderation AS posts_needs_moderation"
        + " \nFROM posts"
        + " \nWHERE (EXISTS (SELECT 1"
        + " \nFROM post_tags, tags"
        + " \nWHERE posts.id = post_tags.post_id AND tags.name = post_tags.tag_id AND tags.name = ?))"
    )
    posts = posts.all()
    assert len(posts) == 0


# TODO combine with test in test_django_oso.
def test_partial_subfield_isa(session, oso, tag_nested_many_many_test_fixture):
    oso.load_str(
        """
            allow(_, _, post: Post) if check_user(post.created_by);
            # User is not a tag.
            check_user(user: Tag) if user.username = "other_user";
            check_user(user: User) if user.username = "user";
        """
    )

    user = tag_nested_many_many_test_fixture["user"]
    posts = authorize_model(oso, user, "read", session, Post)
    # Should only get posts created by user.
    posts = posts.all()
    for post in posts:
        assert post.created_by.username == "user"

    assert len(posts) == 4


# TODO test_nested_relationship_single_many
# TODO test_nested_relationship_single_single
# TODO test_nested_relationship_single_single_single ... etc

# TODO test non Eq conditions
# TODO test f(x) if not x.boolean_attr;
# TODO test this = ? with multiple pks
