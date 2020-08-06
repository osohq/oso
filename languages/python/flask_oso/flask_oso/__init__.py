__version__ = '0.0.0'

from .oso import FlaskOso
from .decorators import authorize, skip_authorization

__all__ = ['FlaskOso', 'authorize', 'skip_authorization']
