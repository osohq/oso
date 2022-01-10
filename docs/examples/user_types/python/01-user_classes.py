class Customer:
    def __init__(self, id):
        self.id = id


class InternalUser:
    def __init__(self, id):
        self.id = id
        # classes-end


# app-start
def customer_dashboard_handler(request):
    oso = get_oso()
    actor = user_from_id(request.id)
    allowed = oso.is_allowed(actor=actor, action="view", resource="customer_dashboard")


def user_from_id(id):
    user_type = db.query("SELECT type FROM users WHERE id = ?", id)
    if user_type == "internal":
        return InternalUser(id)
    elif user_type == "customer":
        return Customer(id)
        # app-end
