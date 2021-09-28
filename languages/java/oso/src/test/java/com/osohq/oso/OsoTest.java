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

public class OsoTest {
  protected Oso o;

  public static class User {
    public String name;

    public User(String name) {
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

    public String role(User a) {
      if (a.name.equals("president")) {
        return "admin";
      }

      return "guest";
    }

    @Override
    public boolean equals(Object obj) {
      return obj instanceof OsoTest.Company && ((OsoTest.Company) obj).id == this.id;
    }

    @Override
    public int hashCode() {
      return this.id;
    }
  }

  @BeforeEach
  public void setUp() throws Exception {
    try {
      URL testOso = getClass().getClassLoader().getResource("test_oso.polar");

      o = new Oso();
      o.registerClass(User.class, "User");
      o.registerClass(Widget.class, "Widget");
      o.registerClass(Company.class, "Company");

      o.loadFile(testOso.getPath());
    } catch (Exception e) {
      throw new Error(e);
    }
  }

  @Test
  public void testIsAllowed() throws Exception {
    User guest = new User("guest");
    Widget resource1 = new Widget(1);
    assertTrue(o.isAllowed(guest, "get", resource1));

    User president = new User("president");
    Company company = new Company(1);
    assertTrue(o.isAllowed(president, "create", company));
  }

  @Test
  public void testFail() throws Exception {
    User guest = new User("guest");
    Widget widget = new Widget(1);
    assertFalse(o.isAllowed(guest, "not_allowed", widget));
  }

  @Test
  public void testInstanceFromExternalCall() throws Exception {
    Company company = new Company(1);
    User guest = new User("guest");
    assertTrue(o.isAllowed(guest, "frob", company));

    // if the guest user can do it, then the dict should
    // create an instance of the user and be allowed
    HashMap<String, String> userMap = new HashMap<String, String>();
    userMap.put("username", "guest");
    assertTrue(o.isAllowed(userMap, "frob", company));
  }

  @Test
  public void testAllowModel() throws Exception {
    User auditor = new User("auditor");

    assertTrue(o.isAllowed(auditor, "list", Company.class));
    assertFalse(o.isAllowed(auditor, "list", Widget.class));
  }

  @Test
  public void testGetAllowedActions() throws Exception {

    Oso o = new Oso();
    o.registerClass(User.class, "User");
    o.registerClass(Widget.class, "Widget");

    o.loadStr(
        "allow(_actor: User{name: \"sally\"}, action, _resource: Widget{id: 1})"
            + " if action in [\"CREATE\", \"READ\"];");

    User actor = new User("sally");
    Widget widget = new Widget(1);
    HashSet<Object> actions = o.getAllowedActions(actor, widget);

    assertEquals(actions.size(), 2);
    assertTrue(actions.contains("CREATE"));
    assertTrue(actions.contains("READ"));

    o.clearRules();

    o.loadStr(
        "allow(_actor: User{name: \"fred\"}, action, _resource: Widget{id: 2})"
            + " if action in [1, 2, 3, 4];");

    User actor2 = new User("fred");
    Widget widget2 = new Widget(2);
    HashSet<Object> actions2 = o.getAllowedActions(actor2, widget2);

    assertEquals(actions2.size(), 4);
    assertTrue(actions2.contains(1));
    assertTrue(actions2.contains(2));
    assertTrue(actions2.contains(3));
    assertTrue(actions2.contains(4));

    User actor3 = new User("doug");
    Widget widget3 = new Widget(4);
    assertTrue(o.getAllowedActions(actor3, widget3).isEmpty());
  }

  @Test
  public void testGetAllowedActionsWildcard() throws Exception {
    Oso o = new Oso();

    o.registerClass(User.class, "User");
    o.registerClass(Widget.class, "Widget");

    o.loadStr("allow(_actor: User{name: \"John\"}, _action, _resource: Widget{id: 1});");

    User actor = new User("John");
    Widget widget = new Widget(1);

    assertEquals(Set.of("*"), o.getAllowedActions(actor, widget, true));
    assertThrows(Exceptions.OsoException.class, () -> o.getAllowedActions(actor, widget, false));
  }

  @Test
  public void testNotEqualOperator() {
    Oso oso = new Oso();
    oso.registerClass(User.class, "User");
    oso.loadStr("allow(actor: User, _action, _resource) if actor != nil;");
    assertFalse(oso.isAllowed(null, "foo", "foo"));
  }
}
