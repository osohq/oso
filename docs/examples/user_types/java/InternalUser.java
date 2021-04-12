public class InternalUser {
  public Integer id;

  public InternalUser(Integer id) {
    this.id = id;
  }

  public String role() {
    return DB.query("SELECT role FROM internal_roles WHERE id = ?", id);
  }
}
