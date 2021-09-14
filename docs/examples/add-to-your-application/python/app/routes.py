from flask import Flask
from oso import NotFoundError
from .models import users_db, Repository
from .oso import oso


app = Flask(__name__)

def serialize(r):
    return str(r)

# implemented here to not polute code samples in model.md
class User:
    @staticmethod
    def get_current_user():
        return users_db['larry']

# docs: begin-show-route
@app.route("/repo/<name>")
def repo_show(name):
    repo = Repository.get_by_name(name)

    try:
        oso.authorize(actor=User.get_current_user(),
                      action="read",
                      resource=repo)
        return f"<h1>A Repo</h1><p>Welcome to repo {repo.name}</p>", 200
    except NotFoundError:
        return f"<h1>Whoops!</h1><p>Repo named {name} was not found</p>", 404
# docs: end-show-route
