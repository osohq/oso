import java.io.File;
import java.io.FileWriter;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.util.*;
import org.json.*;
import java.lang.reflect.*;
import java.lang.annotation.*;

public class TestPolar {
    public static final String ANSI_RED = "\u001B[31m";
    public static final String ANSI_GREEN = "\u001B[32m";
    public static final String ANSI_RESET = "\u001B[0m";
    public static final String ANSI_YELLOW = "\u001B[33m";

    public static class MyClass {
        public String name;
        public int id;

        public MyClass(String name, int id) {
            this.name = name;
            this.id = id;
        }

        public String myMethod(String arg) {
            return arg;
        }
    }

    public static class MySubClass extends MyClass {
        public MySubClass(String name, int id) {
            super(name, id);
        }
    }

    /**** TEST QUERY ****/

    public static void testLoadAndQueryStr() throws Exception {
        Polar p = new Polar();

        p.loadStr("f(1);");
        Polar.Query results = p.queryStr("f(x)");
        if (!results.hasMoreElements() || results.nextElement().get("x") != Integer.valueOf(1)) {
            throw new Exception();
        }
    }

    public static void testInlineQueries() throws Exception {
        Polar p = new Polar();
        p.loadStr("f(1); ?= f(1);");
        try {
            p.loadStr("?= f(2);");
        } catch (Exceptions.InlineQueryFailedError e) {
            return;
        }
        throw new Exception("Expected inline query to fail but it didn't.");

    }

    public static void testBasicQueryPred() throws Exception {
        Polar p = new Polar();
        // test basic query
        p.loadStr("f(a, b) := a = b;");
        if (p.queryPred("f", List.of(1, 1)).results().isEmpty()) {
            throw new Exception("Basic predicate query failed.");
        }
        if (!p.queryPred("f", List.of(1, 2)).results().isEmpty()) {
            throw new Exception("Basic predicate query expected to fail but didn't.");
        }
    }

    public static void testQueryPredWithObject() throws Exception {
        Polar p = new Polar();
        registerClasses(p);
        // test query with Java Object
        p.loadStr("g(x) := x.id = 1;");
        if (p.queryPred("g", List.of(new MyClass("test", 1))).results().isEmpty()) {
            throw new Exception("Predicate query with Java Object failed.");
        }
        if (!p.queryPred("g", List.of(new MyClass("test", 2))).results().isEmpty()) {
            throw new Exception("Predicate query with Java Object expected to fail but didn't.");
        }
    }

    public static void testQueryPredWithVariable() throws Exception {
        Polar p = new Polar();
        // test query with Variable
        p.loadStr("f(a, b) := a = b;");
        if (!p.queryPred("f", List.of(1, new Polar.Variable("result"))).results()
                .equals(List.of(Map.of("result", 1)))) {
            throw new Exception("Predicate query with Variable failed.");
        }
    }

    public static void testExternalIsa() throws Exception {
        Polar p = new Polar();
        registerClasses(p);
        p.loadStr("f(a: MyClass, x) := x = a.id;");
        List<HashMap<String, Object>> result = p
                .queryPred("f", List.of(new MyClass("test", 1), new Polar.Variable("x"))).results();
        if (!result.equals(List.of(Map.of("x", 1)))) {
            throw new Exception();
        }
        p.clear();

        p.loadStr("f(a: MySubClass, x) := x = a.id;");
        result = p.queryPred("f", List.of(new MyClass("test", 1), new Polar.Variable("x"))).results();
        if (!result.isEmpty()) {
            throw new Exception("Failed to filter rules by specializers.");
        }
        p.clear();

        boolean throwsError = false;
        try {
            p.loadStr("f(a: OtherClass, x) := x = a.id;");
            p.queryPred("f", List.of(new MyClass("test", 1), new Polar.Variable("x"))).results();
        } catch (Exceptions.UnregisteredClassError e) {
            throwsError = true;
        }
        if (!throwsError) {
            throw new Exception("Failed to throw unregistered class error");
        }

    }

    public static void testExternalIsSubSpecializer() throws Exception {
        Polar p = new Polar();
        registerClasses(p);

        p.loadStr("f(a: MySubClass, x) := x = 1;");
        p.loadStr("f(a: MyClass, x) := x = 2;");
        List<HashMap<String, Object>> result = p
                .queryPred("f", List.of(new MySubClass("test", 1), new Polar.Variable("x"))).results();
        if (!result.equals(List.of(Map.of("x", 1), Map.of("x", 2)))) {
            throw new Exception("Failed to order rules based on specializers.");
        }

        result = p.queryPred("f", List.of(new MyClass("test", 1), new Polar.Variable("x"))).results();
        if (!result.equals(List.of(Map.of("x", 2)))) {
            throw new Exception("Failed to order rules based on specializers.");
        }
    }

