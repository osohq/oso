package com.osohq.oso;

import java.util.HashSet;
import java.util.stream.Collectors;

public class Enforcer {
  public Oso policy;
  private Object readAction = "read";

  public Enforcer(Oso policy) {
    this.policy = policy;
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
   * @param action the action the actor is attempting to peform
   * @param resource the resource being accessed
   * @param checkRead if set to `false`, do not query the policy for the `"read"` action, and always
   *     throw a ForbiddenException on any non-read authorization failure. Default is `true`.
   * @return boolean
   * @throws Exceptions.OsoException
   */
  public void authorize(Object actor, Object action, Object resource)
      throws Exceptions.OsoException {
    authorize(actor, action, resource, true);
  }

  public void authorize(Object actor, Object action, Object resource, boolean checkRead)
      throws Exceptions.OsoException {
    boolean authorized = policy.queryRuleOnce("allow", actor, action, resource);
    if (authorized) {
      return;
    }

    // Authorization failure. Determine whether to throw a NotFoundException or
    // a ForbiddenException.
    boolean isNotFound = false;
    if (action == readAction) {
      isNotFound = true;
    } else if (checkRead) {
      boolean canRead = policy.queryRuleOnce("allow", actor, readAction, resource);
      if (!canRead) {
        isNotFound = true;
      }
    }
    throw isNotFound ? new Exceptions.NotFoundException() : new Exceptions.ForbiddenException();
  }

  public void authorizeRequest(Object actor, Object request) throws Exceptions.OsoException {
    boolean authorized = policy.queryRuleOnce("allow_request", actor, request);
    if (!authorized) {
      throw new Exceptions.ForbiddenException();
    }
  }

  public void authorizeField(Object actor, Object action, Object resource, Object field)
      throws Exceptions.OsoException {
    boolean authorized = policy.queryRuleOnce("allow_field", actor, action, resource, field);
    if (!authorized) {
      throw new Exceptions.ForbiddenException();
    }
  }

  /**
   * Return the allowed actions for the given actor and resource, if any.
   *
   * <pre>{@code
   * Oso oso = new Oso();
   * o.loadStr("allow(\"guest\", \"get\", \"widget\");");
   * HashSet actions = o.authorizedActions("guest", "widget");
   * assert actions.contains("get");
   * }</pre>
   *
   * @param actor the actor performing the request
   * @param resource the resource being accessed
   * @return HashSet<Object>
   * @throws Exceptions.OsoException
   */
  public HashSet<Object> authorizedActions(Object actor, Object resource)
      throws Exceptions.OsoException {
    return authorizedActions(actor, resource, false);
  }

  /**
   * Return the allowed actions for the given actor and resource, if any. Explicitly allow or
   * disallow wildcard actions. If allowed, wildcard actions are represented as "*".
   *
   * <pre>{@code
   * Oso oso = new Oso();
   * o.loadStr("allow(_actor, _action, _resource);");
   * HashSet actions = o.authorizedActions("guest", "widget", true);
   * assert actions.contains("*");
   * HashSet actions = o.authorizedActions("guest", "widget", false);
   * // OsoException is thrown
   * }</pre>
   *
   * @param actor the actor performing the request
   * @param resource the resource being accessed
   * @param allowWildcard whether or not to allow wildcard actions
   * @return HashSet<Object>
   * @throws Exceptions.OsoException
   */
  public HashSet<Object> authorizedActions(Object actor, Object resource, boolean allowWildcard)
      throws Exceptions.OsoException {
    return policy.queryRule("allow", actor, new Variable("action"), resource).results().stream()
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

  /**
   * Return the allowed fields for the given actor and resource, if any.
   *
   * @param actor the actor performing the request
   * @param action the action being performed on the field
   * @param resource the resource on which the field lives
   * @return HashSet<Object>
   * @throws Exceptions.OsoException
   */
  public HashSet<Object> authorizedFields(Object actor, Object action, Object resource)
      throws Exceptions.OsoException {
    return authorizedFields(actor, action, resource, false);
  }

  /**
   * Return the allowed fields for the given actor and resource, if any. Explicitly allow or
   * disallow wildcard fields. If allowed, wildcard fields are represented as "*".
   *
   * @param actor the actor performing the request
   * @param action the action being performed on the field
   * @param resource the resource on which the field lives
   * @param allowWildcard whether or not to allow wildcard fields
   * @return HashSet<Object>
   * @throws Exceptions.OsoException
   */
  public HashSet<Object> authorizedFields(
      Object actor, Object action, Object resource, boolean allowWildcard)
      throws Exceptions.OsoException {
    return policy
        .queryRule("allow_field", actor, action, resource, new Variable("field"))
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
}
