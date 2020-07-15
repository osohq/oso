import java.io.IOException;
import java.io.OutputStream;
import java.net.InetSocketAddress;
import java.util.concurrent.Executors;
import java.util.concurrent.ThreadPoolExecutor;

import com.sun.net.httpserver.HttpServer;
import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpHandler;

import com.osohq.oso.Oso;
import com.osohq.oso.Exceptions.*;

public class MyHttpHandler implements HttpHandler {
    private Oso oso;

    public MyHttpHandler() {
        try {
            oso = new Oso();
            // Allow Alice to make GET requests to any path.
            oso.loadStr("allow(\"alice@example.com\", \"GET\", _);");

            // Allow anyone whose email address ends in "@example.com" to make
            // POST requests to any path that starts with "/admin".
            oso.loadStr("allow(email, \"POST\", path) if\n" + "email.end_with?(\"@example.com\") = true and\n"
                    + "path.start_with?(\"/admin\") = true;");
        } catch (OsoException e) {
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
        String htmlResponse = authorized(exchange) ? "Authorized!\n" : "Not authorized!\n";
        exchange.sendResponseHeaders(200, htmlResponse.length());
        outputStream.write(htmlResponse.getBytes());
        outputStream.flush();
        outputStream.close();
    }

    public static void main(String[] args) throws IOException {
        HttpServer server = HttpServer.create(new InetSocketAddress("localhost", 5050), 0);
        ThreadPoolExecutor threadPoolExecutor = (ThreadPoolExecutor) Executors.newFixedThreadPool(10);

        server.createContext("/", new MyHttpHandler());
        server.setExecutor(threadPoolExecutor);
        server.start();
        System.out.println("Server started on port 5050");
    }

}
