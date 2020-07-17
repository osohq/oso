require "oso"

OSO ||= Oso.new
OSO.allow(actor: "alice", action: "view", resource: "expense")
