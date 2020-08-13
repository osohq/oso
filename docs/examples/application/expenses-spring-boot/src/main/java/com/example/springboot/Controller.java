package com.example.springboot;

import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.server.ResponseStatusException;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;

import java.sql.PreparedStatement;
import java.sql.SQLException;

import javax.annotation.Resource;

import com.example.springboot.User.CurrentUser;
import com.osohq.oso.Oso;
import com.osohq.oso.Exceptions.OsoException;

@RestController
public class Controller {
    @Resource(name = "setupOso")
    private Oso oso;

    @Autowired
    private Db db;

    @Resource(name = "requestScopeCurrentUser")
    private CurrentUser currentUser;

    @RequestMapping("/")
    public String index() throws SQLException {
        return "hello " + currentUser.get().email;
    }

    @GetMapping("/expenses/{id}")
    public String getExpense(@PathVariable(name = "id") int id) {
        try {
            Expense e = Expense.lookup(id);
            if (!oso.isAllowed(currentUser.get(), "read", e)) {
                throw new ResponseStatusException(HttpStatus.UNAUTHORIZED, "oso authorization");
            } else {
                return e.toString();
            }
        } catch (

        OsoException e) {
            throw new ResponseStatusException(HttpStatus.INTERNAL_SERVER_ERROR, null, e);
        } catch (SQLException e) {
            throw new ResponseStatusException(HttpStatus.BAD_REQUEST, "Expense not found", e);
        }
    }

    @GetMapping("/whoami")
    public String whoami() {
        try {
            User you = currentUser.get();
            if (you != null) {
                PreparedStatement statement = db.prepareStatement("select name from organizations where id = ?");
                statement.setInt(1, you.organizationId);
                String orgName = statement.executeQuery().getString("name");
                return "You are " + you.email + ", the " + you.title + " at " + orgName + ". (User ID: " + you.id + ")";
            }
            return "unimplemented";
        } catch (SQLException e) {
            throw new ResponseStatusException(HttpStatus.BAD_REQUEST, "User not found", e);
        }
    }

    @GetMapping("/organizations/{id}")
    public String getOrganization(@PathVariable(name = "id") int id) {
        try {
            Organization org = Organization.lookup(id);
            if (!oso.isAllowed(currentUser.get(), "read", org)) {
                throw new ResponseStatusException(HttpStatus.UNAUTHORIZED, "oso authorization");
            } else {
                return org.toString();
            }
        } catch (OsoException e) {
            throw new ResponseStatusException(HttpStatus.INTERNAL_SERVER_ERROR, null, e);
        } catch (SQLException e) {
            throw new ResponseStatusException(HttpStatus.BAD_REQUEST, "Organization not found", e);
        }
    }

    @PutMapping("/expenses/submit")
    public String submitExpense(@RequestBody Expense expense) {
        try {
            User user = currentUser.get();
            if (expense.userId == 0)
                expense.userId = user.id;

            if (!oso.isAllowed(user, "create", expense)) {
                throw new ResponseStatusException(HttpStatus.UNAUTHORIZED, "oso authorization");
            } else {
                expense.save();
                return expense.toString();
            }
        } catch (OsoException e) {
            throw new ResponseStatusException(HttpStatus.INTERNAL_SERVER_ERROR, null, e);
        } catch (SQLException e) {
            throw new ResponseStatusException(HttpStatus.BAD_REQUEST, "failed to save expense", e);
        }
    }
}