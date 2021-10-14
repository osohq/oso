---
load_file: LoadFile
load_str: LoadString
files: '[]string{"main.polar", "repository.polar"}'
authorize: Authorize
is_allowed: IsAllowed
false: 'false'
authorize_migration: |
    ```go
    if allowed, err := oso.IsAllowed("Ariadne", "assist", "Theseus"); err != nil {
          // handle non-authorization failure (probably by returning an error)
    } else if !allowed {
          // handle authorization failure (probably by returning an error)
    }
    ```

    as:

    ```go
    if err := oso.Authorize("Ariadne", "assist", "Theseus"); err != nil {
        // handle failures (probably by returning your own error types)
    }
    ```
---
