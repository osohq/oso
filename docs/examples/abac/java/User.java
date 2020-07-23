import java.util.*;

public class User {
    public String name, location;

    private static Map<String, List<String>> MANAGERS = Map.of("cora", List.of("bhavik"), "bhavik", List.of("alice"));

    public User(String name, String location) {
        this.name = name;
        this.location = location;
    }

    public User(String name) {
        this.name = name;
        this.location = "NYC";
    }

    public Enumeration<String> employees() {
        if (MANAGERS.containsKey(name)) {
            return Collections.enumeration(MANAGERS.get(name));
        } else {
            return Collections.emptyEnumeration();
        }
    }
}