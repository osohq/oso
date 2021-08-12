"""Core oso functionality"""

__version__ = "0.14.1"

import os
from polar import Polar, Variable, exceptions
from .exceptions import NotFoundError, ForbiddenError


class Oso:
    """The central object to manage application policy state, e.g.
    the policy data, and verify requests.

    >>> from oso import Oso
    >>> Oso()
    <oso.oso.Oso object at 0x...>

    :param policy: The policy
    """

    def __init__(self, policy=None, *, get_error=None, read_action="read"):
        """Create an oso object."""
        self._print_polar_log_message()
        if get_error is None:
            self._get_error = self._default_get_error
        else:
            self._get_error = get_error
        self.read_action = read_action

        if policy is None:
            policy = Polar()

        self.policy = policy

        # Note: these are for backwards-compatibility. They allow `Oso`
        # instances to be treated like `Polar` instances.
        self.query_rule = self.policy.query_rule
        self.query = self.policy.query
        self.load_str = self.policy.load_str
        self.load_file = self.policy.load_file
        self.register_class = self.policy.register_class
        self.register_constant = self.policy.register_constant
        self.enable_roles = self.policy.enable_roles
        self.clear_rules = self.policy.clear_rules
        self.repl = self.policy.repl

        super().__init__()

    def __del__(self):
        del self.policy

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
        """Ensure that ``actor`` is allowed to perform ``action`` on
        ``resource``.

        If the action is permitted with an ``allow`` rule in the policy, then
        this method returns ``None``. If the action is not permitted by the
        policy, this method will raise an error.

        The error raised by this method depends on whether the user can perform
        the ``"read"`` action on the resource. If they cannot read the resource,
        then a ``NotFound`` error is raised. Otherwise, a ``ForbiddenError`` is
        raised.

        :param actor: The actor performing the request.
        :param action: The action the actor is attempting to perform.
        :param resource: The resource being accessed.

        :param check_read: If set to ``False``, a ``ForbiddenError`` is always
        thrown on authorization failures, regardless of whether the user can
        read the resource. Default is ``True``.
        :type check_read: bool

        """
        if not self.policy.query_rule_once("allow", actor, action, resource):
            is_not_found = False
            if action == self.read_action:
                is_not_found = True
            elif check_read and not self.policy.query_rule_once(
                "allow", actor, self.read_action, resource
            ):
                is_not_found = True
            raise self._get_error(is_not_found, actor, action, resource)

    def authorize_request(self, actor, request):
        """Ensure that ``actor`` is allowed to send ``request`` to the server.

        If there is a matching ``allow_request(user, request)`` rule in the
        policy, then this method returns ``None``. Otherwise, this method raises
        a ``ForbiddenError``.

        :param actor: The actor performing the request.
        :param request: The request that was sent by the user. Can be a string
        path, or an object with information about the request.
        """
        if not self.policy.query_rule_once("allow_request", actor, request):
            raise self._get_error(False, actor, "request", request)

    def _print_polar_log_message(self):
        if os.environ.get("POLAR_LOG", None):
            print(
                "Polar tracing enabled. Get help with "
                + "traces from our engineering team: https://help.osohq.com/trace"
            )
