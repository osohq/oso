---
load_file: loadFile
load_str: loadStr
files: '["main.polar", "repository.polar"]'
authorize: authorize
is_allowed: isAllowed
false: 'false'
authorize_migration: |
    ```js
    if (!(await oso.isAllowed("Ariadne", "assist", "Theseus"))) {
        // handle authorization failure (probably by throwing an error)
    }
    ```

    as:

    ```js
    try {
      await oso.authorize("Ariadne", "assist", "Theseus");
    } catch (e) {
      // handle failures (probably by throwing your own error types)
    }
    ```
---
