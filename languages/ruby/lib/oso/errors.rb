module Oso
  class Error < StandardError
  end

  class AuthorizationError < StandardError
  end

  class ForbiddenError < AuthorizationError
  end

  class NotFoundError < AuthorizationError
  end
end
