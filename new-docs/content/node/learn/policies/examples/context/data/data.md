---
envClass: |
    ```js
    class Env {
      static var(variable) {
        return process.env[variable];
      }
    }

    oso.registerClass(Env);
    ```
---
