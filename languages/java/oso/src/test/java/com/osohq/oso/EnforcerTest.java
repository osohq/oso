package com.osohq.oso;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.net.URL;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

public class EnforcerTest {
  protected Oso policy;
  protected Enforcer oso;

  public static class Actor {
    public String name;

    public Actor(String name) {
      this.name = name;
    }
  }

  public static class Widget {
    public int id;

    public Widget(int id) {
      this.id = id;
    }
  }

  @BeforeEach
  public void setUp() throws Exception {
    try {
      policy = new Oso();
      policy.registerClass(Actor.class, "Actor");
      policy.registerClass(Widget.class, "Widget");

      oso = new Enforcer(policy);
    } catch (Exception e) {
      throw new Error(e);
    }
  }

  @Test
  public void testAuthorize() throws Exception {
    Actor guest = new Actor("guest");
    Actor admin = new Actor("admin");
    Widget widget0 = new Widget(0);
    Widget widget1 = new Widget(1);

    policy.loadStr(
      "allow(_actor: Actor, \"read\", widget: Widget) if " +
        "widget.id = 0; " +
      "allow(actor: Actor, \"update\", _widget: Widget) if " +
        "actor.name = \"admin\";"
    );

    oso.authorize(guest, "read", widget0);
    oso.authorize(admin, "update", widget1);

    // Throws a forbidden error when user can read resource
    assertThrows(Exceptions.ForbiddenException.class, () -> oso.authorize(guest, "update", widget0));

    // Throws a not found error when user cannot read resource
    assertThrows(Exceptions.NotFoundException.class, () -> oso.authorize(guest, "read", widget1));
    assertThrows(Exceptions.NotFoundException.class, () -> oso.authorize(guest, "update", widget1));

    // With checkRead = false, returns a forbidden error
    assertThrows(Exceptions.ForbiddenException.class, () -> oso.authorize(guest, "update", widget1, false));
  }

  @Test
  public void testAuthorizedActions() throws Exception {
    oso.policy.loadStr(
        "allow(_actor: Actor{name: \"sally\"}, action, _resource: Widget{id: 1})"
            + " if action in [\"CREATE\", \"READ\"];");

    Actor actor = new Actor("sally");
    Widget widget = new Widget(1);
    HashSet<Object> actions = oso.authorizedActions(actor, widget);

    assertEquals(actions.size(), 2);
    assertTrue(actions.contains("CREATE"));
    assertTrue(actions.contains("READ"));

    oso.policy.loadStr(
        "allow(_actor: Actor{name: \"fred\"}, action, _resource: Widget{id: 2})"
            + " if action in [1, 2, 3, 4];");

    Actor actor2 = new Actor("fred");
    Widget widget2 = new Widget(2);
    HashSet<Object> actions2 = oso.authorizedActions(actor2, widget2);

    assertEquals(actions2.size(), 4);
    assertTrue(actions2.contains(1));
    assertTrue(actions2.contains(2));
    assertTrue(actions2.contains(3));
    assertTrue(actions2.contains(4));

    Actor actor3 = new Actor("doug");
    Widget widget3 = new Widget(4);
    assertTrue(oso.authorizedActions(actor3, widget3).isEmpty());
  }

  @Test
  public void testAuthorizedActionsWildcard() throws Exception {
    policy.loadStr("allow(_actor: Actor{name: \"John\"}, _action, _resource: Widget{id: 1});");

    Actor actor = new Actor("John");
    Widget widget = new Widget(1);

    assertEquals(Set.of("*"), oso.authorizedActions(actor, widget, true));
    assertThrows(Exceptions.OsoException.class, () -> oso.authorizedActions(actor, widget, false));
  }
}
