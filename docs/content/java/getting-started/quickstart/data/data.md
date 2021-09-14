---
githubUrl: "https://github.com/osohq/oso-java-quickstart"
githubCloneUrl: "https://github.com/osohq/oso-java-quickstart.git"
repoName: oso-java-quickstart
mainPolarFile: "examples/quickstart/java/src/main/java/quickstart/main.polar"
serverFile: "examples/quickstart/java/src/main/java/quickstart/Server.java"
modelFile: "examples/quickstart/java/src/main/java/quickstart/Models.java"
polarFileRelative: "main.polar"
serverFileRelative: "Server.java"
modelFileRelative: "Models.java"
installDependencies: "mvn install # requires maven to be installed!"
startServer: mvn clean package exec:java -Dexec.mainClass="quickstart.Server"
osoAuthorize: oso.authorize()
isPublic: isPublic
hasRole: |-
  has_role(actor: User, role_name: String, repository: Repository) if
    role in actor.roles and
    role_name = role.name and
    repository = role.repository;
endpoint: the `/repo/{name}` route
port: 5000
---
