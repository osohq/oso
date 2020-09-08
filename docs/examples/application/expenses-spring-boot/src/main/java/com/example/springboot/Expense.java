package com.example.springboot;

import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.time.LocalDate;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import org.springframework.context.annotation.AnnotationConfigApplicationContext;

public class Expense {
    public Integer amount;
    public String description;
    public Integer userId, id;

    public Expense(Integer amount, String description, Integer userId, Integer id) {
        this.amount = amount;
        this.description = description;
        this.userId = userId;
        this.id = id;
    }

    @JsonCreator
    public Expense(@JsonProperty("amount") int amount, @JsonProperty("description") String description,
            @JsonProperty("user_id") int userId) {
        this.amount = amount;
        this.description = description;
        this.userId = userId;
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

    public void save() throws SQLException {
        LocalDate now = LocalDate.now();
        AnnotationConfigApplicationContext context = new AnnotationConfigApplicationContext(Application.class);
        Db db = context.getBean(Db.class);
        try {
            PreparedStatement statement = db.prepareStatement(
                    "INSERT INTO expenses (amount, description, user_id, created_at, updated_at) VALUES(?, ?, ?, ?, ?)");
            statement.setInt(1, this.amount);
            statement.setString(2, this.description);
            statement.setInt(3, this.userId);
            statement.setString(4, now.toString());
            statement.setString(5, now.toString());
            statement.executeUpdate();

            ResultSet rs = statement.getGeneratedKeys();
            if (rs.next()) {
                this.id = rs.getInt(1);
            }

        } finally {
            context.close();
        }
    }
}