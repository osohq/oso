import java.util.*;

public class Predicate {
    public String name;
    public List<Object> args;

    public Predicate(String name, List<Object> args) {
        this.name = name;
        this.args = args;
    }

    @Override
    public boolean equals(Object obj) {
        if (!(obj instanceof Predicate)) {
            return false;
        }
        if (((Predicate) obj).name.equals(this.name) && ((Predicate) obj).args.equals(this.args)) {
            return true;
        } else {
            return false;
        }
    }

}