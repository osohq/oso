"""Standardized tests for adapters based on the Post model.

Tests come from the relationship document & operations laid out there.
"""
import pytest
from django.core.exceptions import PermissionDenied
from test_app2.models import Post, Tag, User

from django_oso.models import authorize_model
from django_oso.oso import reset_oso
from django_oso.partial import TRUE_FILTER

from .conftest import is_true, parenthesize


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
def test_authorize_model_basic(post_fixtures, load_additional_str):
    """Test that a simple policy with checks on non-relationship attributes is correct."""
    load_additional_str(
        """
        allow(u, "read", post: test_app2::Post) if u in ["admin", "user"] and post.access_level = "public";
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
    assert str(authorize_filter) == str(TRUE_FILTER)
    posts = Post.objects.filter(authorize_filter)
    assert posts.count() == 9

    authorize_filter = authorize_model(None, Post, actor="moderator", action="read")
    expected = """
         (OR:
            (AND:
                ('access_level', 'private'),
                ('needs_moderation', True)),
            (AND:
                ('access_level', 'public'),
                ('needs_moderation', True)))
    """
    assert str(authorize_filter) == " ".join(expected.split())
    posts = Post.objects.filter(authorize_filter)
    assert posts.count() == 4
    assert posts.all()[0].contents == "private for moderation"
    assert posts.all()[1].contents == "public for moderation"

    # Not authorized
    with pytest.raises(PermissionDenied):
        authorize_model(None, Post, actor="guest", action="read")


@pytest.mark.django_db
def test_authorize_scalar_attribute_eq(post_fixtures, load_additional_str):
    """Test authorization rules on a relationship with one object equaling another."""
    # Object equals another object
    load_additional_str(
        """
        allow(actor: test_app2::User, "read", _: test_app2::Post{created_by: actor, access_level: "private"});
        allow(_: test_app2::User, "read", post) if
            post matches test_app2::Post{access_level: "public"};
        allow(_: test_app2::User{is_moderator: true}, "read", post: test_app2::Post) if
            post matches {access_level: "public"};
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
def test_authorize_scalar_attribute_condition(post_fixtures, load_additional_str):
    """Scalar attribute condition checks."""
    load_additional_str(
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
        return (post.access_level == "public" and not post.created_by.is_banned) or (
            post.access_level == "private" and post.created_by == user
        )

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
def test_in_multiple_attribute_relationship(tag_fixtures, load_additional_str):
    load_additional_str(
        """
        allow(_: test_app2::User, "read", post: test_app2::Post) if
            post.access_level = "public";
        allow(user: test_app2::User, "read", post: test_app2::Post) if
            post.access_level = "private" and
            post.created_by = user;
        allow(_: test_app2::User, "read", post: test_app2::Post) if
            tag in post.tags and
            0 < tag.id and
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


@pytest.mark.django_db
def test_nested_relationship_many_single(tag_nested_fixtures, load_additional_str):
    """Test that nested relationships work.
    post - (many) -> tags - (single) -> User
    A user can read a post with a tag if the tag's creator is the user.
    """
    load_additional_str(
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

    other = Tag(name="other tag")
    other.save()
    eng = Tag(name="eng")
    eng.save()
    eng.users.set([user])
    user_posts = Tag(name="user_posts")
    user_posts.save()
    user_posts.users.set([user])
    random = Tag(name="random", is_public=True)
    random.save()
    random.users.set([other_user])

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
    other_tagged_post = Post(
        contents="other tagged post",
        access_level="public",
        created_by=user,
    )

    posts = {
        "user_eng_post": user_eng_post,
        "user_user_post": user_user_post,
        "random_post": random_post,
        "not_tagged_post": not_tagged_post,
        "all_tagged_post": all_tagged_post,
        "other_tagged_post": other_tagged_post,
    }
    for post in posts.values():
        post.save()

    user_eng_post.tags.set([eng])
    user_user_post.tags.set([user_posts])
    random_post.tags.set([random])
    other_tagged_post.tags.set([other])
    all_tagged_post.tags.set([eng, user_posts, random])

    user.posts.set(
        [
            user_eng_post,
            user_user_post,
            not_tagged_post,
            all_tagged_post,
            other_tagged_post,
        ]
    )
    other_user.posts.set([random_post])

    return posts


@pytest.mark.django_db
def test_nested_relationship_many_many(
    tag_nested_many_many_fixtures, load_additional_str
):
    """Test that nested relationships work.
    post - (many) -> tags - (many) -> User
    A user can read a post with a tag if the tag's creator is the user.
    """
    load_additional_str(
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


@pytest.mark.django_db
def test_partial_in_collection(tag_nested_many_many_fixtures, load_additional_str):
    load_additional_str(
        """
            allow(user: test_app2::User, "read", post: test_app2::Post) if
                post in user.posts.all();
        """
    )

    user = User.objects.get(username="user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter)
    assert tag_nested_many_many_fixtures["user_eng_post"] in posts
    assert tag_nested_many_many_fixtures["user_user_post"] in posts
    assert tag_nested_many_many_fixtures["random_post"] not in posts
    assert tag_nested_many_many_fixtures["not_tagged_post"] in posts
    assert tag_nested_many_many_fixtures["all_tagged_post"] in posts

    user = User.objects.get(username="other_user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter)
    assert tag_nested_many_many_fixtures["user_eng_post"] not in posts
    assert tag_nested_many_many_fixtures["user_user_post"] not in posts
    assert tag_nested_many_many_fixtures["random_post"] in posts
    assert tag_nested_many_many_fixtures["not_tagged_post"] not in posts
    assert tag_nested_many_many_fixtures["all_tagged_post"] not in posts


@pytest.mark.django_db
def test_many_many_with_other_condition(
    tag_nested_many_many_fixtures, load_additional_str
):
    """Test that using a many-to-many condition OR any other condition does not
    result in duplicate results."""
    load_additional_str(
        """
            allow(_: test_app2::User, "read", post: test_app2::Post) if
                tag in post.tags and
                tag.name = "eng";
            allow(_: test_app2::User, "read", post: test_app2::Post) if
                post.access_level = "public";
        """
    )
    user = User.objects.get(username="user")
    posts = Post.objects.authorize(None, actor=user, action="read")
    expected = f"""
       SELECT "test_app2_post"."id", "test_app2_post"."contents", "test_app2_post"."title",
              "test_app2_post"."access_level",
              "test_app2_post"."created_by_id", "test_app2_post"."needs_moderation"
       FROM "test_app2_post"
       WHERE "test_app2_post"."id" IN
           (SELECT W0."id"
            FROM "test_app2_post" W0
            WHERE
                (W0."id" IN
                    (SELECT V0."id"
                    FROM "test_app2_post" V0
                    LEFT OUTER JOIN "test_app2_post_tags" V1 ON (V0."id" = V1."post_id")
                    WHERE
                        EXISTS(SELECT U0."id"
                                FROM "test_app2_tag" U0
                                WHERE (U0."id" = {parenthesize('V1."tag_id"')} AND U0."name" = eng)){is_true()})
                        OR W0."access_level" = public))
    """
    assert str(posts.query) == " ".join(expected.split())
    # all should be returned with no duplicates
    assert list(posts) == list(tag_nested_many_many_fixtures.values())


@pytest.mark.django_db
def test_empty_constraints_in(tag_nested_many_many_fixtures, load_additional_str):
    """Test that ``unbound in partial.field`` without any further constraints
    on unbound translates into an existence check."""
    load_additional_str(
        """
            allow(_: test_app2::User, "read", post: test_app2::Post) if
                _tag in post.tags;
        """
    )
    user = User.objects.get(username="user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter).distinct()
    expected = f"""
        SELECT DISTINCT "test_app2_post"."id", "test_app2_post"."contents", "test_app2_post"."title",
                        "test_app2_post"."access_level", "test_app2_post"."created_by_id",
                        "test_app2_post"."needs_moderation"
        FROM "test_app2_post"
        WHERE "test_app2_post"."id" IN
            (SELECT V0."id"
             FROM "test_app2_post" V0
             LEFT OUTER JOIN "test_app2_post_tags" V1 ON (V0."id" = V1."post_id")
             WHERE EXISTS(SELECT U0."id"
                          FROM "test_app2_tag" U0
                          WHERE U0."id" = {parenthesize('V1."tag_id"')}){is_true()})
    """
    assert str(posts.query) == " ".join(expected.split())
    assert len(posts) == 5
    assert tag_nested_many_many_fixtures["not_tagged_post"] not in posts


@pytest.mark.django_db
def test_in_with_constraints_but_no_matching_objects(
    tag_nested_many_many_fixtures, load_additional_str
):
    load_additional_str(
        """
            allow(_: test_app2::User, "read", post: test_app2::Post) if
                tag in post.tags and
                tag.name = "bloop";
        """
    )
    user = User.objects.get(username="user")
    posts = Post.objects.authorize(None, actor=user, action="read")
    expected = f"""
        SELECT "test_app2_post"."id", "test_app2_post"."contents", "test_app2_post"."title", "test_app2_post"."access_level",
               "test_app2_post"."created_by_id", "test_app2_post"."needs_moderation"
        FROM "test_app2_post"
        WHERE "test_app2_post"."id" IN (SELECT W0."id"
                                        FROM "test_app2_post" W0
                                        WHERE W0."id" IN
                                            (SELECT V0."id"
                                             FROM "test_app2_post" V0
                                             LEFT OUTER JOIN "test_app2_post_tags" V1 ON (V0."id" = V1."post_id")
                                             WHERE EXISTS(SELECT U0."id"
                                                          FROM "test_app2_tag" U0
                                                          WHERE (U0."id" = {parenthesize('V1."tag_id"')}
                                                          AND U0."name" = bloop)){is_true()}))
    """
    assert str(posts.query) == " ".join(expected.split())
    assert len(posts) == 0


@pytest.mark.django_db
def test_reverse_many_relationship(tag_nested_many_many_fixtures, load_additional_str):
    """Test an authorization rule over a reverse relationship"""
    load_additional_str(
        """
        allow(actor, _, post: test_app2::Post) if
            post.users matches test_app2::User and
            actor in post.users;
        """
    )

    user = User.objects.get(username="user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    assert str(authorize_filter) == "(AND: ('users', <User: User object (1)>))"
    posts = Post.objects.filter(authorize_filter)
    expected = """
        SELECT "test_app2_post"."id", "test_app2_post"."contents", "test_app2_post"."title", "test_app2_post"."access_level",
               "test_app2_post"."created_by_id", "test_app2_post"."needs_moderation"
        FROM "test_app2_post"
        INNER JOIN "test_app2_user_posts" ON ("test_app2_post"."id" = "test_app2_user_posts"."post_id")
        WHERE "test_app2_user_posts"."user_id" = 1
    """
    assert str(posts.query) == " ".join(expected.split())
    assert len(posts) == 5


@pytest.mark.xfail(reason="Cannot compare items across subqueries.")
@pytest.mark.django_db
def test_deeply_nested_in(tag_nested_many_many_fixtures, load_additional_str):
    load_additional_str(
        """
        allow(_, _, post: test_app2::Post) if
            foo in post.created_by.posts and foo.id > 1 and
            bar in foo.created_by.posts and bar.id > 2 and
            baz in bar.created_by.posts and baz.id > 3 and
            post in baz.created_by.posts and post.id > 4;
        """
    )
    user = User.objects.get(username="user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter).distinct()
    expected = """
        SELECT DISTINCT "test_app2_post"."id", "test_app2_post"."contents", "test_app2_post"."title",
                        "test_app2_post"."access_level", "test_app2_post"."created_by_id",
                        "test_app2_post"."needs_moderation"
        FROM "test_app2_post"
        INNER JOIN "test_app2_user" ON ("test_app2_post"."created_by_id" = "test_app2_user"."id")
        LEFT OUTER JOIN "test_app2_user_posts" ON ("test_app2_user"."id" = "test_app2_user_posts"."user_id")
        WHERE (EXISTS(SELECT W0."id"
                      FROM "test_app2_post" W0
                      INNER JOIN "test_app2_user" W1 ON (W0."created_by_id" = W1."id")
                      LEFT OUTER JOIN "test_app2_user_posts" W2 ON (W1."id" = W2."user_id")
                      WHERE (EXISTS(SELECT V0."id"
                                    FROM "test_app2_post" V0
                                    INNER JOIN "test_app2_user" V1 ON (V0."created_by_id" = V1."id")
                                    LEFT OUTER JOIN "test_app2_user_posts" V2 ON (V1."id" = V2."user_id")
                                    WHERE (EXISTS(SELECT U0."id"
                                                  FROM "test_app2_post" U0
                                                  INNER JOIN "test_app2_user" U1 ON (U0."created_by_id" = U1."id")
                                                  INNER JOIN "test_app2_user_posts" U2 ON (U1."id" = U2."user_id")
                                                  WHERE (U0."id" = V2."post_id"
                                                         AND U0."id" > 3

                                                         # This is not the sql that is generated.
                                                         # Instead U0."id" is the LHS of below.
                                                         AND "test_app2_post"."id" = U2."post_id"))
                                           AND V0."id" = W2."post_id"
                                           AND V0."id" > 2))
                             AND W0."id" = "test_app2_user_posts"."post_id"
                             AND W0."id" > 1))
               AND "test_app2_post"."id" > 4)
    """
    assert str(posts.query) == " ".join(expected.split())
    assert len(posts) == 1


@pytest.mark.skip("Don't currently handle this case.")
@pytest.mark.django_db
def test_unify_ins(tag_nested_many_many_fixtures, load_additional_str):
    load_additional_str(
        """
        allow(_, _, post) if
            user1 in post.users and
            user2 in post.users and
            user1.id = user2.id and
            user1.id > 1 and
            user2.id <= 2;
        """
    )
    user = User.objects.get(username="user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter)
    expected = """
        SELECT "test_app2_post"."id", "test_app2_post"."contents", "test_app2_post"."title", "test_app2_post"."access_level",
               "test_app2_post"."created_by_id", "test_app2_post"."needs_moderation"
        FROM "test_app2_post"
        LEFT OUTER JOIN "test_app2_user_posts" ON ("test_app2_post"."id" = "test_app2_user_posts"."post_id")
        WHERE (EXISTS(SELECT U0."id"
                      FROM "test_app2_user" U0
                      INNER JOIN "test_app2_user" V0 ON (U0."id" = V0."id")
                      WHERE (U0."id" = "test_app2_user_posts"."user_id"
                             AND V0."id" = "test_app2_user_posts"."user_id"
                             AND U0."id" <= 2
                             AND V0."id" > 1)))
    """
    assert str(posts.query) == " ".join(expected.split())
    assert len(posts) == 1


@pytest.mark.skip("Don't currently handle this case.")
@pytest.mark.django_db
def test_this_in_var(tag_nested_many_many_fixtures, load_additional_str):
    load_additional_str(
        """
        # _this in var
        allow(_, _, post: test_app2::Post) if
            post in x and
            x in post.created_by.posts;
        """
    )
    user = User.objects.get(username="user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter)
    expected = """
    """
    assert str(posts.query) == " ".join(expected.split())
    assert len(posts) == 5050


@pytest.mark.skip("Don't currently handle this case.")
@pytest.mark.django_db
def test_var_in_other_var(tag_nested_many_many_fixtures, load_additional_str):
    load_additional_str(
        """
        # var in other_var
        allow(_, _, post: test_app2::Post) if
            x in y and
            y in post.created_by.posts
            and post.id = x.id;
        """
    )
    user = User.objects.get(username="user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter)
    expected = """
    """
    assert str(posts.query) == " ".join(expected.split())
    assert len(posts) == 5050


@pytest.mark.django_db
def test_in_intersection(tag_nested_many_many_fixtures, load_additional_str):
    load_additional_str(
        """
        allow(_, _, post) if
            u in post.users and
            t in post.tags and
            u in t.users;
        """
    )
    user = User.objects.get(username="user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter)
    expected = f"""
        SELECT "test_app2_post"."id", "test_app2_post"."contents", "test_app2_post"."title", "test_app2_post"."access_level",
               "test_app2_post"."created_by_id", "test_app2_post"."needs_moderation"
        FROM "test_app2_post"
        WHERE "test_app2_post"."id"
        IN (SELECT X0."id"
            FROM "test_app2_post" X0
            LEFT OUTER JOIN "test_app2_user_posts" X1 ON (X0."id" = X1."post_id")
            LEFT OUTER JOIN "test_app2_post_tags" X3 ON (X0."id" = X3."post_id")
            WHERE (EXISTS(SELECT U0."id"
                   FROM "test_app2_user" U0
                   WHERE U0."id" = {parenthesize('X1."user_id"')}){is_true()}
                   AND EXISTS(SELECT W0."id"
                       FROM "test_app2_tag" W0
                       WHERE (W0."id" = {parenthesize('X3."tag_id"')}
                       AND W0."id" IN
                            (SELECT V0."id"
                             FROM "test_app2_tag" V0
                             LEFT OUTER JOIN "test_app2_tag_users" V1 ON (V0."id" = V1."tag_id")
                             WHERE EXISTS(SELECT U0."id"
                                         FROM "test_app2_user" U0
                                         WHERE U0."id" = {parenthesize('V1."user_id"')}){is_true()}))){is_true()}))
    """
    assert str(posts.query) == " ".join(expected.split())
    assert len(posts) == 4


@pytest.mark.django_db
def test_redundant_in_on_same_field(tag_nested_many_many_fixtures, load_additional_str):
    load_additional_str(
        """
        allow(_, "read", post) if
            tag1 in post.tags and
            tag2 in post.tags and
            tag1.name = "random" and
            tag2.is_public = true;
        """
    )
    user = User.objects.get(username="user")
    authorize_filter = authorize_model(None, Post, actor=user, action="read")
    posts = Post.objects.filter(authorize_filter)
    expected = f"""

        SELECT "test_app2_post"."id", "test_app2_post"."contents", "test_app2_post"."title", "test_app2_post"."access_level",
               "test_app2_post"."created_by_id", "test_app2_post"."needs_moderation"
        FROM "test_app2_post"
        WHERE "test_app2_post"."id" IN (SELECT V0."id"
                                        FROM "test_app2_post" V0
                                        LEFT OUTER JOIN "test_app2_post_tags" V1 ON (V0."id" = V1."post_id")
                                        WHERE (EXISTS(SELECT U0."id"
                                                      FROM "test_app2_tag" U0
                                                      WHERE (U0."id" = {parenthesize('V1."tag_id"')}
                                                      AND U0."name" = random)){is_true()}
                                                      AND EXISTS(SELECT U0."id"
                                                                 FROM "test_app2_tag" U0
                                                                 WHERE (U0."id" = {parenthesize('V1."tag_id"')}
                                                                 AND U0."is_public"{is_true()})){is_true()}))
    """
    assert str(posts.query) == " ".join(expected.split())
    assert len(posts) == 2


# todo test_nested_relationship_single_many
# todo test_nested_relationship_single_single
# todo test_nested_relationship_single_single_single ... etc

# TODO test non Eq conditions
# TODO test f(x) if not x.boolean_attr;
