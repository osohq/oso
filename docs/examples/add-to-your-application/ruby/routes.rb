require 'oso'
require 'sinatra'

require_relative './oso'
require_relative './models'

class User
  def self.get_current_user
    USERS_DB['larry']
  end
end

# docs: begin-show-route
get '/repo/:name' do
  repo = Repository.get_by_name(params['name'])

  begin
    # docs: begin-authorize
    OSO.authorize(User.get_current_user, 'read', repo)
    # docs: end-authorize
    "<h1>A Repo</h1><p>Welcome to repo #{repo.name}</p>"
  rescue Oso::NotFoundError
    status 404
    "<h1>Whoops!</h1><p>Repo named #{params['name']} was not found</p>"
  end
end
# docs: end-show-route
