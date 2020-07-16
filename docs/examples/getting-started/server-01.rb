# frozen_string_literal: true

require "webrick"

server = WEBrick::HTTPServer.new Port: 5050
server.mount_proc "/" do |_, res|
  res.body = "Authorized!"
end
server.start
