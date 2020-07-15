import java.io.*;
import java.net.InetSocketAddress;
import java.util.concurrent.*;
import com.sun.net.httpserver.*;

public class MyServer implements HttpHandler {

    @Override
    public void handle(HttpExchange exchange) throws IOException {

        OutputStream outputStream = exchange.getResponseBody();
        String htmlResponse = "Authorized!\n";
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
