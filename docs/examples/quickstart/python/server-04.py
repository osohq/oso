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

from oso import Oso

oso = Oso()
oso.load_str('is_allowed("alice@example.com", "GET", _expense);')


class RequestHandler(BaseHTTPRequestHandler):
    def _respond(self, msg, code=200):
        self.send_response(code)
        self.end_headers()
        self.wfile.write(str(msg).encode())
        self.wfile.write(b"\n")

    def do_GET(self):
        actor = self.headers.get("user", None)
        action = "GET"

        try:
            _, resource_type, resource_id = self.path.split("/")
            if resource_type != "expenses":
                return self._respond("Not Found!", 404)
            resource = EXPENSES[int(resource_id)]
            if oso.is_allowed(actor, action, resource):
                self._respond(resource)
            else:
                self._respond("Not Authorized!", 403)
        except (KeyError, ValueError) as e:
            self._respond("Not Found!", 404)


server_address = ("", 5050)
httpd = HTTPServer(server_address, RequestHandler)
print("running on port", httpd.server_port)
httpd.serve_forever()
