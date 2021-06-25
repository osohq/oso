package com.osohq.oso;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.net.URL;
import java.util.HashMap;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

public class OsoTest {
  protected Oso o;

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
      o.registerClass(Actor.class, "Actor");
      o.registerClass(Widget.class, "Widget");
      o.registerClass(Company.class, "Company");

      o.loadFile(testOso.getPath());
    } catch (Exception e) {
      throw new Error(e);
    }
  }

  @Test
  public void testIsAllowed() throws Exception {
    Actor guest = new Actor("guest");
    Widget resource1 = new Widget(1);
    assertTrue(o.isAllowed(guest, "get", resource1));

    Actor president = new Actor("president");
    Company company = new Company(1);
    assertTrue(o.isAllowed(president, "create", company));
  }

  @Test
  public void testFail() throws Exception {
    Actor guest = new Actor("guest");
    Widget widget = new Widget(1);
    assertFalse(o.isAllowed(guest, "not_allowed", widget));
  }

  @Test
  public void testInstanceFromExternalCall() throws Exception {
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
    Actor auditor = new Actor("auditor");

    assertTrue(o.isAllowed(auditor, "list", Company.class));
    assertFalse(o.isAllowed(auditor, "list", Widget.class));
  }
}
