package com.osohq.oso;

import java.io.IOException;

public class Oso extends Polar {
    public Oso() throws Exceptions.OsoException {
        super();

        // Register helper classes.
        registerClass(Http.class, "Http");
        registerClass(PathMapper.class, "PathMapper");
    }

    /**
     * Submit an `allow` query to the Polar knowledge base.
     */
    public boolean isAllowed(Object actor, Object action, Object resource) throws Exceptions.OsoException {
        return queryRule("allow", actor, action, resource).hasMoreElements();
    }

    public static void main(String[] args) throws Exceptions.OsoException, IOException {
        new Oso().repl(args);
    }
}
