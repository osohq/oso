package com.osohq.oso;

import java.io.IOException;
import java.util.HashSet;
import java.util.stream.Collectors;

public class Oso extends Polar {
  private Object readAction = "read";

  public Oso() {
    super();
  }

  public static void main(String[] args) throws Exceptions.OsoException, IOException {
    new Oso().repl(args);
  }

  /**
   * Override the "read" action, which is used to differentiate between a `NotFoundException` and a
   * `ForbiddenException` on authorization failures.
   */
  public void setReadAction(Object readAction) {
    this.readAction = readAction;
  }

  /**
   * Submit an `allow` query to the Polar knowledge base.
   *
   * <pre>{@code
   * Oso oso = new Oso();
   * o.loadStr("allow(\"guest\", \"get\", \"widget\");");
   * assert o.isAllowed("guest", "get", "widget");
   * }</pre>
   *
   * @param actor the actor performing the request
   * @param action the action the actor is attempting to perform
   * @param resource the resource being accessed
   * @return boolean
   * @throws Exceptions.OsoException
   */
  public boolean isAllowed(Object actor, Object action, Object resource)
      throws Exceptions.OsoException {
    return queryRuleOnce("allow", actor, action, resource);
  }

  /**
   * Return the allowed actions for the given actor and resource, if any.
   *
   * <pre>{@code
   * Oso oso = new Oso();
   * o.loadStr("allow(\"guest\", \"get\", \"widget\");");
   * HashSet actions = o.getAllowedActions("guest", "widget");
   * assert actions.contains("get");
   * }</pre>
   *
   * @param actor the actor performing the request
   * @param resource the resource being accessed
   * @return HashSet<Object>
   * @throws Exceptions.OsoException
   */
  public HashSet<Object> getAllowedActions(Object actor, Object resource)
      throws Exceptions.OsoException {
    return getAllowedActions(actor, resource, false);
  }

  /**
   * Return the allowed actions for the given actor and resource, if any. Explicitly allow or
   * disallow wildcard actions. If allowed, wildcard actions are represented as "*".
   *
   * @deprecated Use `authorizedActions` instead.
   * @param actor the actor performing the request
   * @param resource the resource being accessed
   * @param allowWildcard whether or not to allow wildcard actions
   * @return HashSet<Object>
   * @throws Exceptions.OsoException
   */
  public HashSet<Object> getAllowedActions(Object actor, Object resource, boolean allowWildcard)
      throws Exceptions.OsoException {
    return authorizedActions(actor, resource, allowWildcard);
  }

  /**
   * Ensure that `actor` is allowed to perform `action` on `resource`.
   *
   * <p>If the action is permitted with an `allow` rule in the policy, then this method returns
   * `None`. If the action is not permitted by the policy, this method will raise an error.
   *
   * <p>The error raised by this method depends on whether the actor can perform the `"read"` action
   * on the resource. If they cannot read the resource, then a `NotFoundException` is raised.
   * Otherwise, a `ForbiddenException` is raised.
   *
   * @param actor The actor performing the request.
   * @param action The action the actor is attempting to perform.
   * @param resource The resource being accessed.
   * @param checkRead If set to `false`, a `ForbiddenException` is always thrown on authorization
   *     failures, regardless of whether the actor can read the resource. Default is `true`.
   * @throws Exceptions.OsoException
   */
  public void authorize(Object actor, Object action, Object resource, boolean checkRead)
      throws Exceptions.OsoException {
    boolean authorized = queryRuleOnce("allow", actor, action, resource);
    if (authorized) {
      return;
    }

    // Authorization failure. Determine whether to throw a NotFoundException or
    // a ForbiddenException.
    if (checkRead) {
      if (action == readAction || !queryRuleOnce("allow", actor, readAction, resource)) {
        throw new Exceptions.NotFoundException();
      }
    }
    throw new Exceptions.ForbiddenException();
  }

  public void authorize(Object actor, Object action, Object resource)
      throws Exceptions.OsoException {
    authorize(actor, action, resource, true);
  }

  /**
   * Ensure that `actor` is allowed to send `request` to the server.
   *
   * <p>Checks the `allow_request` rule of a policy.
   *
   * <p>If the request is permitted with an `allow_request` rule in the policy, then this method
   * returns nothing. Otherwise, this method raises a `ForbiddenException`.
   *
   * @param actor The actor performing the request.
   * @param request An object representing the request that was sent by the actor.
   * @throws Exceptions.OsoException
   */
  public void authorizeRequest(Object actor, Object request) throws Exceptions.OsoException {
    boolean authorized = queryRuleOnce("allow_request", actor, request);
    if (!authorized) {
      throw new Exceptions.ForbiddenException();
    }
  }

