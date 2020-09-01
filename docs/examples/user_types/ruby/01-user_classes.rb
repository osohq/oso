# classes-start
require 'oso'

OSO ||= Oso.new

class Customer
  def initialize(id)
    @id = id
  end
end

OSO.register_class(Customer)

class InternalUser
  def initialize(id)
    @id = id
  end
end

OSO.register_class(InternalUser)
# classes-end

# app-start
def customer_dashboard_handler(request)
  actor = user_from_id(request.id)
  OSO.allowed?(actor: actor, action: 'view', resource: 'customer_dashboard')
end

def user_from_id(id)
  case db.query('SELECT type FROM users WHERE id = ?', id)
  when 'internal'
    InternalUser.new(id: id)
  when 'customer'
    Customer.new(id: id)
  end
end
# app-end
