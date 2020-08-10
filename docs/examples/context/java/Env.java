import java.util.Map;

public class Env {
    public static String var(String variable) {
        Map<String, String> env = System.getenv();
        return env.get(variable);
    }
}
