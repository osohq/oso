---
accountId: accountId
customerAccounts: customerAccounts
langName: JavaScript

accountManager: |
    ```js
    class AccountManager extends InternalUser {
      customerAccounts() {
        return db.query(
          'SELECT id FROM customer_accounts WHERE manager_id = ?',
          this.id
        );
      }
    }
    ```

actorClasses: |
    ```js
    const { Oso } = require('oso');

    const oso = new Oso();

    class Customer {
      constructor(id) {
        this.id = id;
      }
    }

    oso.registerClass(Customer);

    class InternalUser {
      constructor(id) {
        this.id = id;
      }
    }

    oso.registerClass(InternalUser);
    ```

customerDashboardHandler: |
    ```js
    async function customerDashboardHandler(request) {
      const actor = userFromId(request.id);
      return oso.isAllowed(actor, 'view', 'customer_dashboard');
    }

    function userFromId(id) {
      const userType = db.query('SELECT type FROM users WHERE id = ?', id);
      if (userType === 'internal') {
        return new InternalUser(id);
      } else if (userType === 'customer') {
        return new Customer(id);
      }
    }
    ```

generateAccountManagers: |
    ```js
    function userFromId(id) {
      const userType = db.query('SELECT type FROM users WHERE id = ?', id);
      if (userType === 'internal') {
        const actor = new InternalUser(id);
        if (actor.role() === 'account_manager') {
          return new AccountManager(id);
        } else {
          return actor;
        }
      } else if (userType === 'customer') {
        return new Customer(id);
      }
    }
    ```

internalUserRole: |
    ```js
    class InternalUser {
      ...

      role() {
        return db.query('SELECT role FROM internal_roles WHERE id = ?', this.id);
      }
    }
    ```
---
