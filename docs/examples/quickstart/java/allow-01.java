import com.osohq.oso.Oso;

class Allow {
    public static void main(String[] args) {
        Oso oso = new Oso();
        String actor = "alice@example.com";
        Expense resource = Expenses.EXPENSES[1];
        boolean allowed = oso.isAllowed(actor, "GET", resource);
    }

}