    /*** TEST FFI CONVERSIONS ***/

    public static void testBoolFFIRoundTrip() throws Exception {
        Polar p = new Polar();
        Boolean b = true;
        JSONObject polar = p.toPolarTerm(b);
        Object java = p.toJava(polar);
        if (java.getClass() != Boolean.class || java != b) {
            throw new Exception();
        }
    }

    public static void testIntFFIRoundTrip() throws Exception {
        Polar p = new Polar();
        int i = 3;
        JSONObject polar = p.toPolarTerm(i);
        Object java = p.toJava(polar);
        if (java.getClass() != Integer.class || (Integer) java != i) {
            throw new Exception();
        }
    }

    public static void testFloatFFIRoundTrip() throws Exception {
        Polar p = new Polar();
        float f = (float) 3.50;
        JSONObject polar = p.toPolarTerm(f);
        Object java = p.toJava(polar);
        if (java.getClass() != Float.class || (Float) java != f) {
            throw new Exception();
        }
    }

    public static void testListFFIRoundTrip() throws Exception {
        Polar p = new Polar();
        List<Integer> l = List.of(1, 2, 3, 4);
        JSONObject polar = p.toPolarTerm(l);
        Object java = p.toJava(polar);
        if (!(java instanceof List) || !((List<Object>) java).equals(l)) {
            throw new Exception();
        }

    }

    public static void testDictFFIRoundTrip() throws Exception {
        Polar p = new Polar();
        Map<String, Integer> m = Map.of("a", 1, "b", 2);
        JSONObject polar = p.toPolarTerm(m);
        Object java = p.toJava(polar);
        if (!(java instanceof Map) || !((Map<String, Object>) java).equals(m)) {
            throw new Exception();
        }

    }

    public static void testJavaClassFFIRoundTrip() throws Exception {
        Polar p = new Polar();
        MyClass instance = new MyClass("test", 1);
        JSONObject polar = p.toPolarTerm(instance);
        Object java = p.toJava(polar);
        if (java.getClass() != MyClass.class || !((MyClass) java).equals(instance)) {
            throw new Exception();
        }
    }

    public static void testPredicateFFIRoundTrip() throws Exception {
        Polar p = new Polar();
        Polar.Predicate pred = new Polar.Predicate("name", List.of(1, "hello"));
        JSONObject polar = p.toPolarTerm(pred);
        Object java = p.toJava(polar);
        if (java.getClass() != Polar.Predicate.class || !((Polar.Predicate) java).equals(pred)) {
            throw new Exception();
        }

    }

    /*** TEST EXTERNALS ***/

    public static void testRegisterAndMakeClass() throws Exception {
        Polar p = new Polar();
        registerClasses(p);

        Map<String, Object> testArg = Map.of("name", "testName", "id", 1);
        MyClass instance = (MyClass) p.makeInstance("MyClass", testArg, Long.valueOf(0));
        if (instance.name != "testName" || instance.id != 1) {
            throw new Exception();
        }
    }

    public static void testMakeInstanceFromPolar() throws Exception {
        Polar p = new Polar();
        registerClasses(p);
        p.loadStr("f(x) := x = new MyClass{name: \"test\", id: 1};");
        Polar.Query query = p.queryStr("f(x)");
        MyClass ret = (MyClass) query.nextElement().get("x");
        if (ret.id != 1 || !ret.name.equals("test")) {
            throw new Exception();
        }

    }

    public static void testRegisterCall() throws Exception {
        Polar p = new Polar();
        registerClasses(p);
        MyClass instance = new MyClass("test", 1);
        p.cacheInstance(instance, Long.valueOf(1));
        p.registerCall("myMethod", List.of("hello world"), 1, 1);
        JSONObject res = p.nextCallResult(1);
        if (!p.toJava(res).equals("hello world")) {
            throw new Exception();
        }
    }

    public static void testExternalCall() throws Exception {
        Polar p = new Polar();
        registerClasses(p);

        // Test get attribute
        p.loadStr("id(x) := x = new MyClass{name: \"test\", id: 1}.id;");
        if (!p.queryStr("id(x)").results().equals(List.of(Map.of("x", 1)))) {
            throw new Exception("Failed to get attribute on external instance.");
        }

        // Test call method
        p.loadStr("method(x) := x = new MyClass{name: \"test\", id: 1}.myMethod(\"hello world\");");
        if (!p.queryStr("method(x)").results().equals(List.of(Map.of("x", "hello world")))) {
            throw new Exception("Failed to get attribute on external instance.");
        }
    }

