package com.osohq.oso;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Arrays;
import java.util.HashSet;
import java.util.Set;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

public class EnforcementTest {
  protected Oso oso;

  public static class User {
    public String name;

    public User(String name) {
      this.name = name;
    }
  }

  public static class Widget {
    public int id;

    public Widget(int id) {
      this.id = id;
    }
  }

  public static class Request {
    public String method;
    public String path;

    public Request(String method, String path) {
      this.method = method;
      this.path = path;
    }
  }

  @BeforeEach
  public void setUp() throws Exception {
    try {
      oso = new Oso();
      oso.registerClass(User.class, "User");
      oso.registerClass(Widget.class, "Widget");
    } catch (Exception e) {
      throw new Error(e);
    }
  }

  @Test
  public void testAuthorize() throws Exception {
    User guest = new User("guest");
    User admin = new User("admin");
    Widget widget0 = new Widget(0);
    Widget widget1 = new Widget(1);

    oso.loadStr(
        "allow(_actor: User, \"read\", widget: Widget) if "
            + "widget.id = 0; "
            + "allow(actor: User, \"update\", _widget: Widget) if "
            + "actor.name = \"admin\";");

    oso.authorize(guest, "read", widget0);
    oso.authorize(admin, "update", widget1);

    // Throws a forbidden exception when user can read resource
    assertThrows(
        Exceptions.ForbiddenException.class, () -> oso.authorize(guest, "update", widget0));

    // Throws a not found exception when user cannot read resource
    assertThrows(Exceptions.NotFoundException.class, () -> oso.authorize(guest, "read", widget1));
    assertThrows(Exceptions.NotFoundException.class, () -> oso.authorize(guest, "update", widget1));

    // With checkRead = false, returns a forbidden exception
    assertThrows(
        Exceptions.ForbiddenException.class, () -> oso.authorize(guest, "read", widget1, false));
    assertThrows(
        Exceptions.ForbiddenException.class, () -> oso.authorize(guest, "update", widget1, false));
  }

  @Test
  public void testAuthorizeRequest() throws Exception {
    oso.registerClass(Request.class, "Request");
    oso.loadStr(
        "allow_request(_: User{name: \"guest\"}, request: Request) if "
            + "request.path.startsWith(\"/repos\"); "
            + "allow_request(_: User{name: \"verified\"}, request: Request) if "
            + "request.path.startsWith(\"/account\"); ");
    User guest = new User("guest");
    User verified = new User("verified");

    oso.authorizeRequest(guest, new Request("GET", "/repos/1"));
    assertThrows(
        Exceptions.ForbiddenException.class,
        () -> oso.authorizeRequest(guest, new Request("GET", "/other")));

    oso.authorizeRequest(verified, new Request("GET", "/account"));
    assertThrows(
        Exceptions.ForbiddenException.class,
        () -> oso.authorizeRequest(guest, new Request("GET", "/account")));
  }

  @Test
  public void testAuthorizedActions() throws Exception {
    oso.loadStr(
        "allow(_actor: User{name: \"sally\"}, action, _resource: Widget{id: 1})"
            + " if action in [\"CREATE\", \"READ\"];");

    User actor = new User("sally");
    Widget widget = new Widget(1);
    HashSet<Object> actions = oso.authorizedActions(actor, widget);

    assertEquals(actions.size(), 2);
    assertTrue(actions.contains("CREATE"));
    assertTrue(actions.contains("READ"));

    oso.clearRules();

    oso.loadStr(
        "allow(_actor: User{name: \"fred\"}, action, _resource: Widget{id: 2})"
            + " if action in [1, 2, 3, 4];");

    User actor2 = new User("fred");
    Widget widget2 = new Widget(2);
    HashSet<Object> actions2 = oso.authorizedActions(actor2, widget2);

    assertEquals(actions2.size(), 4);
    assertTrue(actions2.contains(1));
    assertTrue(actions2.contains(2));
    assertTrue(actions2.contains(3));
    assertTrue(actions2.contains(4));

    User actor3 = new User("doug");
    Widget widget3 = new Widget(4);
    assertTrue(oso.authorizedActions(actor3, widget3).isEmpty());
  }

  @Test
  public void testAuthorizedActionsWildcard() throws Exception {
    oso.loadStr("allow(_actor: User{name: \"John\"}, _action, _resource: Widget{id: 1});");

    User actor = new User("John");
    Widget widget = new Widget(1);

    assertEquals(Set.of("*"), oso.authorizedActions(actor, widget, true));
    assertThrows(Exceptions.OsoException.class, () -> oso.authorizedActions(actor, widget, false));
  }

  @Test
  public void testAuthorizeField() throws Exception {
    oso.loadStr(
        // Admins can update all fields
        "allow_field(actor: User, \"update\", _widget: Widget, field) if "
            + "actor.name = \"admin\" and "
            + "field in [\"name\", \"purpose\", \"private_field\"]; "
            +
            // Anybody who can update a field can also read it
            "allow_field(actor, \"read\", widget: Widget, field) if "
            + "allow_field(actor, \"update\", widget, field); "
            +
            // Anybody can read public fields
            "allow_field(_: User, \"read\", _: Widget, field) if "
            + "field in [\"name\", \"purpose\"];");
    User admin = new User("admin");
    User guest = new User("guest");
    Widget widget = new Widget(0);

    oso.authorizeField(admin, "update", widget, "purpose");
    assertThrows(
        Exceptions.ForbiddenException.class,
        () -> oso.authorizeField(admin, "update", widget, "foo"));

    oso.authorizeField(guest, "read", widget, "purpose");
    assertThrows(
        Exceptions.ForbiddenException.class,
        () -> oso.authorizeField(guest, "read", widget, "private_field"));
  }

  @Test
  public void testAuthorizedFields() throws Exception {
    oso.loadStr(
        // Admins can update all fields
        "allow_field(actor: User, \"update\", _widget: Widget, field) if "
            + "actor.name = \"admin\" and "
            + "field in [\"name\", \"purpose\", \"private_field\"]; "
            +
            // Anybody who can update a field can also read it
            "allow_field(actor, \"read\", widget: Widget, field) if "
            + "allow_field(actor, \"update\", widget, field); "
            +
            // Anybody can read public fields
            "allow_field(_: User, \"read\", _: Widget, field) if "
            + "field in [\"name\", \"purpose\"];");
    User admin = new User("admin");
    User guest = new User("guest");
    Widget widget = new Widget(0);

    // Admins should be able to update all fields
    assertEquals(
        oso.authorizedFields(admin, "update", widget),
        new HashSet(Arrays.asList("name", "purpose", "private_field")));
    // Admins should be able to read all fields
    assertEquals(
        oso.authorizedFields(admin, "read", widget),
        new HashSet(Arrays.asList("name", "purpose", "private_field")));
    // Guests should not be able to update any fields
    assertEquals(oso.authorizedFields(guest, "update", widget), new HashSet());
    // Guests should be able to read public fields
    assertEquals(
        oso.authorizedFields(guest, "read", widget), new HashSet(Arrays.asList("name", "purpose")));
  }

  @Test
  public void testCustomReadAction() throws Exception {
    oso.setReadAction("fetch");
    oso.loadStr("allow(\"graham\", \"fetch\", \"bar\");");
    assertThrows(Exceptions.NotFoundException.class, () -> oso.authorize("sam", "frob", "bar"));
    // A user who can "fetch" should get a ForbiddenException instead of a
    // NotFoundException
    assertThrows(Exceptions.ForbiddenException.class, () -> oso.authorize("graham", "frob", "bar"));
  }
}
