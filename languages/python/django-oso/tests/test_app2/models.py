from django.db import models

from django_oso.models import AuthorizedModel

import pytest


class User(models.Model):
    username = models.CharField(max_length=255)
    is_moderator = models.BooleanField(default=False)
    is_banned = models.BooleanField(default=False)
    posts = models.ManyToManyField("Post")

    class Meta:
        app_label = "test_app2"


class Tag(AuthorizedModel):
    name = models.CharField(max_length=255)
    created_by = models.ForeignKey(User, on_delete=models.CASCADE, null=True)
    users = models.ManyToManyField(User)
    is_public = models.BooleanField(default=False)

    class Meta:
        app_label = "test_app2"


class Post(AuthorizedModel):
    contents = models.CharField(max_length=255)
    AccessLevelType = models.TextChoices("AccessLevelType", "public private")
    access_level = models.CharField(
        choices=AccessLevelType.choices, max_length=7, default="private"
    )
    created_by = models.ForeignKey(User, on_delete=models.CASCADE)
    needs_moderation = models.BooleanField(default=False)
    tags = models.ManyToManyField(Tag)

    class Meta:
        app_label = "test_app2"
