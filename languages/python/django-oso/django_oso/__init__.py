import django
import oso

from .oso import Oso

__version__ = "0.27.0"

if django.VERSION < (3, 2):
    default_app_config = "django_oso.apps.DjangoOsoConfig"

__all__ = ["Oso", "oso"]
