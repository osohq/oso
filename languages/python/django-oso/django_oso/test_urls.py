from django.urls import path
from django.http import HttpResponse, HttpResponseServerError

from django_oso.auth import authorize
from django_oso.decorators import authorize_request
from django_oso import decorators


def root(request):
    return HttpResponse("hello")


def auth(request):
    authorize(request, "resource", action="read", actor="user")
    return HttpResponse("authorized")


@authorize_request(actor="user")
def auth_decorated_fail(request):
    return HttpResponse("authorized")


@decorators.authorize(actor="user", action="read", resource="resource")
def auth_decorated(request):
    return HttpResponse("authorized")


def a(request):
    return HttpResponse("a")


def b(request):
    return HttpResponse("b")


def error(request):
    return HttpResponseServerError()


urlpatterns = [
    path("", root),
    path("auth/", auth),
    path("auth_decorated_fail/", auth_decorated_fail),
    path("auth_decorated/", auth_decorated),
    path("error/", error),
    path("a/", a),
    path("b/", b),
]
