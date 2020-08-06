import functools

from flask import g, current_app, _app_ctx_stack, request, Request

def authorize(func=None, resource=None, actor=None, action=None):
    if func is not None:
        @functools.wraps(func)
        def wrap(*args, **kwargs):
            oso = _app_ctx_stack.top.oso_flask_oso

            oso.authorize(actor=actor, action=action, resource=resource)
            return func(*args, **kwargs)

        return wrap

    return functools.partial(authorize, actor=actor, action=action, resource=resource)

def skip_authorization(func=None, reason=None):
    if func is not None:
        @functools.wraps(func)
        def wrap(*args, **kwargs):
            oso = _app_ctx_stack.top.oso_flask_oso
            oso.skip_authorization(reason=reason)
            return func(*args, **kwargs)

        return wrap

    return functools.partial(skip_authorization, reason=reason)
