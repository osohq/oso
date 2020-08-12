package com.example.springboot;

import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;

import org.springframework.context.annotation.AnnotationConfigApplicationContext;

public class Organization {
    public String name;
    public Integer id;

    public Organization(String name, Integer id) {
        this.name = name;
        this.id = id;
    }

    public static Organization lookup(int id) throws SQLException {
        AnnotationConfigApplicationContext context = new AnnotationConfigApplicationContext(Application.class);
        Db db = context.getBean(Db.class);
        try {
            PreparedStatement statement = db.prepareStatement("select id, name from organizations where id = ?");
            statement.setInt(1, id);
            ResultSet results = statement.executeQuery();
            return new Organization(results.getString("name"), results.getInt("id"));
        } finally {
            context.close();
        }

    }

    // @bp.route("/<int:id>", methods=["GET"])
    // def get_organization(id):
    // organization = Organization.lookup(id)
    // return str(authorize("read", organization))
}