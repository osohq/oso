from django.db import models
from django_oso.manager import OsoManager


class TestRegistration(models.Model):
    class Meta:
        app_label = "test_app"


class TestRegistration2(models.Model):
    class Meta:
        app_label = "test_app"


class TestScope(models.Model):
    public = models.BooleanField()
    user = models.CharField(max_length=256)

    objects = models.Manager()
    authorized_objects = OsoManager()

    class Meta:
        app_label = "test_app"
