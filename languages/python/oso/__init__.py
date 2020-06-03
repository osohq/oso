"""Core oso functionality"""

# Builds on the polar.api functionality as a user-facing module

# External imports
from pathlib import Path
import os
from typing import Any, cast, Callable, List, TYPE_CHECKING
import inspect

from polar import api
from polar.api import Polar, Http, Query, QueryResult

if TYPE_CHECKING:
    import flask

# oso version
__version__ = "0.1.0rc2"


class OsoException(Exception):
    pass


class Oso(api.Polar):
    """The central object to manage application policy state, e.g.
    the policy data, verifying requests, and loading the visualizer.

    >>> Oso()
    <oso.Oso object at 0x7fad57305100>

    """

    def __init__(self, enable_audit: bool = False):
        """Create an oso object.

        Optionally ``kb`` can be provided, which will use an already created
        polar knowledge base.
        """
        super().__init__()
        # Load the base policy
        # self.import_builtin_module("authorization")

        # if enable_audit:
        #     audit.enable()

    def filter_map(
        self,
        request: "flask.Request",
        f: Callable,
        credentials=None,
        credential_header=None,
        hostname="",
    ) -> List[Any]:
        """Filter out unauthorized results for a Flask endpoint, and map over
        the authorized results.

        :param request: The flask request.
        :param f: The function to be called on each query result.

        :return: List of filtered query results.
        """
        if not credentials and credential_header:
            credentials = request.headers.get(credential_header, None)
        if not credentials:
            credentials = {}
        action = request.method.lower()
        resource = api.Http(path=request.path, hostname=hostname)
        query = Query(name="allow", args=(credentials, action, resource))
        return list(f(r) for r in self.query(query, single=True).results if f(r))

    def verify_flask_request(
        self,
        request: "flask.Request",
        credentials=None,
        credential_header=None,
        hostname="",
    ) -> bool:
        """Verify a Flask request
        Credentials can be an "Actor" class, a dictionary of attributes or a string.
        credential_header is the name of a header to read the credentials from.
        """
        if not credentials and credential_header:
            credentials = request.headers.get(credential_header, None)
        if not credentials:
            credentials = {}
        action = request.method.lower()
        resource = api.Http(path=request.path, hostname=hostname)
        query = Query(name="allow", args=(credentials, action, resource))
        return self.query(query, single=True).success

    # oso.actions(["create", "read", "update", "delete"])

    # TODO (dhatch): should we name this 'is_allowed'?
    def allow(self, actor, action, resource, debug=False) -> bool:
        """Evaluate whether ``actor`` is allowed to perform ``action`` on ``resource``.

        Uses allow rules in the Polar policy to determine whether a request is
        permitted. ``actor`` and ``resource`` should be classes that have been
        registered with Polar using the :py:func:`register_python_class` function or
        the ``polar_class`` decorator.

        :param actor: The actor performing the request.
        :param action: The action the actor is attempting to perform.
        :param resource: The resource being accessed.

        :return: ``True`` if the request is allowed, ``False`` otherwise.
        """
        # actor + resource are python classes
        return self.query(
            Query(name="allow", args=[actor, action, resource]),
            debug=debug,
            single=True,
        ).success


def polar_class(_cls=None, *, from_polar=None):
    """Decorator to register a Python class with Polar. An alternative to ``register_python_class()``.

    :param str from_polar: Name of static class function to create a new class instance from ``fields``.
                            Defaults to class constructor.
    """

    def wrap(cls):
        Polar().register_python_class(cls, from_polar)
        return cls

    if _cls is None:
        return wrap

    return wrap(_cls)
