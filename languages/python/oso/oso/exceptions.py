class AuthorizationError(Exception):
    pass

class NotFoundError(AuthorizationError):
    pass


class ForbiddenError(AuthorizationError):
    pass
