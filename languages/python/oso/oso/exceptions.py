class AuthorizationError(Exception):
    pass


class NotFoundError(AuthorizationError):
    """
    Thrown by the ``authorize`` method of an ``Enforcer``. This error indicates
    that the actor is not only not allowed to perform the given action, but also
    is not allowed to ``"read"`` the given resource.

    Most of the time, your app should handle this error by returning a 404 HTTP
    error to the client.

    To control which action is used for the distinction between
    ``NotFoundError`` and ``ForbiddenError``, you can customize the
    ``read_action`` on your ``Enforcer``.
    """
    pass


class ForbiddenError(AuthorizationError):
    """
    Thrown by the Enforcer methods ``authorize``, ``authorize_field``, and
    ``authorize_request`` when the action is not allowed.

    Most of the time, your app should handle this error by returning a 403 HTTP
    error to the client.
    """
    pass
