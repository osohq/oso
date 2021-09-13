---
# This is copied from python so that the build doesn't break.
githubUrl: "https://github.com/osohq/oso-python-quickstart"
githubCloneUrl: "https://github.com/osohq/oso-python-quickstart.git"
repoName: oso-python-quickstart
mainPolarFile: "examples/quickstart/python/app/main.polar"
serverFile: "examples/quickstart/python/app/server.py"
modelFile: "examples/quickstart/python/app/models.py"
polarFileRelative: "app/main.polar"
serverFileRelative: "app/server.py"
modelFileRelative: "app/models.py"
installDependencies: pip install -r requirements.txt
startServer: FLASK_APP=app.server python -m flask run
osoAuthorize: oso.authorize()
isPublic: is_public
endpoint: the `repo_show` route
port: 5000
---
