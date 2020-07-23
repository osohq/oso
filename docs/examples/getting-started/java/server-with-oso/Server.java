import java.io.*;
import java.net.InetSocketAddress;
import com.sun.net.httpserver.*;
import com.osohq.oso.Oso;

public class Server implements HttpHandler {
    public static Expense[] EXPENSES = { new Expense(500, "coffee", "alice@example.com"),
            new Expense(5000, "software", "alice@example.com"), new Expense(50000, "flight", "bhavik@example.com"), };

    private Oso oso;

    public Server() throws Exception {
        oso = new Oso();
        oso.loadFile("expenses.polar");
    }

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
            String actor = exchange.getRequestHeaders().get("user").get(0);
            String action = exchange.getRequestMethod();
            String[] request = exchange.getRequestURI().toString().split("/");
            if (!request[1].equals("expenses")) {
                respond(exchange, "Not Found!", 401);
                return;
            }
            Integer index = Integer.parseInt(request[2]) - 1;
            Expense resource = Server.EXPENSES[index];
            if (!oso.allow(actor, action, resource)) {
                respond(exchange, "Not Authorized!", 403);
                return;
            }
            respond(exchange, resource.toString(), 200);
        } catch (Exception e) {
            respond(exchange, "Not Found!", 401);
        }
    }

    public static void main(String[] args) throws Exception {
        HttpServer server = HttpServer.create(new InetSocketAddress("localhost", 5050), 0);
        server.createContext("/", new Server());
        server.start();
        System.out.println("MyServer running on " + server.getAddress());
    }
}
