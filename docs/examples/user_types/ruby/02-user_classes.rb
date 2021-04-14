# classes-start
require 'oso'

OSO ||= Oso.new

class Customer
  def initialize(id)
    @id = id
  end
end

OSO.register_class(Customer)

# internal-start
class InternalUser
  # ...

  def role
    db.query('SELECT role FROM internal_roles WHERE id = ?', id)
  end
end
# internal-end

OSO.register_class(InternalUser)

# account-start
class AccountManager < InternalUser
  def customer_accounts
    db.query('SELECT id FROM customer_accounts WHERE manager_id = ?', id)
  end
end
# account-end

# generate-start
def user_from_id(id)
  case db.query('SELECT type FROM users WHERE id = ?', id)
  when 'internal'
    actor = InternalUser.new(id: id)
    if actor.role == 'account_manager'
      AccountManager.new(id: id)
    else
      actor
    end
  when 'customer'
    Customer.new(id: id)
  end
end
# generate-end
