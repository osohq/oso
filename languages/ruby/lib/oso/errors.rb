# frozen_string_literal: true

module Oso
  class Error < StandardError
  end

  class AuthorizationError < StandardError
  end

  class ForbiddenError < AuthorizationError
    def initialize
      super(
        'Oso ForbiddenError -- The requested action was not allowed for the ' \
        'given resource. You should handle this error by returning a 403 error ' \
        'to the client.'
      )
    end
  end

  class NotFoundError < AuthorizationError
    def initialize
      super(
        'Oso NotFoundError -- The current user does not have permission to read ' \
        'the given resource. You should handle this error by returning a 404 ' \
        'error to the client.'
      )
    end
  end
end
