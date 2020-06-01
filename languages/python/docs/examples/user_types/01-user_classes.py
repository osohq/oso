# classes-start
from oso import polar_class

@polar_class
class Customer:
    def __init__(self, id):
        self.id = id

@polar_class
class InternalUser:
    def __init__(self, id):
        self.id = id
# classes-end

# app-start
def customer_dashboard_handler(request, ...):
    oso = get_oso()
    actor = user_from_id(request.id)
    allowed = oso.allow(
        actor=actor,
        action="view",
        resource="customer_dashboard")

def user_from_id(id):
    user_type = db.query("SELECT type FROM users WHERE id = ?", request.id)
    if user_type == "internal":
        return InternalUser(request.id)
    elif user_type == "customer":
        return CustomerUser(request.id)
# app-end