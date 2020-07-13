package com.osohq.oso;

import java.util.*;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class PathMapper {
    Pattern pattern;
    List<String> groupNames;

    public PathMapper(String template) {
        Pattern p = Pattern.compile("(\\{([^}]+)\\})");
        Matcher m = p.matcher(template);
        groupNames = new ArrayList<String>();
        while (m.find()) {
            String outer = m.group(1);
            String inner = m.group(2);
            if (inner.equals("*")) {
                template.replaceAll(outer, ".*");
            } else {
                inner.replaceAll("\\{", "\\\\{");
                outer = "\\{" + outer.substring(1, outer.length() - 1) + "\\}";
                template = template.replaceAll(outer, "(?<" + inner + ">[^/]+)");
                groupNames.add(inner);
            }
        }
        pattern = Pattern.compile("^" + template + "$");
    }

    public HashMap<String, String> map(String str) {
        HashMap<String, String> groups = new HashMap<String, String>();
        Matcher m = pattern.matcher(str);
        while (m.find()) {
            for (String name : groupNames) {
                try {
                    groups.put(name, m.group(name));
                } catch (IllegalArgumentException e) {
                    continue;
                }
            }
        }
        return groups;
    }
}
