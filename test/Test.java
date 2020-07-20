import com.osohq.oso.*;

import com.osohq.oso.Oso;

class Test {
    static class A {
        private String x;
    
        public A(String x) {
            this.x = x;
        }
    }

    public static void main(String[] args) {
        try {
            Oso o = new Oso();
            o.registerClass(A.class, m -> new A((String) m.get("x")), "A");
            o.loadFile("test.polar");
        } catch (Exception e) {
            System.out.println(e);
            System.exit(1);
        }
        System.out.println("Tests Pass");
    }
}