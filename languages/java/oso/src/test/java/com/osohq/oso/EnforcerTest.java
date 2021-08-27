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
  protected Oso o;
  protected Enforcer oso;
  protected URL testOso;

  public static class Actor {
    public String name;

    public Actor(String name) {
      this.name = name;
    }

    public List<Company> companies() {
      return List.of(new Company(1));
    }
  }

  public static class Widget {
    public int id;

    public Widget(int id) {
      this.id = id;
    }
  }

  public static class Company {
    public int id;

    public Company(int id) {
      this.id = id;
    }

    public String role(Actor a) {
      if (a.name.equals("president")) {
        return "admin";
      }

      return "guest";
    }

    @Override
    public boolean equals(Object obj) {
      return obj instanceof Company && ((Company) obj).id == this.id;
    }

    @Override
    public int hashCode() {
      return this.id;
    }
  }

  @BeforeEach
  public void setUp() throws Exception {
    try {
      testOso = getClass().getClassLoader().getResource("test_oso.polar");

      o = new Oso();
      o.registerClass(Actor.class, "Actor");
      o.registerClass(Widget.class, "Widget");
      o.registerClass(Company.class, "Company");

      oso = new Enforcer(o);
    } catch (Exception e) {
      throw new Error(e);
    }
  }

  @Test
  public void testIsAllowed() throws Exception {
    o.loadFile(testOso.getPath());
    Actor guest = new Actor("guest");
    Widget resource1 = new Widget(1);
    assertTrue(o.isAllowed(guest, "get", resource1));

    Actor president = new Actor("president");
    Company company = new Company(1);
    assertTrue(o.isAllowed(president, "create", company));
  }

  @Test
  public void testFail() throws Exception {
    o.loadFile(testOso.getPath());
    Actor guest = new Actor("guest");
    Widget widget = new Widget(1);
    assertFalse(o.isAllowed(guest, "not_allowed", widget));
  }

  @Test
  public void testInstanceFromExternalCall() throws Exception {
    o.loadFile(testOso.getPath());
    Company company = new Company(1);
    Actor guest = new Actor("guest");
    assertTrue(o.isAllowed(guest, "frob", company));

    // if the guest user can do it, then the dict should
    // create an instance of the user and be allowed
    HashMap<String, String> userMap = new HashMap<String, String>();
    userMap.put("username", "guest");
    assertTrue(o.isAllowed(userMap, "frob", company));
  }

  @Test
  public void testAllowModel() throws Exception {
    o.loadFile(testOso.getPath());
    Actor auditor = new Actor("auditor");

    assertTrue(o.isAllowed(auditor, "list", Company.class));
    assertFalse(o.isAllowed(auditor, "list", Widget.class));
  }

  @Test
  public void testAuthorizedActions() throws Exception {
    o.loadStr(
        "allow(_actor: Actor{name: \"sally\"}, action, _resource: Widget{id: 1})"
            + " if action in [\"CREATE\", \"READ\"];");

    Actor actor = new Actor("sally");
    Widget widget = new Widget(1);
    HashSet<Object> actions = oso.authorizedActions(actor, widget);

    assertEquals(actions.size(), 2);
    assertTrue(actions.contains("CREATE"));
    assertTrue(actions.contains("READ"));

    o.loadStr(
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
    o.loadStr("allow(_actor: Actor{name: \"John\"}, _action, _resource: Widget{id: 1});");

    Actor actor = new Actor("John");
    Widget widget = new Widget(1);

    assertEquals(Set.of("*"), oso.authorizedActions(actor, widget, true));
    assertThrows(Exceptions.OsoException.class, () -> oso.authorizedActions(actor, widget, false));
  }
}