  /**
   * Ensure that `actor` is allowed to perform `action` on a given `resource`'s `field`.
   *
   * <p>If the action is permitted by an `allow_field` rule in the policy, then this method returns
   * nothing. If the action is not permitted by the policy, this method will raise a
   * `ForbiddenException`.
   *
   * @param actor The actor performing the request.
   * @param action The action the actor is attempting to perform on the field.
   * @param resource The resource being accessed.
   * @param field The name of the field being accessed.
   * @throws Exceptions.OsoException
   */
  public void authorizeField(Object actor, Object action, Object resource, Object field)
      throws Exceptions.OsoException {
    boolean authorized = queryRuleOnce("allow_field", actor, action, resource, field);
    if (!authorized) {
      throw new Exceptions.ForbiddenException();
    }
  }

  /**
   * Determine the actions `actor` is allowed to take on `resource`.
   *
   * <p>Collects all actions allowed by allow rules in the Polar policy for the given combination of
   * actor and resource.
   *
   * @param actor The actor for whom to collect allowed actions
   * @param resource The resource being accessed
   * @param allowWildcard Flag to determine behavior if the policy includes a wildcard action. E.g.,
   *     a rule allowing any action: `allow(_actor, _action, _resource)`. If `true`, the method will
   *     return `["*"]`, if `false`, the method will raise an exception.
   * @return HashSet<Object> A list of the unique allowed actions.
   * @throws Exceptions.OsoException
   */
  public HashSet<Object> authorizedActions(Object actor, Object resource, boolean allowWildcard)
      throws Exceptions.OsoException {
    return queryRule("allow", actor, new Variable("action"), resource).results().stream()
        .map(
            action -> {
              if (action.get("action") instanceof Variable) {
                if (!allowWildcard) {
                  throw new Exceptions.OsoException(
                      "\"The result of authorizedActions contained an \"unconstrained\" action that"
                          + " could represent any\n"
                          + " action, but allowWildcard was set to false. To fix,\n"
                          + " set allowWildcard to true and compare with the \"*\"\n"
                          + " string.\"");
                } else {
                  return "*";
                }
              } else {
                return action.get("action");
              }
            })
        .collect(Collectors.toCollection(HashSet::new));
  }

  public HashSet<Object> authorizedActions(Object actor, Object resource)
      throws Exceptions.OsoException {
    return authorizedActions(actor, resource, false);
  }

  /**
   * Determine the fields of `resource` on which `actor` is allowed to perform `action`.
   *
   * <p>Uses `allow_field` rules in the policy to find all allowed fields.
   *
   * @param actor The actor for whom to collect allowed fields.
   * @param action The action being taken on the field.
   * @param resource The resource being accessed.
   * @param allowWildcard Flag to determine behavior if the policy \ includes a wildcard field.
   *     E.g., a rule allowing any field: \ `allow_field(_actor, _action, _resource, _field)`. If
   *     `true`, the \ method will return `["*"]`, if `false`, the method will raise an \ exception.
   * @return HashSet<Object> A set of the unique allowed fields.
   * @throws Exceptions.OsoException
   */
  public HashSet<Object> authorizedFields(
      Object actor, Object action, Object resource, boolean allowWildcard)
      throws Exceptions.OsoException {
    return queryRule("allow_field", actor, action, resource, new Variable("field"))
        .results()
        .stream()
        .map(
            field -> {
              if (field.get("field") instanceof Variable) {
                if (!allowWildcard) {
                  throw new Exceptions.OsoException(
                      "\"The result of authorizedFields contained an \"unconstrained\" field that"
                          + " could represent any\n"
                          + " field, but allowWildcard was set to false. To fix,\n"
                          + " set allowWildcard to true and compare with the \"*\"\n"
                          + " string.\"");
                } else {
                  return "*";
                }
              } else {
                return field.get("field");
              }
            })
        .collect(Collectors.toCollection(HashSet::new));
  }

  public HashSet<Object> authorizedFields(Object actor, Object action, Object resource)
      throws Exceptions.OsoException {
    return authorizedFields(actor, action, resource, false);
  }
}
