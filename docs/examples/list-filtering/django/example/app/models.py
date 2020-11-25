from django.db import models

from django_oso.models import AuthorizedModel


class Post(AuthorizedModel):
    contents = models.CharField(max_length=255)
    AccessLevelType = models.TextChoices("AccessLevelType", "public private")
    access_level = models.CharField(
        choices=AccessLevelType.choices, max_length=7, default="private"
    )
    creator = models.ForeignKey("User", on_delete=models.CASCADE)

    class Meta:
        app_label = "app"


class User(models.Model):
    username = models.CharField(max_length=255, unique=True)
    is_admin = models.BooleanField(default=False)
    manager = models.ForeignKey(
        "self", null=True, related_name="direct_reports", on_delete=models.CASCADE
    )
    is_anonymous = False
    is_authenticated = True

    USERNAME_FIELD = "username"
    REQUIRED_FIELDS = []

    class Meta:
        app_label = "app"
