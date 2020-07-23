from dataclasses import dataclass
from http.server import HTTPServer, BaseHTTPRequestHandler


@dataclass
class Expense:
    amount: int
    description: str
    submitted_by: str


EXPENSES = {
    1: Expense(500, "coffee", "alice@example.com"),
    2: Expense(5000, "software", "alice@example.com"),
    3: Expense(50000, "flight", "bhavik@example.com"),
}


class RequestHandler(BaseHTTPRequestHandler):
    def _respond(self, msg, code=200):
        self.send_response(code)
        self.end_headers()
        self.wfile.write(str(msg).encode())
        self.wfile.write(b"\n")

    def do_GET(self):
        try:
            _, resource_type, resource_id = self.path.split("/")
            if resource_type != "expenses":
                return self._respond("Not Found!", 404)
            resource = EXPENSES[int(resource_id)]
            self._respond(resource)
        except (KeyError, ValueError) as e:
            self._respond("Not Found!", 404)


server_address = ("", 5050)
httpd = HTTPServer(server_address, RequestHandler)
if __name__ == "__main__":
    print("running on port", httpd.server_port)
    httpd.serve_forever()
