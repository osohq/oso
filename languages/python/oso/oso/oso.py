"""Core oso functionality"""

__version__ = "0.20.0-beta"

import os
from typing import List, Any

from polar import Polar, Variable, exceptions
from .exceptions import NotFoundError, ForbiddenError


class Oso(Polar):
    """The central object to manage application policy state, e.g.
    the policy data, and verify requests.

    >>> from oso import Oso
    >>> Oso()
    <oso.oso.Oso object at 0x...>

    """

    def __init__(self, *, get_error=None, read_action="read"):
        """
        Create an oso object.

        :param get_error: Optionally override the method used to build errors
                          raised by the ``authorize*`` methods. Should be a
                          callable that takes one argument ``is_not_found`` and
                          returns an exception.
        :param read_action: The action used by the ``authorize`` method to
                            determine whether an authorization failure should
                            raise a ``NotFoundError`` or a ``ForbiddenError``
        """
        self._print_polar_log_message()
        super().__init__()

        if get_error is None:
            self._get_error = self._default_get_error
        else:
            self._get_error = get_error
        self.read_action = read_action

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

        Deprecated. Use ``authorized_actions`` instead.
        """
        return self.authorized_actions(actor, resource, allow_wildcard)

    def authorize(self, actor, action, resource, *, check_read=True):
        """Ensure that ``actor`` is allowed to perform ``action`` on
        ``resource``.

        If the action is permitted with an ``allow`` rule in the policy, then
        this method returns ``None``. If the action is not permitted by the
        policy, this method will raise an error.

        The error raised by this method depends on whether the actor can perform
        the ``"read"`` action on the resource. If they cannot read the resource,
        then a ``NotFound`` error is raised. Otherwise, a ``ForbiddenError`` is
        raised.

        :param actor: The actor performing the request.
        :param action: The action the actor is attempting to perform.
        :param resource: The resource being accessed.

        :param check_read: If set to ``False``, a ``ForbiddenError`` is always
            thrown on authorization failures, regardless of whether the actor can
            read the resource. Default is ``True``.
        :type check_read: bool

        """
        if not self.query_rule_once("allow", actor, action, resource):
            is_not_found = False
            if action == self.read_action:
                is_not_found = True
            elif check_read and not self.query_rule_once(
                "allow", actor, self.read_action, resource
            ):
                is_not_found = True
            raise self._get_error(is_not_found)

    def authorize_request(self, actor, request):
        """Ensure that ``actor`` is allowed to send ``request`` to the server.

        Checks the ``allow_request`` rule of a policy.

        If the request is permitted with an ``allow_request`` rule in the
        policy, then this method returns ``None``. Otherwise, this method raises
        a ``ForbiddenError``.

        :param actor: The actor performing the request.
        :param request: An object representing the request that was sent by the
            actor.
        """
        if not self.query_rule_once("allow_request", actor, request):
            raise self._get_error(False)

    def authorized_actions(self, actor, resource, allow_wildcard=False) -> List[Any]:
        """Determine the actions ``actor`` is allowed to take on ``resource``.

        Collects all actions allowed by allow rules in the Polar policy for the
        given combination of actor and resource.

        Identical to ``Oso.get_allowed_actions``.

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
                        """The result of authorized_actions() contained an
                        "unconstrained" action that could represent any
                        action, but allow_wildcard was set to False. To fix,
                        set allow_wildcard to True and compare with the "*"
                        string."""
                    )
                else:
                    return ["*"]
            actions.add(action)

        return list(actions)

    def authorize_field(self, actor, action, resource, field):
        """Ensure that ``actor`` is allowed to perform ``action`` on a given
        ``resource``'s ``field``.

        If the action is permitted by an ``allow_field`` rule in the policy,
        then this method returns ``None``. If the action is not permitted by the
        policy, this method will raise a ``ForbiddenError``.

        :param actor: The actor performing the request.
        :param action: The action the actor is attempting to perform on the
            field.
        :param resource: The resource being accessed.
        :param field: The name of the field being accessed.
        """
        if not self.query_rule_once("allow_field", actor, action, resource, field):
            raise self._get_error(False)

    def authorized_fields(
        self, actor, action, resource, allow_wildcard=False
    ) -> List[Any]:
        """Determine the fields of ``resource`` on which ``actor`` is allowed to
        perform  ``action``.

        Uses ``allow_field`` rules in the policy to find all allowed fields.

        :param actor: The actor for whom to collect allowed fields.
        :param action: The action being taken on the field.
        :param resource: The resource being accessed.

        :param allow_wildcard: Flag to determine behavior if the policy \
            includes a wildcard field. E.g., a rule allowing any field: \
            ``allow_field(_actor, _action, _resource, _field)``. If ``True``, the \
            method will return ``["*"]``, if ``False``, the method will raise an \
            exception.

        :type allow_wildcard: bool

        :return: A list of the unique allowed fields.
        """
        results = self.query_rule(
            "allow_field", actor, action, resource, Variable("field")
        )
        fields = set()
        for result in results:
            field = result.get("bindings").get("field")
            if isinstance(field, Variable):
                if not allow_wildcard:
                    raise exceptions.OsoError(
                        """The result of authorized_fields() contained an
                        "unconstrained" field that could represent any
                        field, but allow_wildcard was set to False. To fix,
                        set allow_wildcard to True and compare with the "*"
                        string."""
                    )
                else:
                    return ["*"]
            fields.add(field)

        return list(fields)

    def _print_polar_log_message(self):
        if os.environ.get("POLAR_LOG", None):
            print(
                "Polar tracing enabled. Get help with "
                + "traces from our engineering team: https://help.osohq.com/trace"
            )

    def _default_get_error(self, is_not_found):
        return NotFoundError() if is_not_found else ForbiddenError()


Policy = Oso
