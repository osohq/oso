import com.osohq.oso.*;

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
            o.registerClass(BC.class, m -> new BC((String) m.get("x")), "C");
            o.loadFile("test.polar");
            o.allow("a", "b", "c");
            o.loadStr("?= x = \"hello world!\" and x.end_with?(\"world!\");");
        } catch (Exception e) {
            e.printStackTrace(System.out);
            System.exit(1);
        }
        System.out.println("Tests Pass");
    }
}