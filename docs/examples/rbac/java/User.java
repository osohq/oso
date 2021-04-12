import com.osohq.oso.*;

public class User {
  public String name;

  public User(String name) {
    this.name = name;
  }

  public String role() {
    return DB.query("SELECT role FROM user_roles WHERE username = ?", name);
  }

  public static void main(String[] args) {
    Oso oso = Oso();
    oso.registerClass(User.class);
  }
}
