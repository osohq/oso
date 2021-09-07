import com.osohq.oso.*;
import java.util.List;
import java.nio.file.Files;
import java.nio.file.Paths;

public class TestAbac {

  // For testing
  public static Expense defaultExpense(String submittedBy) {
    return new Expense(500, submittedBy, "NYC", 2);
  }

  public static Oso setupOso() throws Exception {
    Oso oso = new Oso();
    oso.registerClass(Expense.class);
    oso.registerClass(User.class);
    oso.registerClass(Project.class);
    oso.registerClass(Team.class);
    oso.registerClass(Organization.class);

    return oso;
  }

  public static void testParses() throws Exception {
    String[] policies = {"01-simple.polar", "02-rbac.polar", "03-hierarchy.polar"};
    Oso oso = setupOso();
    oso.loadFiles(policies);
    // TODO(gj): can probably remove this now that files aren't loaded lazily.
    oso.queryRule("test"); // just to force the load
  }

  public static void testSimple01() throws Exception {
    Oso oso = setupOso();
    oso.loadFile("01-simple.polar");

    if (!oso.isAllowed(new User("sam"), "view", defaultExpense("sam"))) {
      throw new Exception("ABAC docs test failed!");
    }

    if (oso.isAllowed(new User("sam"), "view", defaultExpense("steve"))) {
      throw new Exception("ABAC docs test failed!");
    }
  }

  public static void testRbac02() throws Exception {
    Oso oso = setupOso();
    String policy = new String(Files.readAllBytes(Paths.get("02-rbac.polar")));
    policy += "role(_: User { name: \"sam\" }, \"admin\", _: Project { id: 2 });";
    oso.loadStr(policy);

    Expense expense = new Expense(50, "steve", "NYC", 0);
    if (oso.isAllowed(new User("sam"), "view", expense)) {
      throw new Exception("ABAC docs test failed!");
    }

    expense = new Expense(50, "steve", "NYC", 2);
    if (!oso.isAllowed(new User("sam"), "view", expense)) {
      throw new Exception("ABAC docs test failed!");
    }
  }

  public static void testHierarchy03() throws Exception {
    Oso oso = setupOso();
    oso.loadFile("03-hierarchy.polar");

    if (!oso.isAllowed(new User("bhavik"), "view", defaultExpense("alice"))) {
      throw new Exception("ABAC docs test failed!");
    }
    if (!oso.isAllowed(new User("cora"), "view", defaultExpense("alice"))) {
      throw new Exception("ABAC docs test failed!");
    }
    if (!oso.isAllowed(new User("cora"), "view", defaultExpense("bhavik"))) {
      throw new Exception("ABAC docs test failed!");
    }
    if (oso.isAllowed(new User("bhavik"), "view", defaultExpense("cora"))) {
      throw new Exception("ABAC docs test failed!");
    }
    if (oso.isAllowed(new User("alice"), "view", defaultExpense("cora"))) {
      throw new Exception("ABAC docs test failed!");
    }
    if (oso.isAllowed(new User("alice"), "view", defaultExpense("bhavik"))) {
      throw new Exception("ABAC docs test failed!");
    }
  }

  public static void main(String[] args) throws Exception {
    testParses();
    testSimple01();
    testRbac02();
    testHierarchy03();
    System.out.println("Java ABAC tests pass!");
  }
}
