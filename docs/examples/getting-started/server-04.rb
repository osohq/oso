# frozen_string_literal: true

require 'webrick'
require 'oso'

OSO ||= Oso.new

# Allow Alice to make GET requests to any path.
OSO.load_str 'allow("alice@example.com", "GET", _);'

# Allow anyone whose email address ends in "@example.com"
# to make POST requests to any path.
OSO.load_str <<~RULE
  allow(email, "POST", _) if
      email.end_with?("@example.com") = true;
RULE

def authorized?(req)
  OSO.allow(actor: req.header['user']&.first, action: req.request_method, resource: req.path)
end

server = WEBrick::HTTPServer.new Port: 5050
server.mount_proc '/' do |req, res|
  res.body = authorized?(req) ? 'Authorized!' : 'Not Authorized!'
end
server.start
