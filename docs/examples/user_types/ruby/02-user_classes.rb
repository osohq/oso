# classes-start
require "oso"

OSO ||= Oso.new

class Customer
  def initialize(id:)
    @id = id
  end
end

OSO.register_class(Customer)

# internal-start
class InternalUser
  def initialize
    @id = id
  end

  def role
    db.query("SELECT role FROM internal_roles WHERE id = ?", @id)
  end
end

OSO.register_class(InternalUser)

# internal-end

# account-start

class AccountManager < InternalUser
    def customer_accounts
        db.query("SELECT id FROM customer_accounts WHERE manager_id = ?", @id)
    end
end

# account-end
def user_from_id(id)
  user_type = db.query("SELECT type FROM users WHERE id = ?", id)
  if user_type == "internal"
    InternalUser.new(id: id)
    if actor.role() == "account_manager"
      return AccountManager.new(id: id)
    end
  else if user_type == "customer"
    CustomerUser.new(id: id)
  end
end
