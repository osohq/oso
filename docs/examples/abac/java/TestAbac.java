import java.util.List;

import com.osohq.oso.*;

public class TestAbac {

    // For testing
    public static Expense defaultExpense(String submittedBy) {
        return new Expense(500, submittedBy, "NYC", 2);
    }

    public static Oso setupOso() throws Exception {
        Oso oso = new Oso();
        oso.registerClass(Expense.class, (args) -> Expense.byId((Integer) args.get("id")));
        oso.registerClass(User.class, (args) -> new User((String) args.get("name")));
        oso.registerClass(Project.class, (args) -> Project.byId((Integer) args.get("id")));
        oso.registerClass(Team.class, (args) -> Team.byId((Integer) args.get("id")));
        oso.registerClass(Organization.class, (args) -> Organization.byId((Integer) args.get("id")));

        return oso;

    }

    public static void testParses() throws Exception {
        List<String> policies = List.of("01-simple.polar", "02-rbac.polar", "03-hierarchy.polar");
        Oso oso = setupOso();
        for (String policy : policies) {
            oso.loadFile(policy);
            oso.query("test", null); // just to force the load
        }
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
        oso.loadFile("02-rbac.polar");
        oso.loadStr("role(_: User { name: \"sam\" }, \"admin\", __: Project { id: 2 });");

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