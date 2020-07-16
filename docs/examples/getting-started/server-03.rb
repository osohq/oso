# frozen_string_literal: true

require "webrick"
require "oso"

OSO ||= Oso.new

def authorized?(req)
  OSO.allow(actor: req.header["user"]&.first, action: req.request_method, resource: req.path)
end

server = WEBrick::HTTPServer.new Port: 5050
server.mount_proc "/" do |req, res|
  res.body = authorized?(req) ? "Authorized!" : "Not Authorized!"
end
server.start
