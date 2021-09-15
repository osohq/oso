---
load_file: load_file
load_str: load_str
files: '["main.polar", "repository.polar"]'
authorize: authorize
is_allowed: is_allowed
false: 'False'
authorize_migration: |
    ```py
    if not oso.is_allowed("Ariadne", "assist", "Theseus"):
          # handle authorization failure (probably by raising an error)
    ```

    as:

    ```py
    try:
        oso.authorize("Ariadne", "read", "Theseus")
    except (ForbiddenError, NotFoundError) as e:
        # handle failures (probably by raising your own error types)
    ```
---
