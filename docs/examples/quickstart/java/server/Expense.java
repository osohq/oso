public class Expense {
    public int amount;
    public String description;
    public String submittedBy;

    public Expense(int amount, String description, String submittedBy) {
        this.amount = amount;
        this.description = description;
        this.submittedBy = submittedBy;
    }

    public String toString() {
        return String.format("Expense(%d, %s, %s)", this.amount, this.description, this.submittedBy);
    }
}
