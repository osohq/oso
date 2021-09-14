---
githubUrl: "https://github.com/osohq/oso-go-quickstart"
githubCloneUrl: "https://github.com/osohq/oso-go-quickstart.git"
repoName: oso-go-quickstart
mainPolarFile: "examples/quickstart/go/main.polar"
serverFile: "examples/quickstart/go/server.go"
modelFile: "examples/quickstart/go/models.go"
polarFileRelative: "main.polar"
serverFileRelative: "server.go"
modelFileRelative: "models.go"
installDependencies: go mod download
startServer: go run .
osoAuthorize: oso.Authorize()
isPublic: IsPublic
hasRole: |-
  has_role(user: User, roleName: String, repository: Repository) if
    role in user.Roles and
    role.Role = roleName and
    role.RepoId = repository.Id;
endpoint: the `/repo/:repoName` route
port: 5000
---
