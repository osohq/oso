from dataclasses import dataclass

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

OSO = Oso()
OSO.load_str(
    """allow(actor, "GET", expense) if
           actor.endswith("@example.com")
           and expense.submitted_by = actor;"""
)

from http.server import HTTPServer, BaseHTTPRequestHandler

class MyRequestHandler(BaseHTTPRequestHandler):
    def _respond(self, msg, code=200):
        self.send_response(code)
        self.end_headers()
        self.wfile.write(str(msg).encode())

    def do_GET(self):
        actor = self.headers.get("user", None)
        action = "GET"

        try:
            _, resource_type, resource_id = self.path.split("/")
            resource = EXPENSES[int(resource_id)]

            if resource_type != "expenses":
                return self._respond("Not Found!", 404)
            elif OSO.allow(actor, action, resource):
                self._respond(resource)
            else:
                self._respond("Not Authorized!", 403)

        except (KeyError, ValueError) as e:
            self._respond("Not Found!", 404)

server_address = ("", 5050)
httpd = HTTPServer(server_address, MyRequestHandler)
print("running on port", httpd.server_port)
httpd.serve_forever()
