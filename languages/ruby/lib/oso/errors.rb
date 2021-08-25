# frozen_string_literal: true

module Oso
  class Error < StandardError
  end

  class AuthorizationError < StandardError
  end

  # Thrown by the Enforcer methods +authorize+, +authorize_field+, and
  # +authorize_request+ when the action is not allowed.
  #
  # Most of the time, your app should handle this error by returning a 403 HTTP
  # error to the client.
  class ForbiddenError < AuthorizationError
    def initialize
      super(
        'Oso ForbiddenError -- The requested action was not allowed for the ' \
        'given resource. You should handle this error by returning a 403 error ' \
        'to the client.'
      )
    end
  end

  # Thrown by the +authorize+ method of an +Enforcer+. This error indicates
  # that the actor is not only not allowed to perform the given action, but also
  # is not allowed to +"read"+ the given resource.
  #
  # Most of the time, your app should handle this error by returning a 404 HTTP
  # error to the client.
  #
  # To control which action is used for the distinction between
  # +NotFoundError+ and +ForbiddenError+, you can customize the
  # +read_action+ on your +Enforcer+.
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
