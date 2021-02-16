from pathlib import Path
import sys

from django import VERSION
from django.conf import settings


def pytest_configure():
    # Add test apps to sys path
    test_app = Path(__file__).parent
    sys.path.append(test_app.as_posix())

    settings.configure(
        INSTALLED_APPS=[
            "test_app",
            "test_app2",
            "django_oso",
            "django.contrib.auth",
            "django.contrib.contenttypes",
        ],
        ROOT_URLCONF="test_urls",
        DATABASES={
            "default": {"ENGINE": "django.db.backends.sqlite3", "NAME": ":memory:"}
        },
    )


def negated_condition(variable):
    if VERSION >= (3, 1):
        return f"NOT {variable}"
    else:
        return f"{variable} = False"


def parenthesize(variable):
    if VERSION >= (3,):
        return variable
    else:
        return f"({variable})"


def is_true():
    if VERSION >= (3, 1):
        return ""
    else:
        return " = True"
