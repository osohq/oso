from django.core.management.base import BaseCommand

from app.models import Post, User


class Command(BaseCommand):
    help = "seed database for testing and development."

    def handle(self, *args, **options):
        seed()


def seed():
    (manager, _) = User.objects.get_or_create(username="manager")
    (user, _) = User.objects.get_or_create(username="user", manager=manager)

    Post.objects.get_or_create(
        contents="public user post", access_level="public", creator=user
    )
    Post.objects.get_or_create(
        contents="private user post", access_level="private", creator=user
    )
    Post.objects.get_or_create(
        contents="public manager post",
        access_level="public",
        creator=manager,
    )
    Post.objects.get_or_create(
        contents="private manager post",
        access_level="private",
        creator=manager,
    )
