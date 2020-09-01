import os.path

from django.apps import AppConfig, apps
from django.http import HttpRequest
from django.utils.autoreload import autoreload_started

from oso import Oso as _Oso

Oso = _Oso()

def reset_oso():
    Oso.clear()
    init_oso()

def init_oso():
    # Register all models.
    for app in apps.get_app_configs():
        for model in app.get_models():
            print(f"Register {model}")
            Oso.register_class(model)

    # Custom registration for auth (AnonymousUser)
    if apps.is_installed('django.contrib.auth'):
        from django.contrib.auth.models import AnonymousUser
        Oso.register_class(AnonymousUser)

    # Register request
    Oso.register_class(HttpRequest)

    loaded_files = []

    # Load all polar files in each app's "policy" directory.
    for app in apps.get_app_configs():
        policy_dir = os.path.join(app.path, 'policy')
        for path, _, filenames in os.walk(policy_dir):
            for file in filenames:
                path = os.path.join(path, file)
                if os.path.splitext(file)[1] == '.polar':
                    print(f"load file {path}")
                    Oso.load_file(path)
                    loaded_files.append(path)

    return loaded_files

    # TODO (dhatch): Provide setting to disable auto loading
    # customize file directory
    # customize policy files
    # document how to do manual load.

    # ?? NAMESPACING ??

    # TODO (dhatch): Provide setting to disable model registration.

