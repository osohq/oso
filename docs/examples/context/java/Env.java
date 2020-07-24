import java.util.Map;

public class Env {
    public Env() {}

    public String var(String variable) {
        Map<String, String> env = System.getenv();
        return env.get(variable);
    }
}
