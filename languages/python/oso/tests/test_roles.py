import pytest

from oso import Oso, polar_class, Roles
from polar import exceptions


class Actor:
    name: str = ""

    def __init__(self, name=""):
        self.name = name


class Widget:
    id: str = ""
    public: bool

    def __init__(self, id, public=False):
        self.id = id
        self.public = public


@pytest.fixture
def test_roles():
    oso = Oso()
    oso.register_class(Actor, name="Actor")
    oso.register_class(Widget, name="Widget")

    ## ROLE DEFINITION

    # does the user need to be a Python class?
    # probably want to support dicts and strings/ints for users and resources
    # if we do that need to figure out how to make the "id" fields default to the object itself
    oso.register_roles(
        user=Actor,
        user_id="name",
        roles=["ADMIN", "USER"],
        # exclusive=True,
        # inherits=[("Admin", "User")],
    )
    oso.register_roles(user=Actor, resource=Widget, resource_id="id", roles=["OWNER"])
    # role constraints?

    {user: the_actor, role: "ADMIN", resource: the_widget, kind: "Widget"}

    role_config = """
    GlobalRole = {
        user: Actor,
        user_id: "name"
        roles: ["ADMIN", "USER"]
    }

    WidgetRole = {
        user: Actor,
        user_id: "name",
        resource: Widget,
        resource_id: "id"
        roles: ["OWNER"]
    }
    """

    rules = """
    allow(user: Actor, "UPDATE", widget: Widget) if
        {role: "ADMIN"} in Roles.get_user_roles(user) or
        {role: "OWNER"} in Roles.get_user_roles(user, widget) or
        widget.public;

    #allow(user: Actor, "UPDATE", widget: Widget) if
    #    Roles.user_in_role(user, {role: "ADMIN"}) or
    #    Roles.user_in_role(user, "OWNER", widget) or
    #   widget.public;

    allow(user: Actor, "UPDATE", resource: Widget) if
        {role: "OWNER"} in Roles.user_roles(user, resource.parent);

    #allow(user: Actor, "UPDATE", resource: Widget) if
    #    Roles.user_in_role(user, role, resource.parent) and
    #    role_allow(role, action, resource);

    allow(user: Actor, action, resource: Widget) if
        allow(user, action, resource.parent);

    allow(user: Actor, _action, resource: WidgetParent) if
        Roles.user_in_role(user, "ADMIN", resource);
    """

    # need to know
    # User / Actor class
    # Resources the user can have roles on.
    #

    roles = Roles()
