"""Standardized tests for adapters based on the Post model.

Tests come from the relationship document & operations laid out there
https://www.notion.so/osohq/Relationships-621b884edbc6423f93d29e6066e58d16.
"""
import pytest
from django.db import models
from django.core.exceptions import PermissionDenied

from django_oso.models import AuthorizedModel, authorize_model
from django_oso.oso import Oso, reset_oso
from test_app2.models import Post, Tag, User


@pytest.fixture(autouse=True)
def reset():
    reset_oso()


@pytest.fixture
def post_fixtures():
    foo = User(username="foo")
    foo.save()
    admin = User(username="admin", is_moderator=True)
    admin.save()
    banned = User(username="banned", is_banned=True)
    banned.save()

    Post(contents="foo public post", access_level="public", created_by=foo).save()
    Post(
        contents="foo public post 2",
        access_level="public",
        created_by=foo,
    ).save()
    Post(
        contents="foo private post",
        created_by=foo,
    ).save()
    Post(
        contents="foo private post 2",
        created_by=foo,
    ).save()
    Post(
        contents="private for moderation",
        needs_moderation=True,
        created_by=foo,
    ).save()
    Post(
        contents="public for moderation",
        access_level="public",
        needs_moderation=True,
        created_by=foo,
    ).save()
    Post(
        contents="public admin post",
        access_level="public",
        needs_moderation=True,
        created_by=admin,
    ).save()
    Post(
        contents="private admin post",
        needs_moderation=True,
        created_by=admin,
    ).save()
    Post(contents="banned post", access_level="public", created_by=banned).save()


@pytest.mark.django_db
def test_authorize_model_basic(post_fixtures):
    """Test that a simple policy with checks on non-relationship attributes is correct."""
    Oso.load_str(
        """
        allow("user", "read", post: test_app2::Post) if post.access_level = "public";
        allow("user", "write", post: test_app2::Post) if post.access_level = "private";
        allow("admin", "read", _post: test_app2::Post);
        allow("moderator", "read", post: test_app2::Post) if
            (post.access_level = "private" or post.access_level = "public") and
            post.needs_moderation = true;
        """
    )

    authorize_filter = authorize_model(None, Post, actor="user", action="read")
    assert str(authorize_filter) == "(AND: ('access_level', 'public'))"
    posts = Post.objects.filter(authorize_filter)
    assert posts.count() == 5
    assert posts.all()[0].contents == "foo public post"

    authorize_filter = authorize_model(None, Post, actor="user", action="write")
    assert str(authorize_filter) == "(AND: ('access_level', 'private'))"
    posts = Post.objects.filter(authorize_filter)
    assert posts.count() == 4
    assert posts.all()[0].contents == "foo private post"
    assert posts.all()[1].contents == "foo private post 2"

    authorize_filter = authorize_model(None, Post, actor="admin", action="read")
    assert str(authorize_filter) == "(AND: )"
    posts = Post.objects.filter(authorize_filter)
    assert posts.count() == 9

    authorize_filter = authorize_model(None, Post, actor="moderator", action="read")
    assert (
        str(authorize_filter)
        == "(OR: (AND: ('access_level', 'private'), ('needs_moderation', True)), (AND: ('access_level', 'public'), ('needs_moderation', True)))"
    )
    posts = Post.objects.filter(authorize_filter)
    assert posts.count() == 4
    assert posts.all()[0].contents == "private for moderation"
    assert posts.all()[1].contents == "public for moderation"

    # Not authorized
    with pytest.raises(PermissionDenied):
        authorize_model(None, Post, actor="guest", action="read")


@pytest.mark.django_db
def test_authorize_scalar_attribute_eq(post_fixtures):
    """Test authorization rules on a relationship with one object equaling another."""
    # Object equals another object
    Oso.load_str(
        """
        allow(actor: test_app2::User, "read", post: test_app2::Post) if
            post.created_by = actor and
            post.access_level = "private";
        allow(_: test_app2::User, "read", post: test_app2::Post) if
            post.access_level = "public";
        allow(_: test_app2::User{is_moderator: true}, "read", post: test_app2::Post) if
            post.access_level = "public";
        """
    )

    foo = User.objects.get(username="foo")
    authorize_filter = authorize_model(None, Post, actor=foo, action="read")
    posts = Post.objects.filter(authorize_filter)

    def allowed(post):
        return (
            post.access_level == "public"
            or post.access_level == "private"
            and post.created_by == foo
        )

    assert posts.count() == 8
    assert all(allowed(post) for post in posts)


