import java.io.*;
import java.net.InetSocketAddress;
import java.util.concurrent.*;
import com.sun.net.httpserver.*;

import com.osohq.oso.Oso;
import com.osohq.oso.Exceptions.*;

public class MyServer implements HttpHandler {
    private Oso oso;

    public MyServer() {
        try {
            oso = new Oso();
        } catch (OsoException e) {
            System.out.println("Failed to initialize oso.");
        }
    }
    // ...

    @Override
    public void handle(HttpExchange exchange) throws IOException {

        OutputStream outputStream = exchange.getResponseBody();
        String htmlResponse = "Authorized!\n";
        exchange.sendResponseHeaders(200, htmlResponse.length());
        outputStream.write(htmlResponse.getBytes());
        outputStream.flush();
        outputStream.close();

        // for docs
        try {
            oso.allow("alice", "approve", "expense");
        } catch (OsoException e) {
            System.out.println(e.getMessage());
        }
    }

    public static void main(String[] args) throws IOException {
        HttpServer server = HttpServer.create(new InetSocketAddress("localhost", 5050), 0);
        ThreadPoolExecutor threadPoolExecutor = (ThreadPoolExecutor) Executors.newFixedThreadPool(10);

        server.createContext("/", new MyServer());
        server.setExecutor(threadPoolExecutor);
        server.start();

    }

}
