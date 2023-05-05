---
githubUrl: "https://github.com/osohq/oso-rust-quickstart"
githubCloneUrl: "https://github.com/osohq/oso-rust-quickstart.git"
repoName: oso-rust-quickstart
mainPolarFile: "examples/quickstart/rust/src/main.polar"
serverFile: "examples/quickstart/rust/src/server.rs"
modelFile: "examples/quickstart/rust/src/models.rs"
polarFileRelative: "src/main.polar"
serverFileRelative: "src/server.rs"
modelFileRelative: "src/models.rs"
installDependencies: cargo check
startServer: cargo run
osoAuthorize: oso.is_allowed()
isPublic: is_public
hasRole: |-
  has_role(actor: User, role_name: String, repository: Repository) if
    role in actor.roles and
    role_name = role.name and
    repository = role.repository;
endpoint: the `get_repo` route
port: 5050
---
