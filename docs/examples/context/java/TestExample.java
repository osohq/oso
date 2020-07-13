import java.io.IOException;

public class TestExample {
    public static void main(String[] args) {
        try {
            Oso oso = new Oso();
            oso.registerClass(Env.class, m -> new Env(), "Env");
            oso.loadFile("../01-context.polar");
            if (oso.allow("steve", "test", "policy")) {
                System.out.println("Works");
            } else {
                System.out.println("Doesn't work");
            }
        } catch (Exceptions.DuplicateClassAliasError e) {
            System.out.println(e);
        } catch (Exceptions.OsoException e) {
            System.out.println(e);
        } catch (IOException e) {
            System.out.println(e);
        }
    }
}