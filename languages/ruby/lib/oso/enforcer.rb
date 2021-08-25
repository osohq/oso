# frozen_string_literal: true

require 'set'

module Oso
  # NOTE: This is a preview feature.
  #
  # Exposes high-level enforcement APIs which can be used by apps to perform
  # resource-, request-, and query-level authorization.
  class Enforcer
    attr_accessor :policy

    # Create an Enforcer, which is used to enforce an Oso policy in an app.
    #
    # @param policy [Oso::Oso] The `Policy` instance to enforce.
    # @param get_error [Proc] Optionally override the method used to build errors
    #   raised by the  {#authorize} and {#authorize_request}
    #   methods. Should be a callable that takes one argument
    #   +is_not_found+ and returns an exception.
    # @param read_action The action used by the {#authorize} method to
    #   determine whether an authorization failure should
    #   raise a {Oso::NotFoundError} or a {Oso::ForbiddenError}
    def initialize(policy, get_error: method(:default_error), read_action: 'read')
      @policy = policy
      @get_error = get_error
      @read_action = read_action
    end

    # Ensure that +actor+ is allowed to perform +action+ on
    # +resource+.
    #
    # If the action is permitted with an +allow+ rule in the policy, then
    # this method returns +None+. If the action is not permitted by the
    # policy, this method will raise an error.
    #
    # The error raised by this method depends on whether the actor can perform
    # the +"read"+ action on the resource. If they cannot read the resource,
    # then a {Oso::NotFoundError} error is raised. Otherwise, a
    # {Oso::ForbiddenError} is raised.
    #
    # @param actor The actor performing the request.
    # @param action The action the actor is attempting to perform.
    # @param resource The resource being accessed.
    # @param check_read [Boolean] If set to +false+, a {Oso::ForbiddenError} is
    #   always thrown on authorization failures, regardless of whether the actor
    #   can read the resource. Default is +true+.
    #
    # @raise [Oso::ForbiddenError] Raised if the actor does not have permission
    #   to perform this action on this resource, but _does_ have +"read"+
    #   permission on the resource.
    # @raise [Oso::NotFoundError] Raised if the actor does not have permission
    #   to perform this action on this resource and additionally does not have
    #   permission to +"read"+ the resource.
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

    # Ensure that +actor+ is allowed to send +request+ to the server.
    #
    # Checks the +allow_request+ rule of a policy.
    #
    # If the request is permitted with an +allow_request+ rule in the
    # policy, then this method returns nothing. Otherwise, this method raises
    # a {Oso::ForbiddenError}.
    #
    # @param actor The actor performing the request.
    # @param request An object representing the request that was sent by the
    #   actor.
    #
    # @raise [Oso::ForbiddenError] Raised if the actor does not have permission
    #   to send the request.
    def authorize_request(actor, request)
      raise @get_error.call(false) unless policy.query_rule_once('allow_request', actor, request)
    end

    # Ensure that +actor+ is allowed to perform +action+ on a given
    # +resource+'s +field+.
    #
    # If the action is permitted by an +allow_field+ rule in the policy,
    # then this method returns nothing. If the action is not permitted by the
    # policy, this method will raise a {Oso::ForbiddenError}.
    #
    # @param actor The actor performing the request.
    # @param action The action the actor is attempting to perform on the
    #   field.
    # @param resource The resource being accessed.
    # @param field The name of the field being accessed.
    #
    # @raise [Oso::ForbiddenError] Raised if the actor does not have permission
    #   to access this field.
    def authorize_field(actor, action, resource, field)
      raise @get_error.call(false) unless policy.query_rule_once('allow_field', actor, action, resource, field)
    end

    # Determine the actions +actor+ is allowed to take on +resource+.
    #
    # Collects all actions allowed by allow rules in the Polar policy for the
    # given combination of actor and resource.
    #
    # @param actor The actor for whom to collect allowed actions
    # @param resource The resource being accessed
    # @param allow_wildcard Flag to determine behavior if the policy
    #   includes a wildcard action. E.g., a rule allowing any action:
    #   +allow(_actor, _action, _resource)+. If +true+, the method will
    #   return +["*"]+, if +false+, the method will raise an exception.
    # @return A list of the unique allowed actions.
    def authorized_actions(actor, resource, allow_wildcard: false) # rubocop:disable Metrics/MethodLength
      results = policy.query_rule('allow', actor, Polar::Variable.new('action'), resource)
      actions = Set.new
      results.each do |result|
        action = result['action']
        if action.is_a?(Polar::Variable)
          return ['*'] if allow_wildcard

          raise ::Oso::Error,
                'The result of authorized_actions() contained an '\
                '"unconstrained" action that could represent any '\
                'action, but allow_wildcard was set to False. To fix, '\
                'set allow_wildcard to True and compare with the "*" '\
                'string.'
        end
        actions.add(action)
      end
      actions.to_a
    end

    # Determine the fields of +resource+ on which +actor+ is allowed to
    # perform  +action+.
    #
    # Uses +allow_field+ rules in the policy to find all allowed fields.
    #
    # @param actor The actor for whom to collect allowed fields.
    # @param action The action being taken on the field.
    # @param resource The resource being accessed.
    # @param allow_wildcard Flag to determine behavior if the policy \
    #   includes a wildcard field. E.g., a rule allowing any field: \
    #   +allow_field(_actor, _action, _resource, _field)+. If +true+, the \
    #   method will return +["*"]+, if +false+, the method will raise an \
    #   exception.
    # @returns A list of the unique allowed fields.
    def authorized_fields(actor, action, resource, allow_wildcard: false) # rubocop:disable Metrics/MethodLength
      results = policy.query_rule('allow_field', actor, action, resource, Polar::Variable.new('field'))
      fields = Set.new
      results.each do |result|
        field = result['field']
        if field.is_a?(Polar::Variable)
          return ['*'] if allow_wildcard

          raise ::Oso::Error,
                'The result of authorized_fields() contained an '\
                '"unconstrained" field that could represent any '\
                'field, but allow_wildcard was set to False. To fix, '\
                'set allow_wildcard to True and compare with the "*" '\
                'string.'
        end
        fields.add(field)
      end
      fields.to_a
    end

    private

    def default_error(is_not_found)
      is_not_found ? NotFoundError : ForbiddenError
    end
  end
end
