---
authorizeRequest: authorizeRequest
forbiddenErrorLink: |-
  <a href="/java/reference/api/com/osohq/oso/Exceptions.ForbiddenException.html">`ForbiddenException`</a>
exampleCall: |-
    ```java
    public void beforeRequest(Request request) throws AuthorizationException {
        oso.authorizeRequest(request.user, request);
    }
    ```
simplePolicy: |-
    ```polar
    # Allow anyone to hit the login endpoint
    allow_request(_, _: Request{path: "/login"});

    # Only allow access to payments by users with verified emails
    allow_request(user: User, request: Request) if
        request.path.startsWith("/payments") and
        user.verifiedEmail;
    ```
exampleCallWithAccessToken: |-
    ```java
    oso.authorizeRequest(request.accessToken, request);
    ```
---
