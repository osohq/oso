import com.osohq.oso.*;
import java.io.IOException;
import java.util.*;

class Test {
  public static class A {
    public String x;

    public A(String x) {
      this.x = x;
    }

    public int foo() {
      return -1;
    }
  }

  public static class D extends A {
    public D(String x) {
      super(x);
    }
  }

  public static class BC {
    public String y;

    public BC(String y) {
      this.y = y;
    }

    public int foo() {
      return -1;
    }
  }

  public static class E {
    public static int sum(List<Integer> args) {
      int sum = 0;
      for (int arg : args) {
        sum += arg;
      }
      return sum;
    }
  }

  public static void main(String[] args)
      throws IOException, NoSuchMethodException, Exceptions.OsoException {
    Oso o = new Oso();
    o.registerClass(A.class, "A");
    o.registerClass(BC.class, "C");
    o.registerClass(E.class, "E");
    o.loadFile("test.polar");
    assert o.isAllowed("a", "b", "c");

    // Test that a built in string method can be called.
    o.loadStr("?= x = \"hello world!\" and x.endsWith(\"world!\");");

    // Test that a custom error type is thrown.
    boolean throwsException = false;
    try {
      o.loadStr("missingSemicolon()");
    } catch (Exceptions.UnrecognizedEOF e) {
      throwsException = true;
      assert e.getMessage()
          .equals(
              "hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column"
                  + " 19");
    }
    assert throwsException;

    assert !o.queryRule("specializers", new D("hello"), new BC("hello")).results().isEmpty();
    assert !o.queryRule("floatLists").results().isEmpty()
        && !o.queryRule("intDicts").results().isEmpty();
    assert !o.queryRule("comparisons").results().isEmpty()
        && !o.queryRule("testForall").results().isEmpty();
    assert !o.queryRule("testRest").results().isEmpty();
    assert !o.queryRule("testMatches", new A("hello")).results().isEmpty();
    assert !o.queryRule("testMethodCalls", new A("hello"), new BC("hello")).results().isEmpty();
    assert !o.queryRule("testOr").results().isEmpty();

    // Test that cut doesn't return anything.
    assert o.queryRule("testCut").results().isEmpty();

    // Test we can unify against a class
    assert !o.queryRule("testUnifyClass", A.class).results().isEmpty();

    // Test that a constant can be called.
    o.registerConstant(Math.class, "MyMath");
    o.loadStr("?= MyMath.PI == 3.141592653589793;");

    // Test built-in type specializers.
    assert !o.query("builtinSpecializers(true, \"Boolean\")").results().isEmpty();
    assert o.query("builtinSpecializers(false, \"Boolean\")").results().isEmpty();
    assert !o.query("builtinSpecializers(2, \"Integer\")").results().isEmpty();
    assert !o.query("builtinSpecializers(1, \"Integer\")").results().isEmpty();
    assert o.query("builtinSpecializers(0, \"Integer\")").results().isEmpty();
    assert o.query("builtinSpecializers(-1, \"Integer\")").results().isEmpty();
    assert !o.query("builtinSpecializers(1.0, \"Float\")").results().isEmpty();
    assert o.query("builtinSpecializers(0.0, \"Float\")").results().isEmpty();
    assert o.query("builtinSpecializers(-1.0, \"Float\")").results().isEmpty();
    assert !o.query("builtinSpecializers([\"foo\", \"bar\", \"baz\"], \"List\")")
        .results()
        .isEmpty();
    assert o.query("builtinSpecializers([\"bar\", \"foo\", \"baz\"], \"List\")")
        .results()
        .isEmpty();
    assert !o.query("builtinSpecializers({foo: \"foo\"}, \"Dictionary\")").results().isEmpty();
    assert o.query("builtinSpecializers({foo: \"bar\"}, \"Dictionary\")").results().isEmpty();
    assert !o.query("builtinSpecializers(\"foo\", \"String\")").results().isEmpty();
    assert o.query("builtinSpecializers(\"bar\", \"String\")").results().isEmpty();

    // Java integers do not have the denominator field
    // assert !o.query("builtinSpecializers(1,
    // \"IntegerWithFields\")").results().isEmpty();
    assert o.query("builtinSpecializers(2, \"IntegerWithGarbageFields\")").results().isEmpty();
    assert o.query("builtinSpecializers({}, \"DictionaryWithFields\")").results().isEmpty();
    assert o.query("builtinSpecializers({z: 1}, \"DictionaryWithFields\")").results().isEmpty();
    assert !o.query("builtinSpecializers({y: 1}, \"DictionaryWithFields\")").results().isEmpty();

    // Test deref behaviour
    o.loadStr("?= x = 1 and E.sum([x, 2, x]) = 4 and [3, 2, x].indexOf(1) = 2;");

    // Test unspecialized rule ordering
    assert o.queryRule("testUnspecializedRuleOrder", "foo", "bar", new Variable("z"))
        .results()
        .equals(List.of(Map.of("z", 1), Map.of("z", 2), Map.of("z", 3)));

    System.out.println("Tests Pass");
  }
}
