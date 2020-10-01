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


def get_model_name(model):
    app_name = model._meta.app_label
    app_namespace = app_name.replace(".", "::")
    return f"{app_namespace}::{model.__name__}"


def init_oso():
    def register_class(model, name=None):
        try:
            Oso.register_class(model, name=name)
        except DuplicateClassAliasError as e:
            pass

    # Register all models.
    for app in apps.get_app_configs():
        for model in app.get_models():
            register_class(model, get_model_name(model))

    # Custom registration for auth (AnonymousUser)
    if apps.is_installed("django.contrib.auth"):
        from django.contrib.auth.models import AnonymousUser

    register_class(AnonymousUser, name=f"django::contrib::auth::AnonymousUser")

    # Register request
    register_class(HttpRequest)

    return load_policy_files()


def load_policy_files():
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


def reset_oso():
    Oso.clear_rules()
    load_policy_files()
