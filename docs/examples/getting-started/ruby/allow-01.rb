require "oso"

OSO ||= Oso.new
actor = "alice@example.com"
resource = EXPENSES[1]
OSO.allow(actor: actor, action: "GET", resource: resource)
