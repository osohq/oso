package com.example.springboot;

import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;

import org.springframework.context.annotation.AnnotationConfigApplicationContext;

public class Expense {
    public Integer amount, userId, id;
    public String description;

    public Expense(Integer amount, String description, Integer userId, Integer id) {
        this.amount = amount;
        this.description = description;
        this.userId = userId;
        this.id = id;
    }

    public static Expense lookup(int id) throws SQLException {
        AnnotationConfigApplicationContext context = new AnnotationConfigApplicationContext(Application.class);
        Db db = context.getBean(Db.class);
        try {
            PreparedStatement statement = db
                    .prepareStatement("select id, amount, description, user_id from expenses where id  = ?");
            statement.setInt(1, id);
            ResultSet results = statement.executeQuery();
            return new Expense(results.getInt("amount"), results.getString("description"), results.getInt("user_id"),
                    id);
        } finally {
            context.close();
        }
    }

    public String toString() {
        return String.format("Expense(amount=%d, description=%s, user_id=%d, id=%d)", this.amount, this.description,
                this.userId, this.id);
    }
}