---
githubApp: "[Python sample app](https://github.com/osohq/oso-python-quickstart)"
githubURL: "https://github.com/osohq/oso-python-quickstart.git"
installation: |
    Install the project dependencies with pip, then run the server:
    ```bash
    $ pip install -r requirements.txt
    installing requirements
    
    $ python server.py
    server running on port 5050
    ```
amount: amount
manager: manager
submitted_by: submitted_by
endswith: endswith
endswithURL: >
   [the `str.endswith` method](https://docs.python.org/3/library/stdtypes.html#str.endswith)
expensesPath1: examples/quickstart/polar/expenses-01-python.polar
expensesPath2: examples/quickstart/polar/expenses-02-python.polar
isAllowed: is_allowed
installation_new: |
    ```bash
    pip install --upgrade oso
    ```
import: import
import_code: |
    ```python
    from oso import Oso
    oso = Oso()
    oso.enable_roles()
    ```
example_app: a Flask
load_policy: |
    ```python
    oso.load_file("authorization.polar")
    ```
getroles: get_roles
classes: Python classes
objects: Python objects
methods: Python methods
register_classes: |
    ```python
    oso.register_class(Page)
    oso.register_class(User)
    ```
app_code: |
    ```python
    from flask import Flask

    app = Flask(__name__)
    @app.route("/page/<pagenum>")
    def page_show(pagenum):
        page = Page.get_page(pagenum)
        if oso.is_allowed(
            User.get_current_user(),  # the user doing the request
            "read",  # the action we want to do
            page,  # the resource we want to do it to
        ):
            return f"<h1>A Page</h1><p>this is page {pagenum}</p>", 200
        else:
            return f"<h1>Sorry</h1><p>You are not allowed to see this page</p>", 403
    ```
---
