"""Core oso functionality"""

__version__ = "0.1.0"

from pathlib import Path
import os
from typing import Any, cast, Callable, List, TYPE_CHECKING
import inspect

from polar import CLASSES, CONSTRUCTORS, Polar, Predicate, Query, QueryResult

from .extras import Http, PathMapper

if TYPE_CHECKING:
    import flask


class OsoException(Exception):
    pass


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

    # TODO (dhatch): should we name this 'is_allowed'?
    def allow(self, actor, action, resource) -> bool:
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
        result = self.query_predicate("allow", actor, action, resource, single=True)
        return result.success


def polar_class(_cls=None, *, name=None, from_polar=None):
    """Decorator to register a Python class with Polar. An alternative to ``register_class()``.

    :param str from_polar: Name of static class function to create a new class instance from ``fields``.
                            Defaults to class constructor.
    """

    def wrap(cls):
        cls_name = cls.__name__ if name is None else name
        CLASSES[cls_name] = cls
        CONSTRUCTORS[cls_name] = from_polar or cls
        return cls

    if _cls is None:
        return wrap

    return wrap(_cls)
