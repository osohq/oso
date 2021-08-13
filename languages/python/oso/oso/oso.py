"""Core oso functionality"""

__version__ = "0.15.0"

import os
from polar import Polar, Variable, exceptions


class Oso(Polar):
    """The central object to manage application policy state, e.g.
    the policy data, and verify requests.

    >>> from oso import Oso
    >>> Oso()
    <oso.oso.Oso object at 0x...>

    """

    def __init__(self):
        """Create an oso object."""
        self._print_polar_log_message()
        super().__init__()

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

    def _print_polar_log_message(self):
        if os.environ.get("POLAR_LOG", None):
            print(
                "Polar tracing enabled. Get help with "
                + "traces from our engineering team: https://help.osohq.com/trace"
            )
