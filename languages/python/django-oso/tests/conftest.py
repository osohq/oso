from pathlib import Path
import sys

from django.conf import settings


def pytest_configure():
    # Add test app to sys path
    test_app = Path(__file__).parent
    sys.path.append(test_app.as_posix())

    settings.configure(
        INSTALLED_APPS=[
            "test_app",
            "django_oso",
            "django.contrib.auth",
            "django.contrib.contenttypes",
        ],
        ROOT_URLCONF="django_oso.test_urls",
        DATABASES={
            "default": {"ENGINE": "django.db.backends.sqlite3", "NAME": ":memory:"}
        },
    )
