package com.example.springboot;

public class Expense {
    public int amount;
    public String description;
    public String submittedBy;

    public static Expense[] EXPENSES = { new Expense(500, "coffee", "alice@example.com"),
            new Expense(5000, "software", "alice@example.com"), new Expense(50000, "flight", "bhavik@example.com"), };

    public Expense(int amount, String description, String submittedBy) {
        this.amount = amount;
        this.description = description;
        this.submittedBy = submittedBy;
    }

    public static Expense lookup(int id) {
        if (id < EXPENSES.length) {
            return EXPENSES[id];
        } else {
            return null;
        }
    }

    public String toString() {
        return String.format("Expense(amount=%d, description=%s, submittedBy=%s)", this.amount, this.description,
                this.submittedBy);
    }
}