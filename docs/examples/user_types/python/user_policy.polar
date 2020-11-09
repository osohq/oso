# simple-start
# Internal users have access to both the
# internal and customer dashboards
allow(actor: InternalUser, "view", "internal_dashboard");
allow(actor: InternalUser, "view", "customer_dashboard");

# Customers only have access to the customer dashboard
allow(actor: Customer, "view", "customer_dashboard");
# simple-end

# rbac-start
# Internal users can access the accounts dashboard if
# they are an account manager
allow(actor: InternalUser, "view", "accounts_dashboard") if
    actor.role() = "account_manager";
# rbac-end

# manager-start
# Account managers can access the accounts dashboard
allow(actor: AccountManager, "view", "accounts_dashboard");

# Account managers can access account data for the accounts
# that they manage
allow(actor: AccountManager, "view", resource: AccountData) if
    resource.account_id in actor.customer_accounts();
# manager-end
