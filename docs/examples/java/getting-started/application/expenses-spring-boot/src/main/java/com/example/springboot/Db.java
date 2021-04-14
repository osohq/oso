package com.example.springboot;

import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.PreparedStatement;
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
  public Db() throws SQLException {
    final String url = "jdbc:sqlite:expenses.db";
    this.db = DriverManager.getConnection(url);
  }

  @PreDestroy
  private void closeDB() {
    try {
      if (this.db != null) {
        db.close();
      }
    } catch (final SQLException ex) {
      System.out.println(ex.getMessage());
    }
  }

  public Connection get() {
    return this.db;
  }

  public PreparedStatement prepareStatement(String query) throws SQLException {
    return this.db.prepareStatement(query);
  }

  public ResultSet queryDB(final String query) throws SQLException {
    final Statement stmt = this.db.createStatement();
    return stmt.executeQuery(query);
  }
}
