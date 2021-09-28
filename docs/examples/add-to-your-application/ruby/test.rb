require 'test-unit'
require 'rack/test'

require_relative './oso'
require_relative './routes'
require_relative './data_filtering'


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

  def test_data_filtering_works
    get "/repos"
    # make sure some repo is returned
    assert last_response.body.length > 4
    assert last_response.ok?
  end
end
