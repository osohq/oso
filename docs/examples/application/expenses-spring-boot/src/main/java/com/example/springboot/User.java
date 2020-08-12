package com.example.springboot;

import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;

import org.springframework.context.annotation.AnnotationConfigApplicationContext;

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

    public static User get(int id) throws Exception {
        AnnotationConfigApplicationContext context = new AnnotationConfigApplicationContext(Application.class);
        Db db = context.getBean(Db.class);
        try {
            ResultSet results = db.queryDB(
                    "select id, email, title, location_id, organization_id, manager_id from users where id = " + id);
            return new User(id, results.getInt("location_id"), results.getInt("organization_id"),
                    results.getInt("manager_id"), results.getString("email"), results.getString("title"));
        } catch (SQLException e) {
            throw new Exception("user not found");
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

}