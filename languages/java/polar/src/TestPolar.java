import java.io.PrintWriter;
import java.io.StringWriter;
import java.util.*;
import org.json.*;

public class TestPolar {
    public static final String ANSI_RED = "\u001B[31m";
    public static final String ANSI_GREEN = "\u001B[32m";
    public static final String ANSI_RESET = "\u001B[0m";
    public static final String ANSI_YELLOW = "\u001B[33m";

    public static class MyClass {
        public String name;

        public MyClass(String name) {
            this.name = name;
        }
    }

    public static void testLoadAndQueryStr() {
        String name = "testLoadAndQueryStr";
        Boolean passed = true;
        String msg = null;
        try {
            Polar p = new Polar();

            p.loadStr("f(1);");
            Enumeration<HashMap<String, Object>> results = p.query_str("f(x)");
            if (!results.hasMoreElements() || results.nextElement().get("x") != Integer.valueOf(1)) {
                throw new Exception();
            }
        } catch (Exception e) {
            passed = false;
            msg = getExceptionStackTrace(e);
        } finally {
            printResults(passed, msg, name);
        }
    }

    public static void testInlineQueries() {
        String name = "testInlineQueries";
        Boolean passed = true;
        String msg = null;
        try {
            Polar p = new Polar();
            p.loadStr("f(1); ?= f(1);");
            try {
                p.loadStr("?= f(2);");
            } catch (Error e) {
                return;
            }
            throw new Exception("Expected inline query to fail but it didn't.");

        } catch (Exception e) {
            passed = false;
            msg = getExceptionStackTrace(e);
        } finally {
            printResults(passed, msg, name);
        }
    }

    public static void testToJava() {
        String name = "testToJava";
        Boolean passed = true;
        String msg = null;
        try {
            Polar p = new Polar();

            // Test boolean
            p.loadStr("a(x) := x = true;");
            HashMap<String, Object> a = p.query_str("a(x)").nextElement();
            if (!a.equals(Map.of("x", true))) {
                throw new Exception("Failed to convert boolean to java.");
            }
            // Test dictionary
            p.loadStr("b(x) := x = {a: 1};");
            HashMap<String, Object> b = p.query_str("b(x)").nextElement();
            if (!b.equals(Map.of("x", Map.of("a", 1)))) {
                throw new Exception("Failed to convert dictionary to java.");
            }
            // Test list
            p.loadStr("c(x) := x = [\"a\", \"b\", \"c\"];");
            HashMap<String, Object> c = p.query_str("c(x)").nextElement();
            if (!c.equals(Map.of("x", List.of("a", "b", "c")))) {
                throw new Exception("Failed to convert list to java.");
            }
        } catch (Exception e) {
            passed = false;
            msg = getExceptionStackTrace(e);
        } finally {
            printResults(passed, msg, name);

        }
    }

    public static void testFFIRoundTrip() {
        String name = "testFFIRoundTrip";
        Boolean passed = true;
        String msg = null;
        try {
            Polar p = new Polar();

            // Test Boolean
            Boolean b = true;
            JSONObject polar = p.toPolarTerm(b);
            Object java = p.toJava(polar);
            if (java.getClass() != Boolean.class || java != b) {
                throw new Exception("Failed to convert Boolean to Polar");
            }
            // Test Int
            int i = 3;
            polar = p.toPolarTerm(i);
            java = p.toJava(polar);
            if (java.getClass() != Integer.class || (Integer) java != i) {
                throw new Exception("Failed to convert Integer to Polar");
            }
            // Test Float
            float f = (float) 3.50;
            polar = p.toPolarTerm(f);
            java = p.toJava(polar);
            if (java.getClass() != Float.class || (Float) java != f) {
                throw new Exception("Failed to convert Float to Polar");
            }
            // Test String
            String s = "oso!";
            polar = p.toPolarTerm(s);
            java = p.toJava(polar);
            if (java.getClass() != String.class || (String) java != s) {
                throw new Exception("Failed to convert String to Polar");
            }
            // Test List
            List<Integer> l = List.of(1, 2, 3, 4);
            polar = p.toPolarTerm(l);
            java = p.toJava(polar);
            if (!(java instanceof List) || !((List<Object>) java).equals(l)) {
                throw new Exception("Failed to convert List to Polar");
            }
            // Test Dict
            Map<String, Integer> m = Map.of("a", 1, "b", 2);
            polar = p.toPolarTerm(m);
            java = p.toJava(polar);
            if (!(java instanceof Map) || !((Map<String, Object>) java).equals(m)) {
                throw new Exception("Failed to convert Map to Polar");
            }
            // Test ExternalInstance
            MyClass instance = new MyClass("test");
            polar = p.toPolarTerm(instance);
            java = p.toJava(polar);
            if (java.getClass() != MyClass.class || !((MyClass) java).equals(instance)) {
                throw new Exception("Failed to convert Java Object to Polar");
            }
        } catch (Exception e) {
            passed = false;
            msg = getExceptionStackTrace(e);
        } finally {
            printResults(passed, msg, name);

        }
    }

    public static void testRegisterAndMakeClass() {
        String name = "testRegisterAndMakeClass";
        Boolean passed = true;
        String msg = null;
        try {
            Polar p = new Polar();
            p.registerClass(MyClass.class, m -> new MyClass((String) m.get("name")));

            Map<String, String> testArg = Map.of("name", "testName");
            MyClass instance = (MyClass) p.makeInstance(MyClass.class, testArg, Long.valueOf(0));
            if (instance.name != "testName") {
                throw new Exception();
            }
        } catch (Exception e) {
            passed = false;
            msg = getExceptionStackTrace(e);
        } finally {
            printResults(passed, msg, name);

        }
    }

    private static void printResults(Boolean passed, String message, String name) {
        if (passed) {
            System.out.println(name + ANSI_GREEN + " PASSED." + ANSI_RESET);
        } else {
            System.out.println(name + ANSI_RED + " FAILED:" + ANSI_YELLOW);
            if (message != null)
                System.out.println("\t" + message + ANSI_RESET);
        }
    }

    private static String getExceptionStackTrace(Exception e) {
        StringWriter sw = new StringWriter();
        PrintWriter pw = new PrintWriter(sw);
        e.printStackTrace(pw);
        return sw.toString();
    }

    public static void main(String[] args) {
        System.out.println("\nRunning tests...\n");
        testToJava();
        testFFIRoundTrip();
        testRegisterAndMakeClass();
        testLoadAndQueryStr();
        testInlineQueries();
    }

}