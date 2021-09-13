---
githubUrl: "https://github.com/osohq/oso-nodejs-quickstart"
githubCloneUrl: "https://github.com/osohq/oso-nodejs-quickstart.git"
repoName: "oso-nodejs-quickstart"
mainPolarFile: "examples/quickstart/nodejs/main.polar"
serverFile: "examples/quickstart/nodejs/server.js"
modelFile: "examples/quickstart/nodejs/models.js"
polarFileRelative: "main.polar"
serverFileRelative: "server.js"
modelFileRelative: "models.js"
installDependencies: npm install
startServer: npm run dev
osoAuthorize: oso.authorize()
isPublic: isPublic
hasRole: |-
  has_role(actor: User, role_name: String, repository: Repository) if
    role in actor.roles and
    role_name = role.name and
    repository = role.repository;
endpoint: the `/repo/:name` route
port: 5000
---
