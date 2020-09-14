import threading
from django.db import models
from django.db.models import Q

from .oso import Oso
from oso import Variable


class OsoManager(models.Manager):
    _requests = {}

    @classmethod
    def get_request(cls, default=None):
        """
        Retrieve the request object for the current thread, or the optionally
        provided default if there is no current request.
        """
        return cls._requests.get(threading.current_thread(), default)

    @classmethod
    def set_request(cls, request):
        """
        Save the given request into storage for the current thread.
        """
        cls._requests[threading.current_thread()] = request

    @classmethod
    def del_request(cls):
        """
        Delete the request that was stored for the current thread.
        """
        cls._requests.pop(threading.current_thread(), None)

    def get_queryset(self):
        request = OsoManager.get_request()
        filter = Q()
        for result in Oso.query_rule(
            "allow_scope", request.user, request.method, self.model, Variable("scope")
        ):
            filter |= result["bindings"]["scope"]
        return super().get_queryset().filter(filter)
