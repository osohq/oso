import pytest

from django_oso.models import AuthorizedModel, authorize_model
from django_oso.oso import Oso, reset_oso
from django.core.management import call_command

from app.models import Post, User


@pytest.fixture(autouse=True)
def reset():
    reset_oso()


@pytest.fixture
def users():
    (manager, _) = User.objects.get_or_create(username="manager")
    (user, _) = User.objects.get_or_create(username="user", manager=manager)
    return {"user": user, "manager": manager}


@pytest.fixture
def posts(users):
    (public_user_post, _) = Post.objects.get_or_create(
        contents="public user post", access_level="public", creator=users["user"]
    )
    (private_user_post, _) = Post.objects.get_or_create(
        contents="private user post", access_level="private", creator=users["user"]
    )
    (public_manager_post, _) = Post.objects.get_or_create(
        contents="public manager post",
        access_level="public",
        creator=users["manager"],
    )
    (private_manager_post, _) = Post.objects.get_or_create(
        contents="private manager post",
        access_level="private",
        creator=users["manager"],
    )
    return {
        "public_user_post": public_user_post,
        "private_user_post": private_user_post,
        "public_manager_post": public_manager_post,
        "private_manager_post": private_manager_post,
    }


@pytest.mark.django_db
def test_user_access_to_posts(users, posts):
    authorized_posts = Post.objects.authorize(None, actor=users["user"], action="GET")
    assert authorized_posts.count() == 3
    assert posts["public_user_post"] in authorized_posts
    assert posts["private_user_post"] in authorized_posts
    assert posts["public_manager_post"] in authorized_posts


@pytest.mark.django_db
def test_manager_access_to_posts(users, posts):
    authorized_posts = Post.objects.authorize(
        None, actor=users["manager"], action="GET"
    )
    assert authorized_posts.count() == 4
    assert posts["public_user_post"] in authorized_posts
    assert posts["private_user_post"] in authorized_posts
    assert posts["public_manager_post"] in authorized_posts
    assert posts["private_manager_post"] in authorized_posts