    /**** TEST LOADING ****/

    public static void testLoadFile() throws Exception {
        Polar p = new Polar();
        p.loadFile("src/test.polar");
        if (!p.queryStr("f(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3)))) {
            throw new Exception("Failed to load file");
        }
    }

    public static void testLoadNonPolarFile() throws Exception {
        Polar p = new Polar();
        Boolean throwsError = false;
        try {
            p.loadFile("wrong.txt");
        } catch (Exceptions.PolarFileExtensionError e) {
            throwsError = true;
        }
        if (!throwsError) {
            throw new Exception("Failed to catch incorrect Polar file extension.");
        }
    }

    public static void testLoadFilePassesFilename() throws Exception {
        Polar p = new Polar();
        File tempFile = File.createTempFile("error-", ".polar");
        FileWriter w = new FileWriter(tempFile);
        w.write(";");
        w.close();
        Boolean throwsError = false;
        try {
            p.loadFile(tempFile.getPath());
            p.queryStr("f(1)");
        } catch (Exceptions.ParseError e) {
            // TODO: check error message
            throwsError = true;
        }
        if (!throwsError) {
            throw new Exception("Failed to pass filename across FFI boundary.");
        }
        tempFile.deleteOnExit();
    }

    public static void testLoadFileIdempotent() throws Exception {
        Polar p = new Polar();
        p.loadFile("src/test.polar");
        p.loadFile("src/test.polar");
        if (!p.queryStr("f(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))))

        {
            throw new Exception("loadFile behavior is not idempotent.");
        }
    }

    public static void testLoadMultipleFiles() throws Exception {
        Polar p = new Polar();
        p.loadFile("src/test.polar");
        p.loadFile("src/test2.polar");
        if (!p.queryStr("f(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))))

        {
            throw new Exception("Failed to load multiple files.");
        }
        if (!p.queryStr("g(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3)))) {
            throw new Exception("Failed to load multiple files.");
        }
    }

    private static void registerClasses(Polar p) throws Exceptions.DuplicateClassAliasError {
        p.registerClass(MyClass.class, "MyClass", m -> new MyClass((String) m.get("name"), (int) m.get("id")));
        p.registerClass(MySubClass.class, "MySubClass", m -> new MySubClass((String) m.get("name"), (int) m.get("id")));
    }

    private static void printResults(Status status, String message, String name) {
        switch (status) {
            case PASSED:
                System.out.println(name + ANSI_GREEN + " PASSED" + ANSI_RESET);
                break;
            case FAILED:
                System.out.print(name + ANSI_RED + " FAILED" + ANSI_RESET);
                if (message != null)
                    System.out.println("\t" + message);
                break;
            case SKIPPED:
                System.out.print(name + ANSI_YELLOW + " SKIPPED" + ANSI_RESET);
                if (message != null)
                    System.out.println(" " + message);
                break;

        }
    }

    private static String getExceptionStackTrace(Throwable e) {
        StringWriter sw = new StringWriter();
        PrintWriter pw = new PrintWriter(sw);
        e.printStackTrace(pw);
        return sw.toString();
    }

    private static void runAll() throws IllegalAccessException {
        System.out.println("\nRunning tests...\n");
        Method[] methods = TestPolar.class.getDeclaredMethods();
        int total = methods.length;
        int nFailed = 0;
        int nSkipped = 0;
        for (Method m : methods) {
            Status status = Status.PASSED;
            String msg = null;
            String name = m.getName();
            if (name.startsWith("test")) {
                if (m.isAnnotationPresent(Skip.class)) {
                    status = Status.SKIPPED;
                    msg = m.getAnnotation(Skip.class).reason();
                    nSkipped++;
                } else {
                    try {
                        m.invoke(null);
                    } catch (InvocationTargetException e) {
                        status = Status.FAILED;
                        msg = getExceptionStackTrace(e.getCause());
                        nFailed++;
                    }

                }
                printResults(status, msg, name);
            }
        }
        int nPassed = total - nFailed - nSkipped;
        System.out.println("\n" + nPassed + "/" + total + ANSI_GREEN + " PASSED." + ANSI_RESET);
        System.out.println(nSkipped + "/" + total + ANSI_YELLOW + " SKIPPED." + ANSI_RESET);
        System.out.println(nFailed + "/" + total + ANSI_RED + " FAILED." + ANSI_RESET);

    }

    @Retention(RetentionPolicy.RUNTIME)
    @Target(ElementType.METHOD)
    private @interface Skip {
        String reason() default "";
    }

    private enum Status {
        PASSED, SKIPPED, FAILED
    }

    public static void main(String[] args) throws IllegalAccessException {
        runAll();
    }

}