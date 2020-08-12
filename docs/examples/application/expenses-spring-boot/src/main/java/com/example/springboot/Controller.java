package com.example.springboot;

import org.springframework.web.bind.annotation.RestController;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.AnnotationConfigApplicationContext;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestHeader;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestMethod;

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

    @RequestMapping(value = "/expenses/{id}", method = RequestMethod.GET)
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

    @RequestMapping("/whoami")
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

    @RequestMapping("/organizations/{id}")
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
}