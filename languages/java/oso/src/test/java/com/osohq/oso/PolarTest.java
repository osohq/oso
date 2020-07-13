package com.osohq.oso;

import static org.junit.Assert.assertThrows;

import java.io.File;
import java.io.FileWriter;
import java.util.*;
import org.json.*;

import junit.framework.Test;
import junit.framework.TestCase;
import junit.framework.TestSuite;

public class PolarTest extends TestCase {
    protected Polar p;

    public static class MyClass {
        public String name;
        public Integer id;

        public MyClass(String name, Integer id) {
            this.name = name;
            this.id = id;
        }

        public String myMethod(String arg) {
            return arg;
        }

        public List<String> myList() {
            return List.of("hello", "world");
        }

        public MySubClass mySubClass(String name, Integer id) {
            return new MySubClass(name, id);
        }

        public Enumeration<String> myEnumeration() {
            return Collections.enumeration(List.of("hello", "world"));
        }
    }

    public static class MySubClass extends MyClass {
        public MySubClass(String name, Integer id) {
            super(name, id);
        }
    }

    /**
     * Create the test case
     *
     * @param testName name of the test case
     */
    public PolarTest(String testName) {
        super(testName);
    }

    @Override
    public void setUp() {
        try {
            p = new Polar();
            p.registerClass(MyClass.class, m -> new MyClass((String) m.get("name"), (int) m.get("id")), "MyClass");
            p.registerClass(MySubClass.class, m -> new MySubClass((String) m.get("name"), (int) m.get("id")),
                    "MySubClass");
        } catch (Exceptions.OsoException e) {
            throw new Error(e);
        }
    }

    /**
     * @return the suite of tests being tested
     */
    public static Test suite() {
        return new TestSuite(PolarTest.class);
    }

    /**
     * Rigourous Test :-)
     */
    public void testApp() {
        assertTrue(true);
    }

    /**** TEST QUERY ****/

    public void testLoadAndQueryStr() throws Exception {
        p.loadStr("f(1);");
        Query query = p.queryStr("f(x)");
        assertEquals(List.of(Map.of("x", 1)), query.results());
    }

    public void testInlineQueries() throws Exception {
        p.loadStr("f(1); ?= f(1);");
        assertThrows("Expected inline query to fail but it didn't.", Exceptions.InlineQueryFailedError.class,
                () -> p.loadStr("?= f(2);"));
    }

    public void testBasicQueryPred() throws Exception {
        // test basic query
        p.loadStr("f(a, b) := a = b;");
        assertFalse("Basic predicate query failed.", p.queryPred("f", List.of(1, 1)).results().isEmpty());
        assertTrue("Basic predicate query expected to fail but didn't.",
                p.queryPred("f", List.of(1, 2)).results().isEmpty());
    }

    public void testQueryPredWithObject() throws Exception {
        // test query with Java Object
        p.loadStr("g(x) := x.id = 1;");
        assertFalse("Predicate query with Java Object failed.",
                p.queryPred("g", List.of(new MyClass("test", 1))).results().isEmpty());
        assertTrue("Predicate query with Java Object expected to fail but didn't.",
                p.queryPred("g", List.of(new MyClass("test", 2))).results().isEmpty());
    }

    public void testQueryPredWithVariable() throws Exception {
        // test query with Variable
        p.loadStr("f(a, b) := a = b;");
        assertTrue("Predicate query with Variable failed.",
                p.queryPred("f", List.of(1, new Variable("result"))).results().equals(List.of(Map.of("result", 1))));
    }

    public void testExternalIsa() throws Exception {
        p.loadStr("f(a: MyClass, x) := x = a.id;");
        List<HashMap<String, Object>> result = p.queryPred("f", List.of(new MyClass("test", 1), new Variable("x")))
                .results();
        assertTrue(result.equals(List.of(Map.of("x", 1))));
        p.clear();

        p.loadStr("f(a: MySubClass, x) := x = a.id;");
        result = p.queryPred("f", List.of(new MyClass("test", 1), new Variable("x"))).results();
        assertTrue("Failed to filter rules by specializers.", result.isEmpty());
        p.clear();

        p.loadStr("f(a: OtherClass, x) := x = a.id;");
        assertThrows(Exceptions.UnregisteredClassError.class,
                () -> p.queryPred("f", List.of(new MyClass("test", 1), new Variable("x"))).results());
    }

