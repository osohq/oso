class AuthorizationError(Exception):
    def __init__(self, actor, action, resource):
        self.actor = actor
        self.action = action
        self.resource = resource


class NotFoundError(AuthorizationError):
    pass


class ForbiddenError(AuthorizationError):
    pass
