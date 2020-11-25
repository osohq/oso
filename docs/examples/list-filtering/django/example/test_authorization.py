import pytest

from django_oso.models import AuthorizedModel, authorize_model
from django_oso.oso import Oso, reset_oso

from example.models import Post, User


@pytest.fixture(autouse=True)
def reset():
    reset_oso()


@pytest.fixture
def users():
    manager = User(username="manager")
    manager.save()
    user = User(username="user", manager=manager)
    user.save()
    return {"user": user, "manager": manager}


@pytest.fixture
def posts(users):
    public_user_post = Post(
        contents="public user post", access_level="public", creator=users["user"]
    )
    public_user_post.save()
    private_user_post = Post(
        contents="private user post", access_level="private", creator=users["user"]
    )
    private_user_post.save()
    public_manager_post = Post(
        contents="public manager post",
        access_level="public",
        creator=users["manager"],
    )
    public_manager_post.save()
    private_manager_post = Post(
        contents="private manager post",
        access_level="private",
        creator=users["manager"],
    )
    private_manager_post.save()
    return {
        "public_user_post": public_user_post,
        "private_user_post": private_user_post,
        "public_manager_post": public_manager_post,
        "private_manager_post": private_manager_post,
    }


@pytest.mark.django_db
def test_user_access_to_posts(users, posts):
    authorized_posts = Post.objects.authorize(None, actor=users["user"], action="read")
    assert authorized_posts.count() == 3
    assert posts["public_user_post"] in authorized_posts
    assert posts["private_user_post"] in authorized_posts
    assert posts["public_manager_post"] in authorized_posts


@pytest.mark.django_db
def test_manager_access_to_posts(users, posts):
    authorized_posts = Post.objects.authorize(
        None, actor=users["manager"], action="read"
    )
    assert authorized_posts.count() == 4
    assert posts["public_user_post"] in authorized_posts
    assert posts["private_user_post"] in authorized_posts
    assert posts["public_manager_post"] in authorized_posts
    assert posts["private_manager_post"] in authorized_posts
