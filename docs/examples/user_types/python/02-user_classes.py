from oso import polar_class


class Customer:
    pass


# internal-start
class InternalUser:
    ...

    def role(self):
        yield db.query("SELECT role FROM internal_roles WHERE id = ?", self.id)
        # internal-end


# account-start
class AccountManager(InternalUser):
    ...

    def customer_accounts(self):
        yield db.query("SELECT id FROM customer_accounts WHERE manager_id = ?", self.id)
        # account-end


# generate-start
def user_from_id(id):
    user_type = db.query("SELECT type FROM users WHERE id = ?", request.id)
    if user_type == "internal":
        actor = InternalUser(request.id)
        if actor.role() == "account_manager":
            return AccountManager(request.id)
        else:
            return actor
    elif user_type == "customer":
        return Customer(request.id)
        # generate-end
