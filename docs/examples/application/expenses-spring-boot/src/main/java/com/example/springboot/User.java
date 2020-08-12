package com.example.springboot;

import java.sql.ResultSet;
import java.sql.SQLException;

import org.springframework.context.annotation.AnnotationConfigApplicationContext;

public class User {
    int id, locationId, organizationId, managerId;
    String email, title;

    public User(int id, int locationId, int organizationId, int managerId, String email, String title) {
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
            return new User(id, results.getInt("locationId"), results.getInt("organizationId"),
                    results.getInt("managerId"), results.getString("email"), results.getString("title"));
        } catch (SQLException e) {
            throw new Exception("user not found");
        } finally {
            context.close();
        }
    }

    public static User lookup(String email) throws Exception {
        AnnotationConfigApplicationContext context = new AnnotationConfigApplicationContext(Application.class);
        Db db = context.getBean(Db.class);
        try {
            ResultSet results = db.queryDB(
                    "select id, email, title, location_id, organization_id, manager_id from users where email = "
                            + email);
            return new User(results.getInt("id"), results.getInt("locationId"), results.getInt("organizationId"),
                    results.getInt("managerId"), email, results.getString("title"));
        } catch (SQLException e) {
            throw new Exception("user not found");
        } finally {
            context.close();
        }
    }

}