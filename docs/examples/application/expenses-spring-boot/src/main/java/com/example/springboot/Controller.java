package com.example.springboot;

import org.springframework.web.bind.annotation.RestController;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestHeader;

import java.sql.PreparedStatement;
import java.sql.SQLException;

import javax.annotation.Resource;
import com.osohq.oso.Oso;
import com.osohq.oso.Exceptions.OsoException;

@RestController
public class Controller {
    @Resource(name = "setupOso")
    private Oso oso;

    @Autowired
    private Db db;

    @GetMapping("/expenses/{id}")
    public String getExpense(@PathVariable(name = "id") int id,
            @RequestHeader(name = "user", required = true) String email) {
        try {
            Expense e = Expense.lookup(id);
            User user = User.lookup(email);
            if (!oso.isAllowed(user, "read", e)) {
                return "Forbidden";
            } else {
                return e.toString();
            }
        } catch (OsoException e) {
            return "Failure: " + e.getMessage();
        } catch (SQLException e) {
            return "Expense not found";
        }
    }

    @GetMapping("/whoami")
    public String whoami(@RequestHeader("user") String email) {
        try {
            User you = User.lookup(email);
            if (you != null) {
                PreparedStatement statement = db.prepareStatement("select name from organizations where id = ?");
                statement.setInt(1, you.organizationId);
                String orgName = statement.executeQuery().getString("name");
                return "You are " + you.email + ", the " + you.title + " at " + orgName + ". (User ID: " + you.id + ")";
            }
            return "unimplemented";
        } catch (SQLException e) {
            return "user not found";
        }
    }

    @GetMapping("/organizations/{id}")
    public String getOrganization(@PathVariable(name = "id") int id,
            @RequestHeader(name = "user", required = true) String email) {
        try {
            Organization org = Organization.lookup(id);
            User user = User.lookup(email);
            if (!oso.isAllowed(user, "read", org)) {
                return "Forbidden";
            } else {
                return org.toString();
            }
        } catch (OsoException e) {
            return "Failure: " + e.getMessage();
        } catch (SQLException e) {
            return "Organization not found";
        }
    }

    @PutMapping("/expenses/submit")
    public String submitExpense(@RequestBody Expense expense, @RequestHeader(name = "user") String email)
            throws SQLException {
        try {
            User user = User.lookup(email);

            if (expense.userId == 0)
                expense.userId = user.id;

            if (!oso.isAllowed(user, "create", expense)) {
                return "Forbidden";
            } else {
                expense.save();
                return expense.toString();
            }
        } catch (OsoException e) {
            return "Failure: " + e.getMessage();
        }
    }
}