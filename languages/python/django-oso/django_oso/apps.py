import functools
from pathlib import Path

from django.apps import AppConfig
from django.utils.autoreload import autoreload_started

from .oso import init_oso
from .settings import OSO_RELOAD_SERVER


def watch_files(files, sender, **kwargs):
    for file in files:
        sender.extra_files.add(Path(file))


class DjangoOsoConfig(AppConfig):
    name = "django_oso"

    def ready(self):
        loaded_files = init_oso()
        if OSO_RELOAD_SERVER:
            autoreload_started.connect(
                functools.partial(watch_files, files=loaded_files), weak=False
            )
