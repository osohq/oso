package com.example.springboot;

import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestHeader;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestMethod;

import javax.annotation.Resource;
import com.osohq.oso.Oso;
import com.osohq.oso.Exceptions.OsoException;

@RestController
public class Controller {
    @Resource(name = "setupOso")
    private Oso oso;

    @Resource(name = "currentUser")
    private User currentUser;

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
    public String whoami() {
        if (currentUser != null) {
            // TODO

        }
        return "unimplemented";

    }
}