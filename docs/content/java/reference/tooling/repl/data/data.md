---
startRepl: |
    ```
    $ mvn exec:java -Dexec.mainClass="com.osohq.oso.Oso"
    query>
    ```
startReplWithFile: |
    ```
    $ mvn exec:java -Dexec.mainClass="com.osohq.oso.Oso" -Dexec.args="alice.polar"
    ```
replApi: |
    ```python
    import com.example.Expense;
    import com.example.User;

    import com.osohq.oso.*;

    public class AppRepl {
        public static void main(String[] args) throws OsoException, IOException {
            Oso oso = new Oso();
            oso.registerClass(Expense.class);
            oso.registerClass(User.class);
            oso.loadFiles(["alice.polar"]);
            oso.repl(args)
        }
    }
    ```
---