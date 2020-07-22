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

server = WEBrick::HTTPServer.new Port: 5050
server.mount_proc "/" do |req, res|
  _, resource_type, resource_id = req.path.split("/")
  resource = EXPENSES[resource_id.to_i]

  if resource_type != "expenses" || resource.nil?
    res.body = "Not Found!"
  else
    res.body = resource.inspect
  end
end
server.start
