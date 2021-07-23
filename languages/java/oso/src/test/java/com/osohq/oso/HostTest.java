package com.osohq.oso;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

public class HostTest {
  public static class User {}
  ;

  public static class UserSubclass extends User {}
  ;

  public static class NotSubclass {}
  ;

  @Test
  public void isSubclass() {

    Host host = new Host(null);
    host.cacheClass(User.class, "User");
    host.cacheClass(UserSubclass.class, "UserSubclass");
    host.cacheClass(NotSubclass.class, "NotSubclass");

    assertTrue(host.isSubclass("UserSubclass", "User"));
    assertTrue(host.isSubclass("UserSubclass", "UserSubclass"));
    assertTrue(host.isSubclass("User", "User"));
    assertFalse(host.isSubclass("User", "NotSubclass"));
    assertFalse(host.isSubclass("User", "UserSubclass"));
  }
}
