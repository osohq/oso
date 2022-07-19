import oso

from .oso import Oso

__version__ = "0.26.0"

default_app_config = "django_oso.apps.DjangoOsoConfig"

__all__ = ["Oso", "oso"]