    public void testExternalIsSubSpecializer() throws Exception {
        p.loadStr("f(a: MySubClass, x) := x = 1;");
        p.loadStr("f(a: MyClass, x) := x = 2;");
        List<HashMap<String, Object>> result = p.queryPred("f", List.of(new MySubClass("test", 1), new Variable("x")))
                .results();
        assertTrue("Failed to order rules based on specializers.",
                result.equals(List.of(Map.of("x", 1), Map.of("x", 2))));

        result = p.queryPred("f", List.of(new MyClass("test", 1), new Variable("x"))).results();
        assertTrue("Failed to order rules based on specializers.", result.equals(List.of(Map.of("x", 2))));
    }

    public void testReturnListFromCall() throws Exception {
        p.loadStr("test(c: MyClass) := \"hello\" in c.myList;");
        MyClass c = new MyClass("test", 1);
        assertFalse(p.queryPred("test", List.of(c)).results().isEmpty());
    }

    /*** TEST FFI CONVERSIONS ***/

    public void testBoolFFIRoundTrip() throws Exception {
        Boolean b = true;
        JSONObject polar = p.toPolarTerm(b);
        Object java = p.toJava(polar);
        assertEquals(b, java);
    }

    public void testIntFFIRoundTrip() throws Exception {
        int i = 3;
        JSONObject polar = p.toPolarTerm(i);
        Object java = p.toJava(polar);
        assertEquals(i, java);
    }

    public void testFloatFFIRoundTrip() throws Exception {
        float f = (float) 3.50;
        JSONObject polar = p.toPolarTerm(f);
        Object java = p.toJava(polar);
        assertEquals(f, java);
    }

    public void testListFFIRoundTrip() throws Exception {
        List<Integer> l = List.of(1, 2, 3, 4);
        JSONObject polar = p.toPolarTerm(l);
        Object java = p.toJava(polar);
        assertEquals(l, java);
    }

    public void testDictFFIRoundTrip() throws Exception {
        Map<String, Integer> m = Map.of("a", 1, "b", 2);
        JSONObject polar = p.toPolarTerm(m);
        Object java = p.toJava(polar);
        assertEquals(m, java);
    }

    public void testJavaClassFFIRoundTrip() throws Exception {
        MyClass instance = new MyClass("test", 1);
        JSONObject polar = p.toPolarTerm(instance);
        Object java = p.toJava(polar);
        assertEquals(instance, java);
    }

    public void testPredicateFFIRoundTrip() throws Exception {
        Predicate pred = new Predicate("name", List.of(1, "hello"));
        JSONObject polar = p.toPolarTerm(pred);
        Object java = p.toJava(polar);
        assertEquals(pred, java);
    }

    public void testReturnRubyInstanceFromCall() throws Exception {
        MyClass c = new MyClass("test", 1);
        p.loadStr("test(c: MyClass) := x = c.mySubClass(c.name, c.id), x.id = c.id;");
        assertFalse(p.queryPred("test", List.of(c)).results().isEmpty());
    }

    public void testEnumerationCallResults() throws Exception {
        MyClass c = new MyClass("test", 1);
        p.loadStr("test(c: MyClass, x) := x = c.myEnumeration;");
        List<HashMap<String, Object>> results = p.queryPred("test", List.of(c, new Variable("x"))).results();
        assertTrue(results.equals(List.of(Map.of("x", "hello"), Map.of("x", "world"))));
    }

    /*** TEST EXTERNALS ***/

    public void testRegisterAndMakeClass() throws Exception {
        Map<String, Object> testArg = Map.of("name", "testName", "id", 1);
        MyClass instance = (MyClass) p.makeInstance("MyClass", testArg, Long.valueOf(0));
        assertEquals("testName", instance.name);
        assertEquals(Integer.valueOf(1), instance.id);
        // TODO: test that errors when given invalid constructor
        // TODO: test that errors when registering same class twice
        // TODO: test that errors if same alias used twice
        // TODO: test inheritance
    }

    public void testDuplicateRegistration() throws Exception {
        assertThrows(Exceptions.DuplicateClassAliasError.class, () -> p.registerClass(MyClass.class,
                m -> new MyClass((String) m.get("name"), (int) m.get("id")), "MyClass"));
    }

