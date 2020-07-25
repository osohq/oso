package com.osohq.oso;

import java.util.*;

public class Oso extends Polar {
    public Oso() throws Exceptions.OsoException {
        super();
        registerClass(Http.class, (m) -> new Http((String) m.get("hostname"), (String) m.get("path"),
                (Map<String, String>) m.get("query")), "Http");
        registerClass(PathMapper.class, (m) -> new PathMapper((String) m.get("template")), "PathMapper");
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
        return query("allow", List.of(actor, action, resource)).hasMoreElements();
    }
}
