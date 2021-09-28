---
authorizeRequest: authorizeRequest
forbiddenErrorLink: |-
  <a href="/node/reference/api/classes/errors.forbiddenerror.html">`ForbiddenError`</a>
exampleCall: |-
    ```javascript
    app.use(function(req, res, next) {
      oso.authorizeRequest(req.user, req);
      next();
    });
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
    ```javascript
    oso.authorizeRequest(request.accessToken, request)
    ```
---
