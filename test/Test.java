import com.osohq.oso.*;

import java.util.List;
import java.lang.Math;

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

    public static void main(String[] args) {
        try {
            Oso o = new Oso();
            o.registerClass(A.class, m -> new A((String) m.get("x")), "A");
            o.registerClass(BC.class, m -> new BC((String) m.get("y")), "C");
            o.loadFile("test.polar");
            o.isAllowed("a", "b", "c");

            // Test that a built in string method can be called.
            o.loadStr("?= x = \"hello world!\" and x.endsWith(\"world!\");");

            // Test that a custom error type is thrown.
            boolean throwsException = false;
            try {
                o.loadStr("missingSemicolon()");
            } catch (Exceptions.UnrecognizedEOF e) {
                if (!e.getMessage().equals(
                        "hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 19"))
                    throw new Exception();
                throwsException = true;
            }
            if (!throwsException)
                throw new Exception();
            boolean passes = !o.queryRule("specializers", new D("hello"), new BC("hello")).results().isEmpty()
                    && !o.queryRule("floatLists").results().isEmpty() && !o.queryRule("intDicts").results().isEmpty()
                    && !o.queryRule("comparisons").results().isEmpty() && !o.queryRule("testForall").results().isEmpty()
                    && !o.queryRule("testRest").results().isEmpty()
                    && !o.queryRule("testMatches", new A("hello")).results().isEmpty()
                    && !o.queryRule("testMethodCalls", new A("hello"), new BC("hello")).results().isEmpty()
                    && !o.queryRule("testOr").results().isEmpty()
                    // Test that cut doesn't return anything.
                    && o.queryRule("testCut").results().isEmpty()
                    && !o.queryRule("testHttpAndPathMapper").results().isEmpty();
            if (!passes)
                throw new Exception();

            // Test that a constant can be called.
            o.registerConstant("Math", Math.class);
            o.loadStr("?= Math.PI == 3.141592653589793;");

        } catch (Exception e) {
            e.printStackTrace(System.out);
            System.exit(1);
        }
        System.out.println("Tests Pass");
    }
}
