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
# expenses code

from oso import Oso

OSO = Oso()

# server code

from http.server import HTTPServer, BaseHTTPRequestHandler


class MyRequestHandler(BaseHTTPRequestHandler):
    def _respond(self, msg):
        self.send_response(200)
        self.end_headers()
        self.wfile.write(msg.encode())

    def do_GET(self):
        try:
            _, resource_type, resource_id = self.path.split("/")
            resource = EXPENSES[int(resource_id)]
            self._respond(str(resource))
        except (KeyError, ValueError) as e:
            self._respond("Not Found!")


server_address = ("", 5050)
httpd = HTTPServer(server_address, MyRequestHandler)
httpd.serve_forever()
