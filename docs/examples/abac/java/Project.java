public class Project {
    public Integer id, teamId;

    public Project(Integer id, Integer teamId) {
        this.id = id;
        this.teamId = teamId;
    }

    public static Project byId(Integer id) {
        return new Project(id, 0);
    }
}