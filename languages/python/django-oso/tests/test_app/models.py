from django.db import models

from django_oso.models import AuthorizedModel


class TestRegistration(models.Model):
    class Meta:
        app_label = "test_app"


class TestRegistration2(models.Model):
    class Meta:
        app_label = "test_app"


class User(models.Model):
    name = models.CharField(max_length=256)

    class Meta:
        app_label = "test_app"


class Admin(User):
    pass


class Post(AuthorizedModel):
    is_private = models.BooleanField(null=True)
    name = models.CharField(max_length=256, null=True)
    timestamp = models.IntegerField(null=True)
    option = models.BooleanField(null=True)
    created_by = models.ForeignKey(User, on_delete=models.CASCADE, null=True)

    def __str__(self):
        return f"Post(name={self.name}, is_private={self.is_private})"

    class Meta:
        app_label = "test_app"
