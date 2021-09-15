---
load_file: load_file
load_str: load_str
files: '["main.polar", "repository.polar"]'
authorize: authorize
is_allowed: allowed?
false: 'false'
authorize_migration: |
    ```rb
    unless oso.allowed?(actor: "Ariadne", action: "assist", resource: "Theseus")
        # handle authorization failure (probably by raising an error)
    end
    ```

    as:

    ```rb
    begin
      oso.authorize("Ariadne", "assist", "Theseus")
    rescue Oso::ForbiddenError, Oso::NotFoundError
      # handle failures (probably by raising your own error types)
    end
    ```
---
