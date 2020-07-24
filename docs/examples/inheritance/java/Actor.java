public class Actor {
    public String role;
    public List<Actor> treated;

    public Actor(String role, List<Actor> treated) {
        this.role = role;
        this.treated = treated;
    }

    public boolean treated(Patient patient) {
        this.treated.contains(patient);
    }
}