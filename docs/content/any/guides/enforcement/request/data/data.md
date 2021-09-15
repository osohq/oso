---
authorizeRequest: authorize_request
forbiddenErrorLink: |-
  <a href="/python/reference/api/index.html#oso.exceptions.ForbiddenError">`ForbiddenError`</a>
exampleCall: |-
    ```python
    def before_request(request):
        oso.authorize_request(request.user, request)
    ```
simplePolicy: |-
    ```polar
    # Allow anyone to hit the login endpoint
    allow_request(_, _: Request{path: "/login"});

    # Only allow access to payments by users with verified emails
    allow_request(user: User, request: Request) if
        request.path.startswith("/payments") and
        user.verified_email;
    ```
exampleCallWithAccessToken: |-
    ```python
    oso.authorize_request(request.access_token, request)
    ```
---
