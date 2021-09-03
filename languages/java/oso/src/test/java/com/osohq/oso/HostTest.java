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

    Polar polar = new Polar();
    Host host = polar.host;
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
