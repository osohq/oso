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

    @RequestMapping(value = "/expense/{id}", method = RequestMethod.GET)
    public String getResource(@PathVariable(name = "id") int id,
            @RequestHeader(name = "user", required = true) String user) {
        try {
            Expense e = Expense.lookup(id);
            if (!oso.isAllowed(user, "GET", e)) {
                return "Forbidden";
            } else {
                return e.toString();
            }
        } catch (OsoException e) {
            return "Failure";
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
}