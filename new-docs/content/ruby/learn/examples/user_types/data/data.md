---
accountId: account_id
customerAccounts: customer_accounts
langName: Ruby

accountManager: |
    ```ruby
    class AccountManager < InternalUser
      def customer_accounts
        db.query('SELECT id FROM customer_accounts WHERE manager_id = ?', id)
      end
    end
    ```

actorClasses: |
    ```ruby
    require 'oso'

    OSO ||= Oso.new

    class Customer
      def initialize(id)
        @id = id
      end
    end

    OSO.register_class(Customer)

    class InternalUser
      def initialize(id)
        @id = id
      end
    end

    OSO.register_class(InternalUser)
    ```

customerDashboardHandler: |
    ```ruby
    def customer_dashboard_handler(request)
      actor = user_from_id(request.id)
      OSO.allowed?(actor: actor, action: 'view', resource: 'customer_dashboard')
    end

    def user_from_id(id)
      case db.query('SELECT type FROM users WHERE id = ?', id)
      when 'internal'
        InternalUser.new(id: id)
      when 'customer'
        Customer.new(id: id)
      end
    end
    ```

generateAccountManagers: |
    ```ruby
    def user_from_id(id)
      case db.query('SELECT type FROM users WHERE id = ?', id)
      when 'internal'
        actor = InternalUser.new(id: id)
        if actor.role == 'account_manager'
          AccountManager.new(id: id)
        else
          actor
        end
      when 'customer'
        Customer.new(id: id)
      end
    end
    ```

internalUserRole: |
    ```ruby
    class InternalUser
      ...

      def role
        db.query('SELECT role FROM internal_roles WHERE id = ?', id)
      end
    end
    ```
---
