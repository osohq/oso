package com.example.springboot;

import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;

import org.springframework.context.annotation.AnnotationConfigApplicationContext;
import org.springframework.stereotype.Component;

public class User {
    public Integer id, locationId, organizationId, managerId;
    public String email, title;

    public User(Integer id, Integer locationId, Integer organizationId, Integer managerId, String email, String title) {
        this.id = id;
        this.locationId = locationId;
        this.organizationId = organizationId;
        this.managerId = managerId;
        this.email = email;
        this.title = title;
    }

    public static User get(int id) throws SQLException {
        AnnotationConfigApplicationContext context = new AnnotationConfigApplicationContext(Application.class);
        Db db = context.getBean(Db.class);
        try {
            PreparedStatement statement = db.prepareStatement(
                    "select id, email, title, location_id, organization_id, manager_id from users where id = ?");
            statement.setInt(1, id);
            ResultSet results = statement.executeQuery();
            return new User(id, results.getInt("location_id"), results.getInt("organization_id"),
                    results.getInt("manager_id"), results.getString("email"), results.getString("title"));
        } finally {
            context.close();
        }
    }

    public static User lookup(String email) throws SQLException {
        AnnotationConfigApplicationContext context = new AnnotationConfigApplicationContext(Application.class);
        Db db = context.getBean(Db.class);
        try {
            PreparedStatement statement = db.prepareStatement(
                    "select id, email, title, location_id, organization_id, manager_id from users where email = ?");
            statement.setString(1, email);
            ResultSet results = statement.executeQuery();
            return new User(results.getInt("id"), results.getInt("location_id"), results.getInt("organization_id"),
                    results.getInt("manager_id"), email, results.getString("title"));
        } finally {
            context.close();
        }
    }

    @Component
    public static class CurrentUser {
        private User user;

        public void set(User user) {
            this.user = user;
        }

        public User get() {
            return this.user;
        }
    }

}