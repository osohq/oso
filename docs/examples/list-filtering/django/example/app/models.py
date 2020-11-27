from django.db import models
from django.contrib.auth.models import AbstractUser

from django_oso.models import AuthorizedModel


class User(AbstractUser):
    is_admin = models.BooleanField(default=False)
    manager = models.ForeignKey(
        "self", null=True, related_name="direct_reports", on_delete=models.CASCADE
    )


class Post(AuthorizedModel):
    contents = models.CharField(max_length=255)
    AccessLevelType = models.TextChoices("AccessLevelType", "public private")
    access_level = models.CharField(
        choices=AccessLevelType.choices, max_length=7, default="private"
    )
    creator = models.ForeignKey(User, on_delete=models.CASCADE)

    class Meta:
        app_label = "app"
