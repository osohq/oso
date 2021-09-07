# frozen_string_literal: true

require 'set'
require_relative 'polar/polar'

module Oso
  # oso authorization API.
  class Oso < Polar::Polar
    # Create an Oso instance, which is used to configure and enforce an Oso
    # policy in an app.
    #
    # @param forbidden_error [Class] Optionally override the "forbidden" error
    #   class thrown by the `authorize*` methods. Defaults to
    #   {Oso::ForbiddenError}.
    # @param not_found_error [Class] Optionally override the "not found" error
    #   class thrown by {#authorize}. Defaults to {Oso::NotFoundError}.
    # @param read_action The action used by the {#authorize} method to
    #   determine whether an authorization failure should
    #   raise a {Oso::NotFoundError} or a {Oso::ForbiddenError}
    def initialize(not_found_error: NotFoundError, forbidden_error: ForbiddenError, read_action: 'read')
      super()
      @not_found_error = not_found_error
      @forbidden_error = forbidden_error
      @read_action = read_action
    end

    # Query the knowledge base to determine whether an actor is allowed to
    # perform an action upon a resource.
    #
    # @param actor [Object] Subject.
    # @param action [Object] Verb.
    # @param resource [Object] Object.
    # @return [Boolean] An access control decision.
    def allowed?(actor:, action:, resource:)
      query_rule_once('allow', actor, action, resource)
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
      return if query_rule_once('allow', actor, action, resource)

      if check_read && (action == @read_action || !query_rule_once('allow', actor, @read_action, resource))
        raise @not_found_error
      end

      raise @forbidden_error
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
      raise @forbidden_error unless query_rule_once('allow_request', actor, request)
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
      raise @forbidden_error unless query_rule_once('allow_field', actor, action, resource, field)
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
    #   return +Set["*"]+, if +false+, the method will raise an exception.
    # @return A set of the unique allowed actions.
    def authorized_actions(actor, resource, allow_wildcard: false) # rubocop:disable Metrics/MethodLength
      results = query_rule('allow', actor, Polar::Variable.new('action'), resource)
      actions = Set.new
      results.each do |result|
        action = result['action']
        if action.is_a?(Polar::Variable)
          return Set['*'] if allow_wildcard

          raise ::Oso::Error,
                'The result of authorized_actions() contained an '\
                '"unconstrained" action that could represent any '\
                'action, but allow_wildcard was set to False. To fix, '\
                'set allow_wildcard to True and compare with the "*" '\
                'string.'
        end
        actions.add(action)
      end
      actions
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
    #   method will return +Set["*"]+, if +false+, the method will raise an \
    #   exception.
    # @returns A set of the unique allowed fields.
    def authorized_fields(actor, action, resource, allow_wildcard: false) # rubocop:disable Metrics/MethodLength
      results = query_rule('allow_field', actor, action, resource, Polar::Variable.new('field'))
      fields = Set.new
      results.each do |result|
        field = result['field']
        if field.is_a?(Polar::Variable)
          return Set['*'] if allow_wildcard

          raise ::Oso::Error,
                'The result of authorized_fields() contained an '\
                '"unconstrained" field that could represent any '\
                'field, but allow_wildcard was set to False. To fix, '\
                'set allow_wildcard to True and compare with the "*" '\
                'string.'
        end
        fields.add(field)
      end
      fields
    end
  end
end
