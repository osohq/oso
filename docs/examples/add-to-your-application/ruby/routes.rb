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
    OSO.authorize(User.get_current_user,
                  "read",
                  repo)
    "<h1>A Repo</h1><p>Welcome to repo #{repo.name}</p>"
  rescue Oso::NotFoundError
    # TODO can i have a message here
    404
  end
end
# docs: end-show-route
