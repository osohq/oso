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
   * HashSet actions = o.getAllowedActions("guest", "get", "widget");
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

    Query q = queryRule("allow", actor, new Variable("action"), resource);
    return q.results().stream()
        .map(action -> action.get("action"))
        .collect(Collectors.toCollection(HashSet::new));
  }

  public static void main(String[] args) throws Exceptions.OsoException, IOException {
    new Oso().repl(args);
  }
}
