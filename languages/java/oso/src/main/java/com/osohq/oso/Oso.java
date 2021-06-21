package com.osohq.oso;

import java.io.IOException;

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

  public static void main(String[] args) throws Exceptions.OsoException, IOException {
    new Oso().repl(args);
  }
}
