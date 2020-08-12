package com.example.springboot;

import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.sql.Statement;

import javax.annotation.PreDestroy;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Repository;

@Repository
public class Db {
    private Connection db;

    @Autowired
    public Db() {
        try {
            final String url = "jdbc:sqlite:src/main/expenses.db";
            this.db = DriverManager.getConnection(url);

        } catch (final SQLException e) {
            throw new Error("Problem", e);
        }
    }

    @PreDestroy
    private void closeDB(final Connection db) {
        try {
            if (this.db != null) {
                db.close();
            }
        } catch (final SQLException ex) {
            System.out.println(ex.getMessage());
        }
    }

    public ResultSet queryDB(final String query) throws SQLException {
        final Statement stmt = this.db.createStatement();
        return stmt.executeQuery(query);
    }

}