    public void testMakeInstanceFromPolar() throws Exception {
        p.loadStr("f(x) := x = new MyClass{name: \"test\", id: 1};");
        Query query = p.queryStr("f(x)");
        MyClass ret = (MyClass) query.nextElement().get("x");
        assertEquals("test", ret.name);
        assertEquals(Integer.valueOf(1), ret.id);
    }

    public void testRegisterCall() throws Exception {
        MyClass instance = new MyClass("test", 1);
        p.cacheInstance(instance, Long.valueOf(1));
        p.registerCall("myMethod", List.of("hello world"), 1, 1);
        JSONObject res = p.nextCallResult(1);
        assertTrue(p.toJava(res).equals("hello world"));
    }

    public void testExternalCall() throws Exception {
        // Test get attribute
        p.loadStr("id(x) := x = new MyClass{name: \"test\", id: 1}.id;");
        assertTrue("Failed to get attribute on external instance.",
                p.queryStr("id(x)").results().equals(List.of(Map.of("x", 1))));

        // Test call method
        p.loadStr("method(x) := x = new MyClass{name: \"test\", id: 1}.myMethod(\"hello world\");");
        assertTrue("Failed to get attribute on external instance.",
                p.queryStr("method(x)").results().equals(List.of(Map.of("x", "hello world"))));
    }

    /**** TEST LOADING ****/

    // TODO: test parsing errors
    // it 'raises on IntegerOverflow errors' do
    // int = '18446744073709551616'
    // rule = <<~POLAR
    // f(a) := a = #{int};
    // POLAR
    // expect { subject.load_str(rule) }.to raise_error do |e|
    // expect(e).to be_an Oso::Polar::ParseError::IntegerOverflow
    // expect(e.message).to eq("'18446744073709551616' caused an integer overflow at
    // line 1, column 13")
    // end
    // end

    // it 'raises on InvalidTokenCharacter errors' do
    // rule = <<~POLAR
    // f(a) := a = "this is not
    // allowed";
    // POLAR
    // expect { subject.load_str(rule) }.to raise_error do |e|
    // expect(e).to be_an Oso::Polar::ParseError::InvalidTokenCharacter
    // expect(e.message).to eq("'\\n' is not a valid character. Found in this is not
    // at line 1, column 25")
    // end
    // end
    // it 'raises on UnrecognizedToken errors' do
    // rule = <<~POLAR
    // 1;
    // POLAR
    // expect { subject.load_str(rule) }.to raise_error do |e|
    // expect(e).to be_an Oso::Polar::ParseError::UnrecognizedToken
    // expect(e.message).to eq("did not expect to find the token '1' at line 1,
    // column 1")
    // end
    // end

    public void testLoadFile() throws Exception {
        p.loadFile("src/test/java/com/osohq/oso/test.polar");
        assertTrue(p.queryStr("f(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))));
    }

    public void testLoadNonPolarFile() throws Exception {
        assertThrows("Failed to catch incorrect Polar file extension.", Exceptions.PolarFileExtensionError.class,
                () -> p.loadFile("wrong.txt"));
    }

    public void testLoadFilePassesFilename() throws Exception {
        File tempFile = File.createTempFile("error-", ".polar");
        FileWriter w = new FileWriter(tempFile);
        w.write(";");
        w.close();
        p.loadFile(tempFile.getPath());
        assertThrows("Failed to pass filename across FFI boundary.", Exceptions.ParseError.class,
                () -> p.queryStr("f(1)"));
        tempFile.deleteOnExit();
    }

    public void testLoadFileIdempotent() throws Exception {
        p.loadFile("src/test/java/com/osohq/oso/test.polar");
        p.loadFile("src/test/java/com/osohq/oso/test.polar");
        assertTrue("loadFile behavior is not idempotent.",
                p.queryStr("f(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))));
    }

    public void testLoadMultipleFiles() throws Exception {
        p.loadFile("src/test/java/com/osohq/oso/test.polar");
        p.loadFile("src/test/java/com/osohq/oso/test2.polar");
        assertTrue(p.queryStr("f(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))));
        assertTrue(p.queryStr("g(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))));
    }

    // TODO
    public void testClear() throws Exception {
        // subject.load_file(test_file)
        // expect(qvar(subject, 'f(x)', 'x')).to eq([1, 2, 3])
        // subject.clear
        // expect(query(subject, 'f(x)')).to eq([])

    }
}
