import os.path

from django.apps import AppConfig, apps
from django.http import HttpRequest
from django.utils.autoreload import autoreload_started

from oso import Oso as _Oso
from polar.exceptions import DuplicateClassAliasError

Oso = _Oso()
"""Singleton :py:class:`oso.Oso` instance.

Use for loading policy files and registering classes.
"""


def reset_oso():
    """Reset the state of :py:data:`~django_oso.oso.Oso`.

    Useful as a test helper to clean state between tests, but generally should
    not be used otherwise.
    """
    Oso.clear()
    init_oso()


def init_oso():
    def register_class(model, name=None):
        try:
            Oso.register_class(model, name=name)
        except DuplicateClassAliasError as e:
            pass

    # Register all models.
    for app in apps.get_app_configs():
        for model in app.get_models():
            app_namespace = app.name.replace(".", "::")
            name = f"{app_namespace}::{model.__name__}"
            register_class(model, name)

    # Custom registration for auth (AnonymousUser)
    if apps.is_installed("django.contrib.auth"):
        from django.contrib.auth.models import AnonymousUser

    register_class(AnonymousUser, name=f"django::contrib::auth::AnonymousUser")

    # Register request
    register_class(HttpRequest)

    loaded_files = []

    # Load all polar files in each app's "policy" directory.
    for app in apps.get_app_configs():
        policy_dir = os.path.join(app.path, "policy")
        for path, _, filenames in os.walk(policy_dir):
            for file in filenames:
                file_path = os.path.join(path, file)
                if os.path.splitext(file)[1] == ".polar":
                    Oso.load_file(file_path)
                    print(f"load {file_path}")
                    loaded_files.append(file_path)

    return loaded_files
