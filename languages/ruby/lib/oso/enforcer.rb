module Oso
  class Enforcer
    attr_accessor :policy

    def initialize(policy, get_error: nil, read_action: nil)
      @policy = policy
      @get_error = get_error
      get_error ||= self.method(:default_error)
      @read_action = read_action
    end

    def authorize(actor, action, resource)
    end

    def authorize_request(actor, request)
    end

    def authorize_field(actor, action, resource, field)
    end

    def authorized_actions(actor, resource, allow_wildcard: false)
    end

    def authorized_fields(actor, action, resource, allow_wildcard: false)
    end

    private

    def default_error(is_not_found)
      return NotFoundError if is_not_found
      return ForbiddenError
    end
  end
end
