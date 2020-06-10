from typing import Any, Callable, List
from . import Oso
from polar.api import Http, Query


class OsoFlask(Oso):
    """Flask-specific oso functionality"""

    def filter_map(
        self,
        request: "flask.Request",
        f: Callable,
        credentials=None,
        credential_header=None,
        hostname="",
    ) -> List[Any]:
        """Filter out unauthorized results for a Flask endpoint, and map over
        the authorized results.

        :param request: The flask request.
        :param f: The function to be called on each query result.

        :return: List of filtered query results.
        """
        if not credentials and credential_header:
            credentials = request.headers.get(credential_header, None)
        if not credentials:
            credentials = {}
        action = request.method.lower()
        resource = Http(path=request.path, hostname=hostname)
        query = Query(name="allow", args=(credentials, action, resource))
        return list(f(r) for r in self.query(query, single=True).results if f(r))

    def verify_flask_request(
        self,
        request: "flask.Request",
        credentials=None,
        credential_header=None,
        hostname="",
    ) -> bool:
        """Verify a Flask request
        Credentials can be an "Actor" class, a dictionary of attributes or a string.
        credential_header is the name of a header to read the credentials from.
        """
        if not credentials and credential_header:
            credentials = request.headers.get(credential_header, None)
        if not credentials:
            credentials = {}
        action = request.method.lower()
        resource = Http(path=request.path, hostname=hostname)
        query = Query(name="allow", args=(credentials, action, resource))
        return self.query(query, single=True).success