@pytest.mark.django_db
def test_authorize_scalar_attribute_condition(post_fixtures):
    """Scalar attribute condition checks."""
    Oso.load_str(
        """
        # Object equals another object
        allow(actor: test_app2::User, "read", post: test_app2::Post) if
            post.created_by.is_banned = false and
            post.created_by = actor and
            post.access_level = "private";

        allow(_actor: test_app2::User, "read", post: test_app2::Post) if
            post.created_by.is_banned = false and
            post.access_level = "public";

        # moderator can see posts made by banned users.
        allow(actor: test_app2::User, "read", post: test_app2::Post) if
            actor.is_moderator = true and
            post.created_by.is_banned = true;
        """
    )

    foo = User.objects.get(username="foo")
    authorize_filter = authorize_model(None, Post, actor=foo, action="read")
    posts = Post.objects.filter(authorize_filter)

    def allowed(post, user):
        return (
            post.access_level == "public" and post.created_by.is_banned == False
        ) or (post.access_level == "private" and post.created_by == user)

    assert posts.count() == 7
    assert all(allowed(post, foo) for post in posts)

    admin = User.objects.get(username="admin")
    authorize_filter = authorize_model(None, Post, actor=admin, action="read")
    posts = Post.objects.filter(authorize_filter)

    def allowed_admin(post):
        return post.created_by.is_banned

    assert posts.count() == 6
    for post in posts:
        assert allowed(post, admin) or allowed_admin(post)


@pytest.fixture
def tag_fixtures():
    """Test data for tests with tags."""
    user = User(username="user")
    user.save()
    other_user = User(username="other_user")
    other_user.save()
    moderator = User(username="moderator", is_moderator=True)
    moderator.save()

    eng = Tag(name="eng")
    eng.save()
    foo = Tag(name="foo")
    foo.save()
    random = Tag(name="random", is_public=True)
    random.save()

    user_public_post = Post(
        contents="public post", created_by=user, access_level="public"
    )
    user_private_post = Post(contents="private user post", created_by=user)
    other_user_public_post = Post(
        contents="other user public", created_by=other_user, access_level="public"
    )
    other_user_private_post = Post(contents="other user private", created_by=other_user)
    other_user_random_post = Post(contents="other user random", created_by=other_user)
    other_user_foo_post = Post(contents="other user foo", created_by=other_user)

    posts = {
        "user_public_post": user_public_post,
        "user_private_post": user_private_post,
        "other_user_public_post": other_user_public_post,
        "other_user_private_post": other_user_private_post,
        "other_user_random_post": other_user_random_post,
        "other_user_foo_post": other_user_foo_post,
    }
    for post in posts.values():
        post.save()

    other_user_random_post.tags.set([random])
    other_user_foo_post.tags.set([foo])

    return posts


