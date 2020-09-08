from django.conf import settings


def pytest_configure():
    settings.configure(
        INSTALLED_APPS=[
            "django_oso",
            "django.contrib.auth",
            "django.contrib.contenttypes",
        ],
        ROOT_URLCONF="django_oso.test_urls",
    )
