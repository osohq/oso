require 'oso'

OSO ||= Oso.new
actor = 'alice@example.com'
resource = EXPENSES[1]
OSO.allowed?(actor: actor, action: 'GET', resource: resource)
