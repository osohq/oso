module Oso
  class Enforcer
    attr_accessor :policy

    def initialize(policy, get_error: method(:default_error), read_action: 'read')
      @policy = policy
      @get_error = get_error
      @read_action = read_action
    end

    def authorize(actor, action, resource, check_read: true)
      if !policy.query_rule_once('allow', actor, action, resource)
        is_not_found = false
        if action == @read_action
          is_not_found = true
        elsif check_read && !policy.query_rule_once('allow', actor, @read_action, resource)
          is_not_found = true
        end
        raise @get_error.call(is_not_found)
      end
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
