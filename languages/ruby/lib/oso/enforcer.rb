# frozen_string_literal: true

require 'set'

module Oso
  class Enforcer
    attr_accessor :policy

    def initialize(policy, get_error: method(:default_error), read_action: 'read')
      @policy = policy
      @get_error = get_error
      @read_action = read_action
    end

    def authorize(actor, action, resource, check_read: true)
      return if policy.query_rule_once('allow', actor, action, resource)

      is_not_found = false
      if action == @read_action
        is_not_found = true
      elsif check_read && !policy.query_rule_once('allow', actor, @read_action, resource)
        is_not_found = true
      end
      raise @get_error.call(is_not_found)
    end

    def authorize_request(actor, request)
      raise @get_error.call(false) unless policy.query_rule_once('allow_request', actor, request)
    end

    def authorize_field(actor, action, resource, field)
      raise @get_error.call(false) unless policy.query_rule_once('allow_field', actor, action, resource, field)
    end

    def authorized_actions(actor, resource, allow_wildcard: false)
      results = policy.query_rule("allow", actor, Polar::Variable.new("action"), resource)
      actions = Set.new
      results.each do |result|
        action = result["action"]
        if action.is_a?(Polar::Variable)
          return ["*"] if allow_wildcard

          raise ::Oso::Error.new(%|
            The result of authorized_actions() contained an
            "unconstrained" action that could represent any
            action, but allow_wildcard was set to False. To fix,
            set allow_wildcard to True and compare with the "*"
            string.
          |)
        end
        actions.add(action)
      end
      actions.to_a
    end

    def authorized_fields(actor, action, resource, allow_wildcard: false)
      results = policy.query_rule("allow_field", actor, action, resource, Polar::Variable.new("field"))
      fields = Set.new
      results.each do |result|
        field = result["field"]
        if field.is_a?(Polar::Variable)
          return ["*"] if allow_wildcard

          raise ::Oso::Error.new(%|
            The result of authorized_fields() contained an
            "unconstrained" field that could represent any
            field, but allow_wildcard was set to False. To fix,
            set allow_wildcard to True and compare with the "*"
            string.
          |)
        end
        fields.add(field)
      end
      fields.to_a
    end

    private

    def default_error(is_not_found)
      return NotFoundError if is_not_found
      return ForbiddenError
    end
  end
end
