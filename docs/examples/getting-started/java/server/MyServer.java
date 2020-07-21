import java.io.*;
import java.net.InetSocketAddress;
import java.util.concurrent.*;
import com.sun.net.httpserver.*;

class Expense {
    public int amount;
    public String description;
    public String submittedBy;

    public Expense(int amount, String description, String submittedBy) {
        this.amount = amount;
        this.description = description;
        this.submittedBy = submittedBy;
    }

    public String toString() {
        return String.format("Expense(%d, %s, %s)", this.amount, this.description, this.submittedBy);
    }
}

public class MyServer implements HttpHandler {

    public Expense[] EXPENSES = { new Expense(500, "coffee", "alice@example.com"),
            new Expense(5000, "software", "alice@example.com"), new Expense(50000, "flight", "bhavik@example.com"), };

    private void respond(HttpExchange exchange, String message, int code) throws IOException {
        exchange.sendResponseHeaders(code, message.length());
        OutputStream outputStream = exchange.getResponseBody();
        outputStream.write(message.getBytes());
        outputStream.write("\n".getBytes());
        outputStream.flush();
        outputStream.close();
    }

    @Override
    public void handle(HttpExchange exchange) throws IOException {
        String[] request = exchange.getRequestURI().toString().split("/");
        if (request.length != 3 || !request[1].equals("expenses")) {
            this.respond(exchange, "Not Found!", 401);
            return;
        }

        Integer index;
        try {
            index = Integer.parseInt(request[2]) - 1;
        } catch (NumberFormatException e) {
            this.respond(exchange, "Not Found!", 401);
            return;
        }
        if (index >= this.EXPENSES.length) {
            System.out.println("Index out of range");
            this.respond(exchange, "Not Found!", 401);
            return;
        }
        Expense resource = this.EXPENSES[index];
        this.respond(exchange, resource.toString(), 200);
    }

    public static void main(String[] args) throws IOException {
        HttpServer server = HttpServer.create(new InetSocketAddress("localhost", 5050), 0);
        ThreadPoolExecutor threadPoolExecutor = (ThreadPoolExecutor) Executors.newFixedThreadPool(10);

        server.createContext("/", new MyServer());
        server.setExecutor(threadPoolExecutor);
        server.start();
        System.out.println("MyServer running on " + server.getAddress());
    }

}
