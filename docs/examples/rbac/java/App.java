import java.util.HashSet;

import com.osohq.oso.Oso;

public class App {

// docs: begin-types
public static class Resource {
  public String name;

  public Resource(String name) {
    this.name = name;
  }
}

public static class Organization extends Resource {
  public Organization(String name) {
    super(name);
  }
}

public static class Repository extends Resource {
  public Organization organization;

  public Repository(String name, Organization organization) {
    super(name);
    this.organization = organization;
  }
}

public static class Role {
  public String name;
  public Resource resource;

  public Role(String name, Resource resource) {
    this.name = name;
    this.resource = resource;
  }
}

public static class User {
  public String name;
  public HashSet<Role> roles;

  public User(String name) {
    this.name = name;
    this.roles = new HashSet();
  }

  public void assignRoleForResource(String name, Resource resource) {
    this.roles.add(new Role(name, resource));
  }
}
// docs: end-types

public static Oso setupOso() throws Exception {
// docs: begin-setup
Oso oso = new Oso();

// docs: begin-register
oso.registerClass(Organization.class, "Organization");
oso.registerClass(Repository.class, "Repository");
oso.registerClass(User.class, "User");
// docs: end-register

oso.loadFiles(new String[] { "main.polar" });
// docs: end-setup
return oso;
}
}
