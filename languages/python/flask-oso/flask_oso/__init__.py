__version__ = "0.5.0"

from .flask_oso import FlaskOso
from .decorators import authorize, skip_authorization

__all__ = ["FlaskOso", "authorize", "skip_authorization"]
