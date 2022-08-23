"""Core oso functionality"""

__version__ = "0.26.2"

import os
from typing import List, Any, Set

from polar import Polar, Variable, exceptions
from .exceptions import NotFoundError, ForbiddenError


class Oso(Polar):
    """The central object to manage application policy state, e.g.
    the policy data, and verify requests.

    >>> from oso import Oso
    >>> Oso()
    <oso.oso.Oso object at 0x...>

    """

    def __init__(
        self,
        *,
        forbidden_error=ForbiddenError,
        not_found_error=NotFoundError,
        read_action="read"
    ):
        """
        Create an Oso object.

        :param forbidden_error:
            Optionally override the error class that is raised when an action is
            unauthorized.
        :param not_found_error:
            Optionally override the error class that is raised by the
            ``authorize`` method when an action is unauthorized AND the actor
            does not have permission to ``"read"`` the resource (and thus should
            not know it exists).
        :param read_action:
            The action used by the ``authorize`` method to determine whether an
            authorization failure should raise a ``NotFoundError`` or a
            ``ForbiddenError``.
        """
        self._print_polar_log_message()
        super().__init__()

        self.forbidden_error = forbidden_error
        self.not_found_error = not_found_error
        self.read_action = read_action

    def is_allowed(self, actor, action, resource) -> bool:
        """Evaluate whether ``actor`` is allowed to perform ``action`` on ``resource``.

        Uses allow rules in the Polar policy to determine whether a request is
        permitted. ``actor`` and ``resource`` should be classes that have been
        registered with Polar using the :py:func:`register_class` function.

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

    def get_allowed_actions(self, actor, resource, allow_wildcard=False) -> List[Any]:
        """Determine the actions ``actor`` is allowed to take on ``resource``.

        Deprecated. Use ``authorized_actions`` instead.
        """
        return list(self.authorized_actions(actor, resource, allow_wildcard))

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
        if self.query_rule_once("allow", actor, action, resource):
            return

        if check_read and (
            action == self.read_action
            or not self.query_rule_once("allow", actor, self.read_action, resource)
        ):
            raise self.not_found_error()
        raise self.forbidden_error()

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
            raise self.forbidden_error()

    def authorized_actions(self, actor, resource, allow_wildcard=False) -> Set[Any]:
        """Determine the actions ``actor`` is allowed to take on ``resource``.

        Collects all actions allowed by allow rules in the Polar policy for the
        given combination of actor and resource.

        Identical to ``Oso.get_allowed_actions``.

        :param actor: The actor for whom to collect allowed actions

        :param resource: The resource being accessed

        :param allow_wildcard: Flag to determine behavior if the policy
            contains an "unconstrained" action that could represent any action:
            ``allow(_actor, _action, _resource)``. If ``True``, the method will
            return ``["*"]``, if ``False`` (the default), the method will raise
            an exception.

        :type allow_wildcard: bool

        :return: A set containing all allowed actions.
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
                    return {"*"}
            actions.add(action)

        return actions

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
            raise self.forbidden_error()

    def authorized_fields(
        self, actor, action, resource, allow_wildcard=False
    ) -> Set[Any]:
        """Determine the fields of ``resource`` on which ``actor`` is allowed to
        perform  ``action``.

        Uses ``allow_field`` rules in the policy to find all allowed fields.

        :param actor: The actor for whom to collect allowed fields.
        :param action: The action being taken on the fields.
        :param resource: The resource being accessed.

        :param allow_wildcard: Flag to determine behavior if the policy \
            includes a wildcard field. E.g., a rule allowing any field: \
            ``allow_field(_actor, _action, _resource, _field)``. If ``True``, the \
            method will return ``["*"]``, if ``False``, the method will raise an \
            exception.

        :type allow_wildcard: bool

        :return: A set containing all allowed fields.
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
                    return {"*"}
            fields.add(field)

        return fields

    def authorized_query(self, actor, action, resource_cls):
        """Create a query for resources of type ``resource_cls``
        that ``actor`` is allowed to perform ``action`` on. The
        query is built by using the ``build_query`` and ``combine_query``
        functions registered for the ``resource_cls``.

        :param actor: The actor for whom to collect allowed resources.
        :param action: The action that user wants to perform.
        :param resource_cls: The type of the resources.

        :return: A query to fetch the resources,
        """

        return self.new_authorized_query(actor, action, resource_cls)

    def authorized_resources(self, actor, action, resource_cls) -> List[Any]:
        """Determine the resources of type ``resource_cls`` that ``actor``
        is allowed to perform ``action`` on.

        :param actor: The actor for whom to collect allowed resources.
        :param action: The action that user wants to perform.
        :param resource_cls: The type of the resources.

        :return: The requested resources.
        """
        query = self.authorized_query(actor, action, resource_cls)
        return self.host.adapter.execute_query(query)

    def set_data_filtering_adapter(self, adapter):
        """Set a global adapter for the new data filtering interface."""
        self.host.adapter = adapter

    def _print_polar_log_message(self):
        if os.environ.get("POLAR_LOG", "0") not in ("off", "0"):
            print(
                "Polar tracing enabled. Get help with "
                + "traces from our engineering team: https://help.osohq.com/trace"
            )
