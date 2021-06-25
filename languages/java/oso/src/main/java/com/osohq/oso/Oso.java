package com.osohq.oso;

import java.io.IOException;
import java.util.HashSet;
import java.util.stream.Collectors;

public class Oso extends Polar {
  public Oso() {
    super();
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
   * @return boolean
   * @throws Exceptions.OsoException
   */
  public boolean isAllowed(Object actor, Object action, Object resource)
      throws Exceptions.OsoException {
    return queryRule("allow", actor, action, resource).hasMoreElements();
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
    return queryRule("allow", actor, new Variable("action"), resource).results().stream()
        .map(action -> action.get("action"))
        .collect(Collectors.toCollection(HashSet::new));
  }

  /**
   * Return the allowed actions for the given actor and resource, if any. Explicitly allow or
   * disallow wildcard actions. If allowed, wildcard actions are represented as "*".
   *
   * <pre>{@code
   * Oso oso = new Oso();
   * o.loadStr("allow(_actor, _action, _resource);");
   * HashSet actions = o.getAllowedActions("guest", "widget", true);
   * assert actions.contains("*");
   * HashSet actions = o.getAllowedActions("guest", "widget", false);
   * // OsoException is thrown
   * }</pre>
   *
   * @param actor the actor performing the request
   * @param resource the resource being accessed
   * @param allowWildcard whether or not to allow wildcard actions
   * @return HashSet<Object>
   * @throws Exceptions.OsoException
   */
  public HashSet<Object> getAllowedActions(Object actor, Object resource, boolean allowWildcard)
      throws Exceptions.OsoException {
    return queryRule("allow", actor, new Variable("action"), resource).results().stream()
        .map(
            action -> {
              if (action.get("action") instanceof Variable) {
                if (!allowWildcard) {
                  throw new Exceptions.OsoException(
                      "\"The result of getAllowedActions contained an \"unconstrained\" action that"
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

  public static void main(String[] args) throws Exceptions.OsoException, IOException {
    new Oso().repl(args);
  }
}
