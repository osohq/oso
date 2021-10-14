---
authorizeRequest: authorize_request
forbiddenErrorLink: |-
  <a href="/ruby/reference/api/Oso/ForbiddenError.html">`ForbiddenError`</a>
exampleCall: |-
    ```ruby
    def before_request(request)
      oso.authorize_request(current_user, request)
    end
    ```
simplePolicy: |-
    ```polar
    # Allow anyone to hit the login endpoint
    allow_request(_, _: Request{path: "/login"});

    # Only allow access to payments by users with verified emails
    allow_request(user: User, request: Request) if
        request.path.start_with?("/payments") and
        user.verified_email;
    ```
exampleCallWithAccessToken: |-
    ```ruby
    oso.authorize_request(current_access_token, request)
    ```
---
