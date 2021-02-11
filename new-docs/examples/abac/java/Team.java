public class Team {
  public Integer organizationId;

  public Team(Integer organizationId) {
    this.organizationId = organizationId;
  }

  public static Team id(Integer id) {
    return new Team(0);
  }
}
