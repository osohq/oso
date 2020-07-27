require "oso"
require "webrick"

require "./expense"

OSO ||= Oso.new
OSO.load_file("expenses.polar")

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
server.start if __FILE__ == $0
