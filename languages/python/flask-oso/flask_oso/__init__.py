__version__ = "0.20.0-beta"

from .flask_oso import FlaskOso
from .decorators import authorize, skip_authorization

__all__ = ["FlaskOso", "authorize", "skip_authorization"]
