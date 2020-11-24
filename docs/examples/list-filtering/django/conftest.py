from pathlib import Path
import sys

from django.conf import settings


def pytest_configure():
    # Add example app to sys path.
    parent_dir = Path(__file__).parent
    sys.path.append(parent_dir.as_posix())

    settings.configure(
        INSTALLED_APPS=[
            "example",
            "django_oso",
            "django.contrib.auth",
            "django.contrib.contenttypes",
        ],
        ROOT_URLCONF="django_oso.test_urls",
        DATABASES={
            "default": {"ENGINE": "django.db.backends.sqlite3", "NAME": ":memory:"}
        },
    )
