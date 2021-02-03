---
accountId: account_id
customerAccounts: customer_accounts
langName: Python

accountManager: |
    ```python
    @polar_class
    class AccountManager(InternalUser):
        def customer_accounts(self):
            yield db.query("SELECT id FROM customer_accounts WHERE manager_id = ?", self.id)
    ```

actorClasses: |
    ```python
    from oso import polar_class

    @polar_class
    class Customer:
        def __init__(self, id):
            self.id = id

    @polar_class
    class InternalUser:
        def __init__(self, id):
            self.id = id
    ```

customerDashboardHandler: |
    ```python
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
    ```

generateAccountManagers: |
    ```python
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
    ```

internalUserRole: |
    ```python
    class InternalUser:
        ...

        def role(self):
            yield db.query("SELECT role FROM internal_roles WHERE id = ?", self.id)
    ```
---
