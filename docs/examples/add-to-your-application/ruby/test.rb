require 'test/unit'
require 'rack/test'

require_relative './oso'
require_relative './routes'


class TestEverything < Test::Unit::TestCase
  include Rack::Test::Methods

  def app
    Sinatra::Application
  end

  def test_policy_loads_and_oso_inits
    assert OSO
  end

  def test_route_works
    get "/repo/gmail"
    assert last_response.ok?
  end
end
