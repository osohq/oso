---
authorizeRequest: AuthorizeRequest
forbiddenErrorLink: |-
  <a href="https://pkg.go.dev/github.com/osohq/go-oso/errors#ForbiddenError">`ForbiddenError`</a>
exampleCall: |-
    ```go
    func BeforeRequest(request Request) error {
        if err := oso.AuthorizeRequest(request.User, request); err != nil {
            // handle error
        }
    }
    ```
simplePolicy: |-
    ```polar
    # Allow anyone to hit the login endpoint
    allow_request(_, _: Request{Path: "/login"});

    # Only allow access to payments by users with verified emails
    allow_request(user: User, request: Request) if
        request.Path.StartsWith("/payments") and
        user.VerifiedEmail;
    ```
exampleCallWithAccessToken: |-
    ```go
    err := oso.AuthorizeRequest(request.AccessToken, request)
    ```
---
