package com.osohq.oso;

import java.io.IOException;
import java.util.*;

public class Oso extends Polar {
    public Oso() throws Exceptions.OsoException {
        super();

        // Register helper classes.
        registerClass(Http.class, "Http");
        registerClass(PathMapper.class, "PathMapper");
    }

    /**
     * Submit an `allow` query to the Polar knowledge base.
     *
     * @param actor
     * @param action
     * @param resource
     * @return
     * @throws Exceptions.OsoException
     */
    public boolean isAllowed(Object actor, Object action, Object resource) throws Exceptions.OsoException {
        return queryRule("allow", actor, action, resource).hasMoreElements();
    }

    public static void main(String[] args) throws Exceptions.OsoException, IOException {
        new Oso().repl(args);
    }
}
