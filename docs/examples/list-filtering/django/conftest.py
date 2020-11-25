from pathlib import Path
import sys

from django.conf import settings

BASE_DIR = Path(__file__).resolve().parent.parent


def pytest_configure():
    # Add example app to sys path.
    parent_dir = Path(__file__).parent
    sys.path.append(parent_dir.as_posix())

    settings.configure(
        INSTALLED_APPS=[
            "app",
            "django_oso",
            "django.contrib.auth",
            "django.contrib.contenttypes",
        ],
        DATABASES={
            "default": {
                "ENGINE": "django.db.backends.sqlite3",
                "NAME": BASE_DIR / "db.sqlite3",
            }
        },
        AUTH_USER_MODEL="app.User",
    )
