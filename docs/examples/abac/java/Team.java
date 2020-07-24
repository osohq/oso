public class Team {
    public Integer organizationId;

    public Team(Integer organizationId) {
        this.organizationId = organizationId;
    }

    public static Team byId(Integer id) {
        return new Team(0);
    }

}