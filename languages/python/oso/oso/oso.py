"""Core oso functionality"""

__version__ = "0.8.0"

from pathlib import Path
import os
from typing import Any, cast, Callable, List, TYPE_CHECKING
import inspect

from polar import Polar

from .extras import Http, PathMapper
from polar.exceptions import OsoError

if TYPE_CHECKING:
    import flask


class Oso(Polar):
    """The central object to manage application policy state, e.g.
    the policy data, and verify requests.

    >>> Oso()
    <oso.Oso object at 0x7fad57305100>

    """

    def __init__(self):
        """Create an oso object."""
        super().__init__()

        # Register built-in classes.
        self.register_class(Http)
        self.register_class(PathMapper)

    def is_allowed(self, actor, action, resource) -> bool:
        """Evaluate whether ``actor`` is allowed to perform ``action`` on ``resource``.

        Uses allow rules in the Polar policy to determine whether a request is
        permitted. ``actor`` and ``resource`` should be classes that have been
        registered with Polar using the :py:func:`register_class` function or
        the ``polar_class`` decorator.

        :param actor: The actor performing the request.
        :param action: The action the actor is attempting to perform.
        :param resource: The resource being accessed.

        :return: ``True`` if the request is allowed, ``False`` otherwise.
        """
        try:
            next(self.query_rule("allow", actor, action, resource))
            return True
        except StopIteration:
            return False
