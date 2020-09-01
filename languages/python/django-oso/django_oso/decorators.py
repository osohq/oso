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
    """Authorize view."""
    if view_func is not None:
        @functools.wraps(view_func)
        def wrap_view(request, *args, **kwargs):
            auth.authorize(request, actor=actor, action=action, resource=resource)
            return view_func(request, *args, **kwargs)

        return wrap_view

    return functools.partial(authorize, actor=actor, action=action, resource=resource)

def authorize_request(view_func=None, actor=None, action=None):
    """Authorize the view function, using the request as the resource."""
    if view_func is not None:
        @functools.wraps(view_func)
        def wrap_view(request, *args, **kwargs):
            auth.authorize(request, actor=actor, action=action, resource=request)
            return view_func(request, *args, **kwargs)

        return wrap_view

    return functools.partial(authorize_request, actor=actor, action=action)
