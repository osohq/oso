---
load_file: loadFile
load_str: loadStr
files: 'new String[] {"main.polar", "repository.polar"}'
authorize: authorize
is_allowed: isAllowed
false: 'false'
authorize_migration: |
    ```java
    if (!oso.isAllowed("Ariadne", "assist", "Theseus")) {
          // handle authorization failure (probably by raising an error)
    }
    ```

    as:

    ```java
    try {
        oso.authorize("Ariadne", "assist", "Theseus");
    } catch (ForbiddenException | NotFoundException e) {
        // handle failures (probably by raising your own error types)
    }
    ```
---
