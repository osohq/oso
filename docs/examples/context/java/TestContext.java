import java.util.List;

import com.osohq.oso.*;

public class TestContext {
    public static Oso setupOso() throws Exception {
        Oso oso = new Oso();
        oso.registerClass(Env.class);
        return oso;
    }

    public static void testPolicy() throws Exception {
        Oso oso = setupOso();
        oso.loadFile("../01-context.polar");

        if (!oso.isAllowed("steve", "test", "policy")) {
            throw new Exception("test context failed!");
        }
    }

    public static void main(String[] args) throws Exception {
        testPolicy();
        System.out.println("Context tests pass!");
    }
}
