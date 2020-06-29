import java.util.*;
import org.json.*;

public class TestPolar {
    public static void testToJava() {
        Boolean passed = true;
        List<String> failures = new ArrayList<String>();
        Polar p = new Polar();
        // Test boolean
        p.load_str("a(x) := x = true;");
        HashMap<String, Object> a = p.query_str("a(x)").nextElement();
        if (!a.equals(Map.of("x", true))) {
            passed = false;
            failures.add("Failed to convert boolean to java.");
        }
        // Test dictionary
        p.load_str("b(x) := x = {a: 1};");
        HashMap<String, Object> b = p.query_str("b(x)").nextElement();
        if (!b.equals(Map.of("x", Map.of("a", 1)))) {
            passed = false;
            failures.add("Failed to convert dictionary to java.");
        }
        // Test list
        p.load_str("c(x) := x = [\"a\", \"b\", \"c\"];");
        HashMap<String, Object> c = p.query_str("c(x)").nextElement();
        if (!c.equals(Map.of("x", List.of("a", "b", "c")))) {
            passed = false;
            failures.add("Failed to convert list to java.");
        }

        printResults(passed, failures, "testToJava");
    }

    public static void testToPolarTerm() {
        Boolean passed = true;
        List<String> failures = new ArrayList<String>();
        Polar p = new Polar();

        // Test Boolean
        Boolean b = true;
        JSONObject polar = p.toPolarTerm(b);
        Object java = p.toJava(polar);
        if (java.getClass() != Boolean.class || java != b) {
            passed = false;
            failures.add("Failed to convert Boolean to Polar.");
        }
        // Test Int
        int i = 3;
        polar = p.toPolarTerm(i);
        java = p.toJava(polar);
        if (java.getClass() != Integer.class || (Integer) java != i) {
            passed = false;
            failures.add("Failed to convert Integer to Polar.");
        }
        // Test Float
        float f = (float) 3.50;
        polar = p.toPolarTerm(f);
        java = p.toJava(polar);
        if (java.getClass() != Float.class || (Float) java != f) {
            passed = false;
            failures.add("Failed to convert Float to Polar.");
        }
        // Test String
        String s = "oso!";
        polar = p.toPolarTerm(s);
        java = p.toJava(polar);
        if (java.getClass() != String.class || (String) java != s) {
            passed = false;
            failures.add("Failed to convert String to Polar.");
        }
        // Test List
        List<Integer> l = List.of(1, 2, 3, 4);
        polar = p.toPolarTerm(l);
        java = p.toJava(polar);
        if (!(java instanceof List) || !((List<Object>) java).equals(l)) {
            passed = false;
            failures.add("Failed to convert List to Polar.");
        }
        // Test Dict
        Map<String, Integer> m = Map.of("a", 1, "b", 2);
        polar = p.toPolarTerm(m);
        java = p.toJava(polar);
        if (!(java instanceof Map) || !((Map<String, Object>) java).equals(m)) {
            passed = false;
            failures.add("Failed to convert List to Polar.");
        }

        printResults(passed, failures, "testToPolarTerm");

    }

    private static void printResults(Boolean passed, List<String> failures, String name) {
        if (!passed) {
            System.out.println(name + " FAILED:");
            for (String e : failures) {
                System.out.println("\t" + e);
            }
        } else {
            System.out.println(name + " PASSED.");
        }

    }

    public static void main(String[] args) {
        System.out.println("\nRunning tests...");
        testToJava();
        testToPolarTerm();
    }

}