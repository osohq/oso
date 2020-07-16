# frozen_string_literal: true

require "webrick"
require "oso"

OSO ||= Oso.new

# ...

server = WEBrick::HTTPServer.new Port: 5050
server.mount_proc "/" do |_, res|
  res.body = "Authorized!"
end
server.start
