---
githubUrl: "https://github.com/osohq/oso-ruby-quickstart"
githubCloneUrl: "https://github.com/osohq/oso-ruby-quickstart.git"
repoName: oso-ruby-quickstart
mainPolarFile: "examples/quickstart/ruby/main.polar"
serverFile: "examples/quickstart/ruby/server.rb"
modelFile: "examples/quickstart/ruby/models.rb"
polarFileRelative: "main.polar"
serverFileRelative: "server.rb"
modelFileRelative: "models.rb"
installDependencies: bundle install
startServer: bundle exec ruby server.rb
osoAuthorize: OSO.authorize()
isPublic: is_public
hasRole: |-
  has_role(actor: User, role_name: String, repository: Repository) if
    role in actor.roles and
    role_name = role.name and
    repository = role.repository;
endpoint: the `/repo/:name` route
port: 5000
---
