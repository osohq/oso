import java.io.*;
import java.net.InetSocketAddress;
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

public class Server implements HttpHandler {
    public static Expense[] EXPENSES = {
            new Expense(500, "coffee", "alice@example.com"),
            new Expense(5000, "software", "alice@example.com"),
            new Expense(50000, "flight", "bhavik@example.com"), };

    private void respond(HttpExchange exchange, String message, int code) throws IOException {
        exchange.sendResponseHeaders(code, message.length() + 1);
        OutputStream outputStream = exchange.getResponseBody();
        outputStream.write(message.getBytes());
        outputStream.write("\n".getBytes());
        outputStream.flush();
    }

    @Override
    public void handle(HttpExchange exchange) throws IOException {
        try {
            String[] request = exchange.getRequestURI().toString().split("/");
            if (!request[1].equals("expenses")) {
                this.respond(exchange, "Not Found!", 401); return;
            }
            Integer index = Integer.parseInt(request[2]) - 1;
            Expense resource = Server.EXPENSES[index];
            this.respond(exchange, resource.toString(), 200);
        } catch (Exception e) {
            this.respond(exchange, "Not Found!", 401); return;
        }
    }

    public static void main(String[] args) throws IOException {
        HttpServer server = HttpServer.create(new InetSocketAddress("localhost", 5050), 0);
        server.createContext("/", new Server());
        server.start();
        System.out.println("MyServer running on " + server.getAddress());
    }
}
