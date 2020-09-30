from django.db import models

import pytest


class TestRegistration(models.Model):
    class Meta:
        app_label = "test_app"


class TestRegistration2(models.Model):
    class Meta:
        app_label = "test_app"


class Post(models.Model):
    is_private = models.BooleanField()
    name = models.CharField(max_length=256)
    timestamp = models.IntegerField()

    def __str__(self):
        return f"Post(name={self.name}, is_private={self.is_private})"

    class Meta:
        app_label = "test_app"
