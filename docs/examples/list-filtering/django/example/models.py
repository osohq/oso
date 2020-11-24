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
        app_label = "example"


class User(AuthorizedModel):
    username = models.CharField(max_length=255)
    is_admin = models.BooleanField(default=False)
    manager = models.ForeignKey(
        "self", null=True, related_name="direct_reports", on_delete=models.CASCADE
    )

    class Meta:
        app_label = "example"
