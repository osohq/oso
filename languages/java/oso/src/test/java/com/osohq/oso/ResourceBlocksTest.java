package com.osohq.oso;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

public class ResourceBlocksTest {
  protected Polar p;

  public static class Org {
    public String name;

    public Org(String name) {
      this.name = name;
    }
  }

  public static class Repo {
    public String name;
    public Org org;

    public Repo(String name, Org org) {
      this.name = name;
      this.org = org;
    }
  }

  public static class Issue {
    public String name;
    public Repo repo;

    public Issue(String name, Repo repo) {
      this.name = name;
      this.repo = repo;
    }
  }

  public abstract static class Role {
    public String name;
    public Object resource;
  }

  public static class OrgRole extends Role {
    public Org resource;

    public OrgRole(String name, Org resource) {
      this.name = name;
      this.resource = resource;
    }
  }

  public static class RepoRole extends Role {
    public Repo resource;

    public RepoRole(String name, Repo resource) {
      this.name = name;
      this.resource = resource;
    }
  }

  public static class User {
    public String name;
    public List<Role> roles;

    public User(String name, List<Role> roles) {
      this.name = name;
      this.roles = roles;
    }
  }

  @BeforeEach
  public void setUp() throws Exception {
    try {
      p = new Polar();
      p.registerClass(User.class, "User");
      p.registerClass(Role.class, "Role");
      p.registerClass(Repo.class, "Repo");
      p.registerClass(Org.class, "Org");
      p.registerClass(Issue.class, "Issue");
      p.registerClass(OrgRole.class, "OrgRole");
      p.registerClass(RepoRole.class, "RepoRole");
      p.loadFile("src/test/java/com/osohq/oso/roles_policy.polar");
    } catch (Exceptions.OsoException e) {
      throw new Error(e);
    }
  }

  @Test
  public void testResourceBlocks() {
    Org osohq = new Org("osohq"), apple = new Org("apple");
    Repo oso = new Repo("oso", osohq), ios = new Repo("ios", apple);
    Issue bug = new Issue("bug", oso), laggy = new Issue("laggy", ios);
    Role osohqOwner = new OrgRole("owner", osohq), osohqMember = new OrgRole("member", osohq);

    List<Role> osohqOwnerList = new ArrayList(), osohqMemberList = new ArrayList();
    osohqOwnerList.add(osohqOwner);
    osohqMemberList.add(osohqMember);
    User leina = new User("leina", osohqOwnerList), steve = new User("steve", osohqMemberList);

    assertFalse(p.queryRule("allow", leina, "invite", osohq).results().isEmpty());
    assertFalse(p.queryRule("allow", leina, "create_repo", osohq).results().isEmpty());
    assertFalse(p.queryRule("allow", leina, "push", oso).results().isEmpty());
    assertFalse(p.queryRule("allow", leina, "pull", oso).results().isEmpty());
    assertFalse(p.queryRule("allow", leina, "edit", bug).results().isEmpty());

    assertTrue(p.queryRule("allow", steve, "invite", osohq).results().isEmpty());
    assertFalse(p.queryRule("allow", steve, "create_repo", osohq).results().isEmpty());
    assertTrue(p.queryRule("allow", steve, "push", oso).results().isEmpty());
    assertFalse(p.queryRule("allow", steve, "pull", oso).results().isEmpty());
    assertTrue(p.queryRule("allow", steve, "edit", bug).results().isEmpty());

    assertTrue(p.queryRule("allow", leina, "edit", laggy).results().isEmpty());
    assertTrue(p.queryRule("allow", steve, "edit", laggy).results().isEmpty());

    User gabe = new User("gabe", new ArrayList());
    assertTrue(p.queryRule("allow", gabe, "edit", bug).results().isEmpty());
    gabe = new User("gabe", osohqMemberList);
    assertTrue(p.queryRule("allow", gabe, "edit", bug).results().isEmpty());
    gabe = new User("gabe", osohqOwnerList);
    assertFalse(p.queryRule("allow", gabe, "edit", bug).results().isEmpty());
  }
}
