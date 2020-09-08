import functools

from . import auth


def skip_authorization(view_func):
    """View-decorator that marks a view as not requiring authorization.

    Use in combination with :py:func:`django_oso.middleware.RequireAuthorization`.
    Some views will not require authorization.  This decorator marks those views
    so that the middleware can skip the check.
    """

    @functools.wraps(view_func)
    def wrap_view(request, *args, **kwargs):
        auth.skip_authorization(request)
        return view_func(request, *args, **kwargs)

    return wrap_view


def authorize(view_func=None, resource=None, actor=None, action=None):
    """Authorize view for ``resource``, ``actor``, and ``action``.

    All three parameters must be constant for this decorator to be used. If
    actor or action are omitted, the defaults from
    :py:func:`django_oso.auth.authorize`. are used.
    """
    if view_func is not None:

        @functools.wraps(view_func)
        def wrap_view(request, *args, **kwargs):
            auth.authorize(request, actor=actor, action=action, resource=resource)
            return view_func(request, *args, **kwargs)

        return wrap_view

    return functools.partial(authorize, actor=actor, action=action, resource=resource)


def authorize_request(view_func=None, actor=None, action=None):
    """Authorize the view function, using the request as the resource.

    This performs route authorization, similarly to
    :py:class:`~django_oso.middleware.RouteAuthorization`, but on a single view.
    """
    if view_func is not None:

        @functools.wraps(view_func)
        def wrap_view(request, *args, **kwargs):
            auth.authorize(request, actor=actor, action=action, resource=request)
            return view_func(request, *args, **kwargs)

        return wrap_view

    return functools.partial(authorize_request, actor=actor, action=action)
