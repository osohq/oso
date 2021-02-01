---
langName: Node.js

userClass: |
    ```js
    const oso = new Oso();

    class User {
      constructor(name) {
        this.name = name;
      }

      role() {
        return db.query('SELECT role FROM user_roles WHERE username = ?', [
          this.name,
        ]);
      }
    }

    oso.registerClass(User);
    ```
---
