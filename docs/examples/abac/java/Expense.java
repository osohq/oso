import java.lang.module.ModuleDescriptor.Builder;
import java.util.List;

public class Expense {
    public Integer amount, projectId;
    public String submittedBy, location;

    private static List<Expense> EXPENSES = List.of(new Expense(500, "alice", "NYC", 2));

    public Expense(Integer amount, String submittedBy, String location, Integer projectId) {
        this.amount = amount;
        this.projectId = projectId;
        this.submittedBy = submittedBy;
        this.location = location;
    }

    public Expense() {
    }

    public static Expense byId(Integer id) {
        if (id < EXPENSES.size()) {
            return EXPENSES.get(id);
        } else {
            return new Expense();
        }
    }

}