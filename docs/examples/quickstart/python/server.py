from http.server import HTTPServer, BaseHTTPRequestHandler
from oso import Oso

from .expense import Expense, EXPENSES

oso = Oso()
oso.load_file("expenses.polar")


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
if __name__ == "__main__":
    print("running on port", httpd.server_port)
    httpd.serve_forever()