@pytest.mark.django_db
def test_in_multiple_attribute_relationship(tag_fixtures):
    Oso.load_str(
        """
        allow(_: test_app2::User, "read", post: test_app2::Post) if
            post.access_level = "public";
        allow(user: test_app2::User, "read", post: test_app2::Post) if
            post.access_level = "private" and
            post.created_by = user;
        allow(_: test_app2::User, "read", post: test_app2::Post) if
            tag in post.tags and
            tag.id > 0 and
            (tag.is_public = true or tag.name = "foo");
        """
    )

    user = User.objects.get(username="user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter)

    assert tag_fixtures["user_public_post"] in posts
    assert tag_fixtures["user_private_post"] in posts
    assert tag_fixtures["other_user_public_post"] in posts
    assert tag_fixtures["other_user_private_post"] not in posts
    assert tag_fixtures["other_user_random_post"] in posts
    assert tag_fixtures["other_user_foo_post"] in posts


@pytest.fixture
def tag_nested_fixtures():
    user = User(username="user")
    user.save()
    other_user = User(username="other_user")
    other_user.save()
    moderator = User(username="moderator", is_moderator=True)
    moderator.save()

    eng = Tag(name="eng", created_by=user)
    eng.save()
    user_posts = Tag(name="user_posts", created_by=user)
    user_posts.save()
    random = Tag(name="random", is_public=True, created_by=other_user)
    random.save()

    user_eng_post = Post(
        contents="user eng post", access_level="public", created_by=user
    )
    user_user_post = Post(
        contents="user eng post",
        access_level="public",
        created_by=user,
    )
    random_post = Post(
        contents="other random post",
        access_level="public",
        created_by=other_user,
    )
    not_tagged_post = Post(
        contents="not tagged post", access_level="public", created_by=user
    )
    all_tagged_post = Post(
        contents="not tagged post",
        access_level="public",
        created_by=user,
    )

    posts = {
        "user_eng_post": user_eng_post,
        "user_user_post": user_user_post,
        "random_post": random_post,
        "not_tagged_post": not_tagged_post,
        "all_tagged_post": all_tagged_post,
    }
    for post in posts.values():
        post.save()

    user_eng_post.tags.set([eng])
    user_user_post.tags.set([user_posts])
    random_post.tags.set([random])
    all_tagged_post.tags.set([eng, user_posts, random])

    return posts


# TODO (dhatch): This doesn't actually exercise nested attribute code, because
# the nested piece is in a sub expression.
@pytest.mark.django_db
def test_nested_relationship_many_single(tag_nested_fixtures):
    """Test that nested relationships work.
    post - (many) -> tags - (single) -> User
    A user can read a post with a tag if the tag's creator is the user.
    """
    Oso.load_str(
        """
        allow(user: test_app2::User, "read", post: test_app2::Post) if
            tag in post.tags and
            tag.created_by = user;
        """
    )

    user = User.objects.get(username="user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter)
    assert tag_nested_fixtures["user_eng_post"] in posts
    assert tag_nested_fixtures["user_user_post"] in posts
    assert tag_nested_fixtures["random_post"] not in posts
    assert tag_nested_fixtures["not_tagged_post"] not in posts
    assert tag_nested_fixtures["all_tagged_post"] in posts

    other_user = User.objects.get(username="other_user")
    authorize_filter = authorize_model(None, Post, actor=other_user, action="read")
    posts = Post.objects.filter(authorize_filter)
    assert tag_nested_fixtures["user_eng_post"] not in posts
    assert tag_nested_fixtures["user_user_post"] not in posts
    assert tag_nested_fixtures["random_post"] in posts
    assert tag_nested_fixtures["not_tagged_post"] not in posts
    assert tag_nested_fixtures["all_tagged_post"] in posts


@pytest.fixture
def tag_nested_many_many_fixtures():

    user = User(username="user")
    user.save()
    other_user = User(username="other_user")
    other_user.save()
    moderator = User(username="moderator")
    moderator.save()

    eng = Tag(name="eng")
    eng.save()
    eng.users.set([user, moderator])
    user_posts = Tag(name="user_posts")
    user_posts.save()
    user_posts.users.set([user, moderator])
    random = Tag(name="random", is_public=True)
    random.save()
    random.users.set([other_user, moderator])

    user_eng_post = Post(
        contents="user eng post", access_level="public", created_by=user
    )
    user_user_post = Post(
        contents="user user post",
        access_level="public",
        created_by=user,
    )
    random_post = Post(
        contents="other random post",
        access_level="public",
        created_by=other_user,
    )
    not_tagged_post = Post(
        contents="not tagged post", access_level="public", created_by=user
    )
    all_tagged_post = Post(
        contents="not tagged post",
        access_level="public",
        created_by=user,
    )

    posts = {
        "user_eng_post": user_eng_post,
        "user_user_post": user_user_post,
        "random_post": random_post,
        "not_tagged_post": not_tagged_post,
        "all_tagged_post": all_tagged_post,
    }
    for post in posts.values():
        post.save()

    user_eng_post.tags.set([eng])
    user_user_post.tags.set([user_posts])
    random_post.tags.set([random])
    all_tagged_post.tags.set([eng, user_posts, random])

    return posts


@pytest.mark.django_db
def test_nested_relationship_many_many(tag_nested_many_many_fixtures):
    """Test that nested relationships work.
    post - (many) -> tags - (many) -> User
    A user can read a post with a tag if the tag's creator is the user.
    """
    # TODO This direction doesn't work, because tag in user.tags is a concrete object.
    # allow(user, "read", post: Post) if tag in post.tags and tag in user.tags;
    Oso.load_str(
        """
            allow(user: test_app2::User, "read", post: test_app2::Post) if
                tag in post.tags and
                user in tag.users;
        """
    )

    user = User.objects.get(username="user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter)
    assert tag_nested_many_many_fixtures["user_eng_post"] in posts
    assert tag_nested_many_many_fixtures["user_user_post"] in posts
    assert tag_nested_many_many_fixtures["random_post"] not in posts
    assert tag_nested_many_many_fixtures["not_tagged_post"] not in posts
    assert tag_nested_many_many_fixtures["all_tagged_post"] in posts

    user = User.objects.get(username="other_user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter)
    assert tag_nested_many_many_fixtures["user_eng_post"] not in posts
    assert tag_nested_many_many_fixtures["user_user_post"] not in posts
    assert tag_nested_many_many_fixtures["random_post"] in posts
    assert tag_nested_many_many_fixtures["not_tagged_post"] not in posts
    assert tag_nested_many_many_fixtures["all_tagged_post"] in posts


# todo test_nested_relationship_single_many
# todo test_nested_relationship_single_single
# todo test_nested_relationship_single_single_single ... etc

# TODO test non Eq conditions
# TODO test f(x) if not x.boolean_attr;
