"""Core oso functionality"""

# Builds on the polar.api functionality as a user-facing module

# External imports
from pathlib import Path
import os
from typing import Any, cast, Callable, List, TYPE_CHECKING
import inspect

from polar import api
from polar.api import Http, Polar, Predicate, QueryResult

if TYPE_CHECKING:
    import flask

# oso version
__version__ = "0.1.0rc8"


class OsoException(Exception):
    pass


class Oso(api.Polar):
    """The central object to manage application policy state, e.g.
    the policy data, verifying requests, and loading the visualizer.

    >>> Oso()
    <oso.Oso object at 0x7fad57305100>

    """

    def __init__(self, enable_audit: bool = False):
        """Create an oso object."""
        super().__init__()

    # TODO (dhatch): should we name this 'is_allowed'?
    def allow(self, actor, action, resource, debug=False) -> bool:
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
        # actor + resource are python classes
        return self._query_pred(
            Predicate(name="allow", args=[actor, action, resource]),
            debug=debug,
            single=True,
        ).success


def polar_class(_cls=None, *, from_polar=None):
    """Decorator to register a Python class with Polar. An alternative to ``register_class()``.

    :param str from_polar: Name of static class function to create a new class instance from ``fields``.
                            Defaults to class constructor.
    """

    def wrap(cls):
        Polar().register_class(cls, from_polar)
        return cls

    if _cls is None:
        return wrap

    return wrap(_cls)
