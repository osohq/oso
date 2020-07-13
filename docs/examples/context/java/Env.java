public class Env {
    public String var(String variable) {
        String value = System.getenv(variable);
        return value;
    }
}