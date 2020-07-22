package com.osohq.oso;

import java.util.Map;

public class Http {
    public String hostname, path;
    public Map<String, String> query;

    public Http(String hostname, String path, Map<String, String> query) {
        this.hostname = hostname;
        this.path = path;
        this.query = query;

    }

}