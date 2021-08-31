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
    class Page:
      def __init__(self, contents):
          self.contents = contents

    class User:
        def __init__(self, role):
            self.role = role

    oso.register_class(Page)
    ```

app_code: |
    ```python
    page = Page("a readable page")
    if oso.is_allowed(
        User(),  # the user doing the request
        "read",  # the action we want to do
        page,  # the resource we want to do it to
    ):
        print(page.content)
    else:
        raise Exception("Forbidden")
    ```

assert_code: |
    ```python
    assert oso.is_allowed(User(role="guest"), "read", Page("readable page"))
    assert not oso.is_allowed(User(role="guest", "write", Page("readable page"))
    assert oso.is_allowed(User(role="admin", "write", Page("readable page"))
    ```
---
