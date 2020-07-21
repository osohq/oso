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
            o.allow("a", "b", "c");
            o.loadStr("?= x = \"hello world!\" and x.endsWith(\"world!\");");
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
            boolean passes = !o.queryPredicate("specializers", List.of(new D("hello"), new BC("hello"))).isEmpty()
                    && !o.queryPredicate("floatLists", null).isEmpty() && !o.queryPredicate("intDicts", null).isEmpty()
                    && !o.queryPredicate("comparisons", null).isEmpty()
                    && !o.queryPredicate("testForall", null).isEmpty() && !o.queryPredicate("testRest", null).isEmpty()
                    && !o.queryPredicate("testMatches", List.of(new A("hello"))).isEmpty()
                    && !o.queryPredicate("testMethodCalls", List.of(new A("hello"), new BC("hello"))).isEmpty()
                    && !o.queryPredicate("testOr", null).isEmpty() && o.queryPredicate("testCut", null).isEmpty()
                    && !o.queryPredicate("testHttpAndPathMapper", null).isEmpty();
            if (!passes)
                throw new Exception();

            o.registerConstant("Math", Math.class);
            o.loadStr("?= Math.PI == 3.141592653589793;");

        } catch (Exception e) {
            e.printStackTrace(System.out);
            System.exit(1);
        }
        System.out.println("Tests Pass");
    }
}