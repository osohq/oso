# classes-start
require "oso"

OSO ||= Oso.new

class Customer
  def initialize(id:)
    @id = id
  end
end

OSO.register_class(Customer)

class InternalUser
  def initialize(id:)
    @id = id
  end
end

OSO.register_class(InternalUser)
# classes-end

# app-start
def customer_dashboard_handler(request)
  actor = user_from_id(request.id)
  allowed = OSO.allow(
    actor: actor,
    action: "view",
    resource: "customer_dashboard")
end

def user_from_id(id)
  user_type = db.query("SELECT type FROM users WHERE id = ?", id)
  if user_type == "internal"
    InternalUser.new(id: id)
  else if user_type == "customer"
    CustomerUser.new(id: id)
  end
end

# app-end
