public class Organization {
  public String name;

  public Organization(String name) {
    this.name = name;
  }

  public static Organization id(Integer id) {
    return new Organization("ACME");
  }
}
