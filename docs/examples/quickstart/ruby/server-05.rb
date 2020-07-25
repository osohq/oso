require "oso"
require "webrick"

class Expense
  attr_reader :amount, :description, :submitted_by

  def initialize(amount, description, submitted_by)
    @amount = amount
    @description = description
    @submitted_by = submitted_by
  end
end

EXPENSES = {
  1 => Expense.new(500,   "coffee",   "alice@example.com"),
  2 => Expense.new(5000,  "software", "alice@example.com"),
  3 => Expense.new(50000, "flight",   "bhavik@example.com"),
}

OSO ||= Oso.new
OSO.load_str <<~RULE
  allow(actor, "GET", _expense) if
      actor.end_with?("@example.com");
RULE

server = WEBrick::HTTPServer.new Port: 5050
server.mount_proc "/" do |req, res|
  actor = req.header["user"]&.first
  action = req.request_method
  _, resource_type, resource_id = req.path.split("/")
  resource = EXPENSES[resource_id.to_i]

  if resource_type != "expenses" || resource.nil?
    res.body = "Not Found!"
  elsif OSO.allowed?(actor: actor, action: action, resource: resource)
    res.body = resource.inspect
  else
    res.body = "Not Authorized!"
  end
end
server.start
