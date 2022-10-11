package com.osohq.oso;

import static com.osohq.oso.Operator.Dot;
import static com.osohq.oso.Operator.Isa;
import static com.osohq.oso.Operator.Unify;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.File;
import java.io.FileWriter;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.Enumeration;
import java.util.HashMap;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import org.json.JSONObject;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

public class PolarTest {
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

    public static String myStaticMethod() {
      return "hello world";
    }

    public String myReturnNull() {
      return null;
    }

    @Override
    public boolean equals(Object obj) {
      return obj instanceof MyClass
          && ((MyClass) obj).name.equals(this.name)
          && ((MyClass) obj).id.equals(this.id);
    }

    @Override
    public int hashCode() {
      return this.id;
    }
  }

  public static class MySubClass extends MyClass {
    public MySubClass(String name, Integer id) {
      super(name, id);
    }
  }

  @BeforeEach
  public void setUp() throws NoSuchMethodException {
    try {
      p = new Polar();
      p.registerClass(MyClass.class, "MyClass");
      p.registerClass(MySubClass.class, "MySubClass");
    } catch (Exceptions.OsoException e) {
      throw new Error(e);
    }
  }

  /** Rigorous Test :-) */
  @Test
  public void testApp() {
    assertTrue(true);
  }

  /**** TEST QUERY ****/

  @Test
  public void testLoadAndQueryStr() throws Exception {
    p.loadStr("f(1);");
    Query query = p.query("f(x)");
    assertEquals(List.of(Map.of("x", 1)), query.results());
  }

  @Test
  public void testInlineQueries() throws Exception {
    p.loadStr("f(1); ?= f(1);");
    p.clearRules();
    assertThrows(
        Exceptions.InlineQueryFailedError.class,
        () -> p.loadStr("f(1); ?= f(2);"),
        "Expected inline query to fail but it didn't.");
  }

  @Test
  public void testBasicQueryPred() throws Exception {
    // test basic query
    p.loadStr("f(a, b) if a = b;");
    assertFalse(p.queryRule("f", 1, 1).results().isEmpty(), "Basic predicate query failed.");
    assertTrue(
        p.queryRule("f", 1, 2).results().isEmpty(),
        "Basic predicate query expected to fail but didn't.");
  }

  @Test
  public void testQueryPredWithObject() throws Exception {
    // test query with Java Object
    p.loadStr("g(x) if x.id = 1;");
    assertFalse(
        p.queryRule("g", new MyClass("test", 1)).results().isEmpty(),
        "Predicate query with Java Object failed.");
    assertTrue(
        p.queryRule("g", new MyClass("test", 2)).results().isEmpty(),
        "Predicate query with Java Object expected to fail but didn't.");
  }

  @Test
  public void testQueryPredWithVariable() throws Exception {
    // test query with Variable
    p.loadStr("f(a, b) if a = b;");
    assertTrue(
        p.queryRule("f", 1, new Variable("result")).results().equals(List.of(Map.of("result", 1))),
        "Predicate query with Variable failed.");
  }

  /*** TEST FFI CONVERSIONS ***/

  @Test
  public void testBoolFFIRoundTrip() throws Exception {
    Boolean b = true;
    JSONObject polar = p.host.toPolarTerm(b);
    Object java = p.host.toJava(polar);
    assertEquals(b, java);
  }

  @Test
  public void testIntFFIRoundTrip() throws Exception {
    int i = 3;
    JSONObject polar = p.host.toPolarTerm(i);
    Object java = p.host.toJava(polar);
    assertEquals(i, java);
  }

  @Test
  public void testFloatFFIRoundTrip() throws Exception {
    double f = 3.50;
    JSONObject polar = p.host.toPolarTerm(f);
    Object java = p.host.toJava(polar);
    assertEquals(f, java);
  }

  @Test
  public void testListFFIRoundTrip() throws Exception {
    List<Integer> l = List.of(1, 2, 3, 4);
    JSONObject polar = p.host.toPolarTerm(l);
    Object java = p.host.toJava(polar);
    assertEquals(l, java);
  }

  @Test
  public void testArrayFFIRoundTrip() throws Exception {
    int[] a1 = {1, 2, 3, 4};
    JSONObject polar = p.host.toPolarTerm(a1);
    Object java = p.host.toJava(polar);
    assertEquals(List.of(1, 2, 3, 4), java);

    double[] a2 = {1.2, 3.5};
    polar = p.host.toPolarTerm(a2);
    java = p.host.toJava(polar);

    assertEquals(List.of(1.2, 3.5), java);

    String[] a3 = {"hello", "world"};
    polar = p.host.toPolarTerm(a3);
    java = p.host.toJava(polar);
    assertEquals(List.of("hello", "world"), java);
  }

  @Test
  public void testDictFFIRoundTrip() throws Exception {
    Map<String, Integer> m = Map.of("a", 1, "b", 2);
    JSONObject polar = p.host.toPolarTerm(m);
    Object java = p.host.toJava(polar);
    assertEquals(m, java);
  }

  @Test
  public void testJavaClassFFIRoundTrip() throws Exception {
    MyClass instance = new MyClass("test", 1);
    JSONObject polar = p.host.toPolarTerm(instance);
    Object java = p.host.toJava(polar);
    assertEquals(instance, java);
  }

  @Test
  public void testPredicateFFIRoundTrip() throws Exception {
    Predicate pred = new Predicate("name", List.of(1, "hello"));
    JSONObject polar = p.host.toPolarTerm(pred);
    Object java = p.host.toJava(polar);
    assertEquals(pred, java);
  }

  @Test
  public void testNaN() throws Exception {
    p.registerConstant(Double.NaN, "nan");

    List<HashMap<String, Object>> results = p.query("x = nan").results();
    HashMap<String, Object> result = results.get(0);
    Object x = result.get("x");
    assertTrue(x instanceof Double);
    Double y = (Double) x;
    assertTrue(Double.isNaN(y));

    assertTrue(p.query("nan = nan").results().isEmpty(), "NaN != NaN");
  }

  @Test
  public void testInfinities() throws Exception {
    p.registerConstant(Double.POSITIVE_INFINITY, "inf");

    List<HashMap<String, Object>> inf_results = p.query("x = inf").results();
    HashMap<String, Object> inf_result = inf_results.get(0);
    Object inf = inf_result.get("x");
    assertTrue((Double) inf == Double.POSITIVE_INFINITY);

    assertFalse(p.query("inf = inf").results().isEmpty(), "Infinity == Infinity");

    p.registerConstant(Double.NEGATIVE_INFINITY, "neg_inf");

    List<HashMap<String, Object>> neg_inf_results = p.query("x = neg_inf").results();
    HashMap<String, Object> neg_inf_result = neg_inf_results.get(0);
    Object neg_inf = neg_inf_result.get("x");
    assertTrue((Double) neg_inf == Double.NEGATIVE_INFINITY);

    assertFalse(p.query("neg_inf = neg_inf").results().isEmpty(), "-Infinity == -Infinity");

    assertTrue(p.query("inf = neg_inf").results().isEmpty(), "Infinity != -Infinity");
    assertTrue(p.query("inf < neg_inf").results().isEmpty(), "Infinity > -Infinity");
    assertFalse(p.query("neg_inf < inf").results().isEmpty(), "-Infinity < Infinity");
  }

  @Test
  // test_nil
  public void testNil() throws Exception {
    p.loadStr("null(nil);");

    // Map.of() can't handle a null value.
    HashMap<String, Object> expected = new HashMap<String, Object>();
    expected.put("x", null);
    assertEquals(p.query("null(x)").results(), List.of(expected));
    assertTrue(p.queryRule("null", (Object) null).results().equals(List.of(Map.of())));
    assertTrue(p.queryRule("null", List.of()).results().isEmpty());
  }

  /*** TEST EXTERNALS ***/

  @Test
  public void testRegisterAndMakeClass() throws Exception {
    MyClass instance =
        (MyClass) p.host.makeInstance("MyClass", Arrays.asList("testName", 1), Long.valueOf(0));
    assertEquals("testName", instance.name);
    assertEquals(Integer.valueOf(1), instance.id);
    // TODO: test that errors when given invalid constructor
    // TODO: test that errors when registering same class twice
    // TODO: test that errors if same alias used twice
    // TODO: test inheritance
  }

  @Test
  public void testDuplicateRegistration() throws Exception {
    assertThrows(
        Exceptions.DuplicateClassAliasError.class, () -> p.registerClass(MyClass.class, "MyClass"));
  }

  @Test
  public void testMakeInstanceFromPolar() throws Exception {
    p.loadStr("f(x) if x = new MyClass(\"test\", 1);");
    Query query = p.query("f(x)");
    MyClass ret = (MyClass) query.nextElement().get("x");
    assertEquals("test", ret.name);
    assertEquals(Integer.valueOf(1), ret.id);
  }

  @Test
  public void testNoKeywordArgs() throws Exception {
    p.registerConstant(true, "MyClass");
    assertThrows(
        Exceptions.InstantiationError.class, () -> p.query("x = new MyClass(\"test\", id: 1)"));
    assertThrows(
        Exceptions.InvalidCallError.class,
        () -> p.query("x = (new MyClass(\"test\", 1)).foo(\"test\", id: 1)"));
  }

  @Test
  public void testExternalCall() throws Exception {
    // Test get attribute
    p.loadStr("id(x) if x = new MyClass(\"test\", 1).id;");
    assertTrue(
        p.query("id(x)").results().equals(List.of(Map.of("x", 1))),
        "Failed to get attribute on external instance.");

    p.clearRules();

    // Test call method
    p.loadStr("method(x) if x = new MyClass(\"test\", 1).myMethod(\"hello world\");");
    assertTrue(
        p.query("method(x)").results().equals(List.of(Map.of("x", "hello world"))),
        "Failed to get attribute on external instance.");
  }

  @Test
  public void testReturnJavaInstanceFromCall() throws Exception {
    MyClass c = new MyClass("test", 1);
    p.loadStr("test(c: MyClass) if x = c.mySubClass(c.name, c.id) and x.id = c.id;");
    assertFalse(p.queryRule("test", c).results().isEmpty());
  }

  @Test
  public void testEnumerationCallResults() throws Exception {
    MyClass c = new MyClass("test", 1);
    p.loadStr("test(c: MyClass, x) if x in c.myEnumeration();");
    List<HashMap<String, Object>> results = p.queryRule("test", c, new Variable("x")).results();
    assertTrue(results.equals(List.of(Map.of("x", "hello"), Map.of("x", "world"))));
  }

  @Test
  public void testStringMethods() throws Exception {
    p.loadStr("f(x) if x.length() = 3;");
    assertFalse(p.query("f(\"oso\")").results().isEmpty());
    assertTrue(p.query("f(\"notoso\")").results().isEmpty());
  }

  @Test
  public void testListMethods() throws Exception {
    p.loadStr("f(x) if x.size() = 3;");
    assertFalse(p.queryRule("f", new ArrayList(Arrays.asList(1, 2, 3))).results().isEmpty());
    assertTrue(p.queryRule("f", new ArrayList(Arrays.asList(1, 2, 3, 4))).results().isEmpty());

    assertFalse(p.queryRule("f", new int[] {1, 2, 3}).results().isEmpty());
    assertTrue(p.queryRule("f", new int[] {1, 2, 3, 4}).results().isEmpty());
  }

  @Test
  public void testExternalIsa() throws Exception {
    p.loadStr("f(a: MyClass, x) if x = a.id;");
    List<HashMap<String, Object>> result =
        p.queryRule("f", new MyClass("test", 1), new Variable("x")).results();
    assertTrue(result.equals(List.of(Map.of("x", 1))));
    p.clearRules();

    p.loadStr("f(a: MySubClass, x) if x = a.id;");
    result = p.queryRule("f", new MyClass("test", 1), new Variable("x")).results();
    assertTrue(result.isEmpty(), "Failed to filter rules by specializers.");
    p.clearRules();

    p.loadStr("f(a: OtherClass, x) if x = a.id;");
    assertThrows(
        Exceptions.UnregisteredClassError.class,
        () -> p.queryRule("f", new MyClass("test", 1), new Variable("x")).results());
  }

  @Test
  public void testExternalIsSubSpecializer() throws Exception {
    String policy = "f(_: MySubClass, x) if x = 1;\n" + "f(_: MyClass, x) if x = 2;";
    p.loadStr(policy);
    List<HashMap<String, Object>> result =
        p.queryRule("f", new MySubClass("test", 1), new Variable("x")).results();
    assertTrue(
        result.equals(List.of(Map.of("x", 1), Map.of("x", 2))),
        "Failed to order rules based on specializers.");

    result = p.queryRule("f", new MyClass("test", 1), new Variable("x")).results();
    assertTrue(
        result.equals(List.of(Map.of("x", 2))), "Failed to order rules based on specializers.");
  }

  @Test
  public void testExternalUnify() throws Exception {
    assertFalse(p.query("new MyClass(\"foo\", 1) = new MyClass(\"foo\", 1)").results().isEmpty());
    assertTrue(p.query("new MyClass(\"foo\", 1) = new MyClass(\"foo\", 2)").results().isEmpty());
    assertTrue(p.query("new MyClass(\"foo\", 1) = new MyClass(\"bar\", 1)").results().isEmpty());
    assertTrue(p.query("new MyClass(\"foo\", 1) = {foo: 1}").results().isEmpty());
  }

  @Test
  public void testExternalInternalUnify() throws Exception {
    assertFalse(p.query("new String(\"foo\") = \"foo\"").results().isEmpty());
  }

  @Test
  public void testReturnListFromCall() throws Exception {
    p.loadStr("test(c: MyClass) if \"hello\" in c.myList();");
    MyClass c = new MyClass("test", 1);
    assertFalse(p.queryRule("test", c).results().isEmpty());
  }

  @Test
  public void testClassMethods() throws Exception {
    p.loadStr("test(x) if x=1 and MyClass.myStaticMethod() = \"hello world\";");

    assertFalse(p.query("test(1)").results().isEmpty());
  }

  @Test
  public void testExternalOp() throws Exception {
    assertFalse(p.query("new String(\"foo\") == new String(\"foo\")").results().isEmpty());
  }

  /**** TEST PARSING ****/

  @Test
  public void testIntegerOverFlowError() throws Exception {
    String rule = "f(x) if x = 18446744073709551616;";
    Exceptions.IntegerOverflow e =
        assertThrows(Exceptions.IntegerOverflow.class, () -> p.loadStr(rule));
    assertTrue(
        e.getMessage()
            .startsWith("'18446744073709551616' caused an integer overflow at line 1, column 13"));
  }

  @Test
  public void testInvalidTokenCharacter() throws Exception {
    String rule = "f(x) if x = \"This is not\n allowed\"";
    Exceptions.InvalidTokenCharacter e =
        assertThrows(Exceptions.InvalidTokenCharacter.class, () -> p.loadStr(rule));
    // TODO: this is a wacky message
    assertTrue(
        e.getMessage()
            .startsWith(
                "'\\n' is not a valid character. Found in This is not at line 1, column 25"));
  }

  @Test
  public void testUnrecognizedTokenError() throws Exception {
    String rule = "1";
    Exceptions.UnrecognizedToken e =
        assertThrows(Exceptions.UnrecognizedToken.class, () -> p.loadStr(rule));
    assertTrue(
        e.getMessage().startsWith("did not expect to find the token '1' at line 1, column 1"));
  }

  /**** TEST LOADING ****/

  @Test
  public void testLoadFile() throws Exception {
    p.loadFile("src/test/java/com/osohq/oso/test.polar");
    assertTrue(
        p.query("f(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))));
  }

  @Test
  public void testLoadNonPolarFile() throws Exception {
    assertThrows(
        Exceptions.PolarFileExtensionError.class,
        () -> p.loadFile("wrong.txt"),
        "Failed to catch incorrect Polar file extension.");
  }

  @Test
  public void testLoadFilePassesFilename() throws Exception {
    File tempFile = File.createTempFile("error-", ".polar");
    FileWriter w = new FileWriter(tempFile);
    w.write(";");
    w.close();
    assertThrows(
        Exceptions.ParseError.class,
        () -> p.loadFile(tempFile.getPath()),
        "Failed to pass filename across FFI boundary.");
    tempFile.deleteOnExit();
  }

  @Test
  public void testLoadFileIdempotent() throws Exception {
    p.loadFile("src/test/java/com/osohq/oso/test.polar");
    assertThrows(
        Exceptions.PolarRuntimeException.class,
        () -> p.loadFile("src/test/java/com/osohq/oso/test.polar"));
    assertTrue(
        p.query("f(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))),
        "loadFile behavior is not idempotent.");
  }

  @Test
  public void testLoadMultipleFiles() throws Exception {
    p.loadFiles(
        new String[] {
          "src/test/java/com/osohq/oso/test.polar", "src/test/java/com/osohq/oso/test2.polar"
        });
    assertTrue(
        p.query("f(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))));
    assertTrue(
        p.query("g(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))));
  }

  // test_load_multiple_files_same_name_different_path
  @Test
  public void testLoadMultipleFilesSameNameDifferentPath() throws Exception {
    p.loadFiles(
        new String[] {
          "src/test/java/com/osohq/oso/test.polar", "src/test/java/com/osohq/oso/other/test.polar"
        });
    assertTrue(
        p.query("f(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))));
    assertTrue(
        p.query("g(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))));
  }

  @Test
  @DisplayName("testLoadFilesFromResources loads multiple files from resources folder")
  public void testLoadFilesFromResources() throws Exception {
    p.loadFilesFromResources("/test.polar", "/test2.polar");
    assertTrue(
        p.query("f(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))));
    assertTrue(
        p.query("g(x)").results().equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3))));
  }

  @Test
  @DisplayName("testLoadFilesFromResources throws exception when tries loading missing file")
  public void testLoadFilesFromResourcesTriesLoadingMissingFile() throws Exception {
    assertThrows(
        Exceptions.PolarFileNotFoundError.class, () -> p.loadFilesFromResources("/missing.polar"));
  }

  @Test
  @DisplayName(
      "testLoadFilesFromResources throws exception when tries loading file with non-polar"
          + " extension")
  public void testLoadFilesFromResourcesTriesLoadingNonPolarFile() throws Exception {
    assertThrows(
        Exceptions.PolarFileExtensionError.class, () -> p.loadFilesFromResources("/test.file"));
  }

  @Test
  public void testClearRules() throws Exception {
    p.loadFile("src/test/java/com/osohq/oso/test.polar");
    assertEquals(
        List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3)), p.query("f(x)").results());
    p.clearRules();
    assertThrows(
        Exceptions.PolarRuntimeException.class,
        () -> p.query("f(x)"),
        "Query for undefined rule `f`");

    // make sure classes are still registered
    assertFalse(p.query("x = new MyClass(\"test\", 1)").results().isEmpty());
  }

  public static class Foo {
    public String foo;

    public Foo() {
      this.foo = "foo";
    }
  }

  @Test
  public void testLookupErrors() throws Exception {
    p.registerClass(Foo.class, "Foo");
    assertEquals(List.of(), p.query("new Foo() = {bar: \"bar\"}").results());
    assertThrows(
        Exceptions.PolarRuntimeException.class,
        () -> p.query("new Foo().bar = \"bar\""),
        "Expected error.");
  }

  @Test
  public void testUnboundVariable() throws Exception {
    p.loadStr("rule(_x, y) if y = 1;");
    List<HashMap<String, Object>> results = p.query("rule(x, y)").results();
    HashMap<String, Object> result = results.get(0);
    assertTrue(result.get("x") instanceof Variable);
    assertEquals(result.get("y"), 1);
  }

  @Test
  public void testReturnNull() throws Exception {
    p.loadStr("f(x) if x.myReturnNull() = nil;");
    assertFalse(p.queryRule("f", new MyClass("test", 1)).results().isEmpty());

    p.clearRules();

    p.loadStr("g(x) if x.myReturnNull().badCall() = 1;");
    assertThrows(
        NullPointerException.class, () -> p.queryRule("g", new MyClass("test", 1)).results());
  }

  /*** TEST OSO ***/
  public static class NotIterable {
    public NotIterable() {}
  }

  public static class BarIterator implements Iterable<Integer> {
    private List<Integer> list;

    public BarIterator(List<Integer> list) {
      this.list = list;
    }

    // code for data structure
    public Integer sum() {
      int count = 0;
      for (int i : list) {
        count += i;
      }
      return count;
    }

    // code for data structure
    public Iterator<Integer> iterator() {
      return list.iterator();
    }
  }

  @Test
  public void testIterators() throws Exception {
    // builtins sort of work for Java
    p.query("d = {a: 1, b: 2} and x in d.entrySet() and x in d")
        .results()
        .equals(List.of(Map.of("x", "a"), Map.of("x", "b")));

    // non iterables throw exception
    p.registerClass(NotIterable.class, "NotIterable");
    assertThrows(
        Exceptions.InvalidIteratorError.class, () -> p.query("x in new NotIterable()").results());

    // custom iterators work
    p.registerClass(BarIterator.class, "BarIterator");
    p.query("x in new BarIterator([1, 2, 3])")
        .results()
        .equals(List.of(Map.of("x", 1), Map.of("x", 2), Map.of("x", 3)));
    p.query("x = new BarIterator([1, 2, 3]).sum()").results().equals(List.of(Map.of("x", 6)));
  }

  @Test
  public void testExpressionGt() throws Exception {
    // GIVEN
    p.loadStr("f(x) if x > 2;");
    // WHEN
    List<HashMap<String, Object>> res = p.query("f(x)", true).results();
    // THEN
    assertEquals(1, res.size());
    HashMap<String, Object> hm = res.get(0);
    assertEquals(1, hm.size());
    Object expr = hm.get("x");
    assertEquals(
        new Expression(
            Operator.And, List.of(new Expression(Operator.Gt, List.of(new Variable("_this"), 2)))),
        expr);
  }

  @Test
  public void testPartialUnification() throws Exception {
    // GIVEN
    p.loadStr("f(x, y) if x = y;");
    Variable x = new Variable("x");
    Variable y = new Variable("y");

    // WHEN
    List<HashMap<String, Object>> results = p.queryRule("f", x, y).results();

    // THEN
    assertEquals(1, results.size());
    HashMap<String, Object> bindings = results.get(0);
    assertEquals(bindings.get("x"), y);
    assertEquals(bindings.get("y"), x);
  }

  @Test
  public void testPartial() {
    // GIVEN
    p.loadStr("f(1); f(x) if x = 1 and x = 2;");

    // WHEN
    Predicate rule = new Predicate("f", List.of(new Variable("x")));
    List<HashMap<String, Object>> results = p.query(rule, true).results();
    assertEquals(1, results.size());
    assertEquals(1, results.get(0).get("x"));

    p.clearRules();

    p.loadStr("g(x) if x.bar = 1 and x.baz = 2;");

    Predicate gRule = new Predicate("g", List.of(new Variable("x")));
    results = p.query(gRule, true).results();
    assertEquals(1, results.size());
    Expression expr = (Expression) results.get(0).get("x");
    List<Object> args = (List<Object>) unwrapAnd(expr);
    assertEquals(2, args.size());
    assertEquals(
        args.get(0),
        new Expression(
            Unify, List.of(1, new Expression(Dot, List.of(new Variable("_this"), "bar")))));
    assertEquals(
        args.get(1),
        new Expression(
            Unify, List.of(2, new Expression(Dot, List.of(new Variable("_this"), "baz")))));
  }

  private Object unwrapAnd(Expression expression) {
    assertEquals(expression.getOperator(), Operator.And);
    if (expression.getArgs().size() == 1) {
      return expression.getArgs().get(0);
    } else {
      return expression.getArgs();
    }
  }

  public static class User {}

  public static class Post {}

  @Test
  public void testPartialConstraint() {
    p.registerClass(User.class, "User");
    p.registerClass(Post.class, "Post");
    p.loadStr("f(x: User) if x.user = 1; f(x: Post) if x.post = 1;");

    Variable x = new Variable("x");
    Predicate rule = new Predicate("f", List.of(x));
    List<HashMap<String, Object>> results =
        p.query(rule, Map.of("x", new TypeConstraint(x, "User")), true).results();

    assertEquals(1, results.size());

    List<Object> andArgs = (List<Object>) unwrapAnd((Expression) results.get(0).get("x"));
    assertEquals(2, andArgs.size());
    assertEquals(
        new Expression(Isa, List.of(new Variable("_this"), new Pattern("User", new HashMap<>()))),
        andArgs.get(0));
    assertEquals(
        new Expression(
            Unify, List.of(1, new Expression(Dot, List.of(new Variable("_this"), "user")))),
        andArgs.get(1));
  }

  @Test
  public void testUnexpectedExpression() {
    p.loadStr("f(x) if x > 2;");

    assertThrows(
        Exceptions.UnexpectedPolarTypeError.class,
        () -> p.query("f(x)"),
        "Expected inline query to fail but it didn't.");
  }

  public static class Bar extends Foo {}

  public static class Baz extends Bar {}

  public static class Bad {}

  @Test
  public void testRuleTypes() {
    // NOTE: keep this order of registering classes--confirms that MROs are added at the correct
    // time
    p.registerClass(Baz.class, "Baz");
    p.registerClass(Bar.class, "Bar");
    p.registerClass(Foo.class, "Foo");
    p.registerClass(Bad.class, "Bad");

    final String policy1 =
        "type f(_x: Integer);"
            + "f(1);"
            + "type f(_x: Foo);"
            + "type f(_x: Foo, _y: Bar);"
            + "f(_x: Bar);"
            + "f(_x: Baz);";

    p.loadStr(policy1);

    p.clearRules();

    assertThrows(
        Exceptions.ValidationError.class,
        () -> p.loadStr(policy1 + "f(_x: Bad);"),
        "Expected rule type validation error.");

    //  Test with fields
    final String policy2 = "type f(_x: Foo{id: 1});" + "f(_x: Bar{id: 1});" + "f(_x: Baz{id: 1});";

    p.loadStr(policy2);

    p.clearRules();

    assertThrows(
        Exceptions.ValidationError.class,
        () -> p.loadStr(policy2 + "f(_x: Baz);"),
        "Expected rule type validation error.");

    // Test invalid rule type
    assertThrows(
        Exceptions.ValidationError.class,
        () -> p.loadStr("type f(x: Foo, x.baz);"),
        "Expected rule type validation error.");
  }
}
