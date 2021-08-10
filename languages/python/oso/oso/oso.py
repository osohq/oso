"""Core oso functionality"""

__version__ = "0.14.1"

import os
from polar import Polar, Variable, exceptions
from .exceptions import NotFoundError, ForbiddenError


class Oso(Polar):
    """The central object to manage application policy state, e.g.
    the policy data, and verify requests.

    >>> from oso import Oso
    >>> Oso()
    <oso.oso.Oso object at 0x...>

    """

    read_action = "read"

    def __init__(self, *, get_error=None, read_action=None):
        """Create an oso object."""
        self._print_polar_log_message()
        if get_error is None:
            self._get_error = self._default_get_error
        else:
            self._get_error = get_error
        if read_action is not None:
            self.read_action = read_action
        super().__init__()

    def _default_get_error(self, is_not_found, actor, action, resource):
        err_class = NotFoundError if is_not_found else ForbiddenError
        return err_class(actor, action, resource)

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

    def get_allowed_actions(self, actor, resource, allow_wildcard=False) -> list:
        """Determine the actions ``actor`` is allowed to take on ``resource``.

        Collects all actions allowed by allow rules in the Polar policy for the
        given combination of actor and resource.

        :param actor: The actor for whom to collect allowed actions

        :param resource: The resource being accessed

        :param allow_wildcard: Flag to determine behavior if the policy \
        includes a wildcard action. E.g., a rule allowing any action: \
        ``allow(_actor, _action, _resource)``. If ``True``, the method will \
        return ``["*"]``, if ``False``, the method will raise an exception.

        :type allow_wildcard: bool

        :return: A list of the unique allowed actions.
        """
        results = self.query_rule("allow", actor, Variable("action"), resource)
        actions = set()
        for result in results:
            action = result.get("bindings").get("action")
            if isinstance(action, Variable):
                if not allow_wildcard:
                    raise exceptions.OsoError(
                        """The result of get_allowed_actions() contained an
                        "unconstrained" action that could represent any
                        action, but allow_wildcard was set to False. To fix,
                        set allow_wildcard to True and compare with the "*"
                        string."""
                    )
                else:
                    return ["*"]
            actions.add(action)

        return list(actions)

    def authorize(self, actor, action, resource, *, check_read=True):
        if not self.query_rule_once("allow", actor, action, resource):
            is_not_found = False
            if (action == self.read_action):
                is_not_found = True
            elif check_read and not self.query_rule_once("allow", actor, self.read_action, resource):
                is_not_found = True
            raise self._get_error(is_not_found, actor, action, resource)

    def _print_polar_log_message(self):
        if os.environ.get("POLAR_LOG", None):
            print(
                "Polar tracing enabled. Get help with "
                + "traces from our engineering team: https://help.osohq.com/trace"
            )
