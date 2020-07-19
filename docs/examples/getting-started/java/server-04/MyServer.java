import java.io.*;
import java.net.InetSocketAddress;
import java.util.concurrent.*;
import com.sun.net.httpserver.*;

import com.osohq.oso.Oso;
// import com.osohq.oso.Exceptions.OsoException;

public class MyServer implements HttpHandler {
    private Oso oso;

    public MyServer() {
        try {
            oso = new Oso();
            // Allow Alice to make GET requests to any path.
            oso.loadStr("allow(\"alice@example.com\", \"GET\", _);");

            // Allow anyone whose email address ends in "@example.com" to make
            // POST requests to any path that starts with "/admin".
            oso.loadStr("allow(email, \"POST\", path) if\n" + "email.endsWith(\"@example.com\") = true and\n"
                    + "path.startsWith(\"/admin\") = true;");
        } catch (Exception e) {
            System.out.println("Failed to initialize oso.");
        }
    }

    private boolean authorized(HttpExchange exchange) {
        try {
            String actor = exchange.getRequestHeaders().get("user").get(0);
            String action = exchange.getRequestMethod();
            String resource = exchange.getRequestURI().toString();

            return oso.allow(actor, action, resource);
        } catch (Exception e) {
            return false;
        }
    }

    @Override
    public void handle(HttpExchange exchange) throws IOException {

        OutputStream outputStream = exchange.getResponseBody();
        String htmlResponse = authorized(exchange) ? "Authorized!\n" : "Not Authorized!\n";
        exchange.sendResponseHeaders(200, htmlResponse.length());
        outputStream.write(htmlResponse.getBytes());
        outputStream.flush();
        outputStream.close();
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
