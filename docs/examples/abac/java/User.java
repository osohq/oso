import java.util.*;

public class User {
  public String name, location;

  private static Map<String, List<String>> MANAGERS =
      Map.of("cora", List.of("bhavik"), "bhavik", List.of("alice"));

  public User(String name, String location) {
    this.name = name;
    this.location = location;
  }

  public User(String name) {
    this.name = name;
    this.location = "NYC";
  }

  public Enumeration<User> employees() {
    List<User> employees = new ArrayList<User>();
    if (MANAGERS.containsKey(name)) {
      for (String e : MANAGERS.get(name)) {
        employees.add(new User(e));
      }
      return Collections.enumeration(employees);
    } else {
      return Collections.emptyEnumeration();
    }
  }
}
