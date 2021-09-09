public class User {
  public String role;
  public List<User> treated;

  public User(String role, List<User> treated) {
    this.role = role;
    this.treated = treated;
  }

  public boolean treated(Patient patient) {
    this.treated.contains(patient);
  }
}
