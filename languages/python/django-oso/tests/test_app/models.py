from django.db import models


class TestRegistration(models.Model):
    class Meta:
        app_label = "test_app"


class TestRegistration2(models.Model):
    class Meta:
        app_label = "test_app"
