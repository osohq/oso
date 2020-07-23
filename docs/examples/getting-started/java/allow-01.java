import com.osohq.oso.Oso;

class Allow {
    public static void main(String[] args) {
        Oso oso = new Oso();
        String actor = "alice@example.com";
        Expense resource = EXPENSES[1];
        boolean allowed = oso.allow(actor, "view", resource);
    }

}
