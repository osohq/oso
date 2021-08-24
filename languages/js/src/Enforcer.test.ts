import { Enforcer } from './Enforcer';
import { Policy } from './Oso';
import { Actor, User, Widget } from '../test/classes';
import { ForbiddenError, NotFoundError, OsoError } from './errors';

describe(Enforcer, () => {
  let oso: Enforcer<Actor, String>;

  beforeEach(() => {
    const policy = new Policy();
    policy.registerClass(Actor);
    policy.registerClass(User);
    policy.registerClass(Widget);

    oso = new Enforcer(policy);
  });

  describe('#authorize', () => {
    const guest = new Actor('guest');
    const admin = new Actor('admin');
    const widget0 = new Widget('0');
    const widget1 = new Actor('admin');
    beforeEach(async () => {
      await oso.policy.loadStr(`
        allow(_actor: Actor, "read", widget: Widget) if
          widget.id = 0;

        allow(actor: Actor, "update", _widget: Widget) if
          actor.name = "admin";
      `);
    });

    test('succeeds when the actor is allowed to perform the action', async () => {
      await oso.authorize(new Actor('guest'), 'read', new Widget('0'));
      await oso.authorize(new Actor('admin'), 'update', new Widget('1'));
    });

    test('throws a ForbiddenError when the actor is allowed to read', async () => {
      expect(() => oso.authorize(guest, 'update', widget0)).rejects.toThrow(
        ForbiddenError
      );
    });

    test('throws a NotFoundError when the actor is not allowed to read', async () => {
      expect(() => oso.authorize(guest, 'read', widget1)).rejects.toThrow(
        NotFoundError
      );
      expect(() => oso.authorize(guest, 'update', widget1)).rejects.toThrow(
        NotFoundError
      );
    });
  });

  describe('#authorizedActions', () => {
    const guest = new Actor('guest');
    const admin = new Actor('admin');
    const widget0 = new Widget('0');
    const widget1 = new Actor('admin');
    beforeEach(async () => {
      await oso.policy.loadStr(`
        allow(_actor: Actor, "read", _widget: Widget);
        allow(_actor: Actor, "update", _widget: Widget{id: 0});
        allow(actor: Actor, "update", _widget: Widget) if
          actor.name = "admin";
      `);
    });

    test('returns a list of actions the user is allowed to take', async () => {
      expect(await oso.authorizedActions(guest, widget0)).toBe([
        'read',
        'update',
      ]);
      expect(await oso.authorizedActions(guest, widget1)).toBe(['read']);
      expect(await oso.authorizedActions(admin, widget1)).toBe([
        'read',
        'update',
      ]);
    });

    test('throws an OsoError if there is a wildcard action', async () => {
      await oso.policy.loadStr(`
        allow(actor, _action, _widget: Widget) if actor.name = "superadmin";
      `);
      const superadmin = new Actor('superadmin');
      expect(() => oso.authorizedActions(superadmin, widget0)).rejects.toThrow(
        OsoError
      );
    });

    test('returns a wildcard * if wildcard is explicitly allowed', async () => {
      await oso.policy.loadStr(`
        allow(actor, _action, _widget: Widget) if actor.name = "superadmin";
      `);
      const superadmin = new Actor('superadmin');
      expect(await oso.authorizedActions(superadmin, widget0, true)).toBe([
        '*',
      ]);
    });
  });

  describe('#authorizeRequest', () => {
    class Request {
      constructor(public method: string, public path: string) {}
    }
    const guest = new Actor('guest');
    const verified = new Actor('verified');

    beforeEach(async () => {
      oso.policy.loadStr(`
        allow_request(Actor{name: "guest"}, request: Request) if
            request.path.startsWith("/repos");

        allow_request(Actor{name: "verified"}, request: Request) if
            request.path.startsWith("/account");
      `);
    });

    test('throws a ForbiddenError only if request is not allowed', async () => {
      await oso.authorizeRequest(guest, new Request('GET', '/repos/1'));
      expect(() =>
        oso.authorizeRequest(guest, new Request('GET', '/other'))
      ).rejects.toThrow(ForbiddenError);

      await oso.authorizeRequest(verified, new Request('GET', '/account'));
      expect(() =>
        oso.authorizeRequest(guest, new Request('GET', '/account'))
      ).rejects.toThrow(ForbiddenError);
    });
  });

  describe('field-level authorization', () => {
    class Request {
      constructor(public method: string, public path: string) {}
    }
    const admin = new Actor('admin');
    const guest = new Actor('guest');
    const widget = new Widget('0');

    beforeEach(async () => {
      oso.policy.loadStr(`
        # Admins can update all fields
        allow_field(actor: Actor, "update", _widget: Widget, field) if
            actor.name = "admin" and
            field in ["name", "purpose", "private_field"];

        # Anybody who can update a field can also read it
        allow_field(actor, "read", widget: Widget, field) if
            allow_field(actor, "update", widget, field);

        # Anybody can read public fields
        allow_field(_: Actor, "read", _: Widget, field) if
            field in ["name", "purpose"];
      `);
    });

    test('authorizeField throws a ForbiddenError only if request is not allowed', async () => {
      await oso.authorizeField(admin, 'update', widget, 'purpose');
      expect(() =>
        oso.authorizeField(admin, 'update', widget, 'foo')
      ).rejects.toThrow(ForbiddenError);

      await oso.authorizeField(guest, 'read', widget, 'purpose');
      expect(() =>
        oso.authorizeField(guest, 'read', widget, 'private_field')
      ).rejects.toThrow(ForbiddenError);
    });

    test('authorizedFields returns a list of allowed fields', async () => {
      // Admins should be able to update all fields
      expect(new Set(await oso.authorizedFields(admin, 'update', widget))).toBe(
        new Set(['name', 'purpose', 'private_field'])
      );
      // Admins should be able to read all fields
      expect(new Set(await oso.authorizedFields(admin, 'read', widget))).toBe(
        new Set(['name', 'purpose', 'private_field'])
      );
      // Guests should not be able to update any fields
      expect(await oso.authorizedFields(guest, 'update', widget)).toHaveLength(
        0
      );
      // Guests should be able to read public fields
      expect(await oso.authorizedFields(guest, 'read', widget)).toBe(
        new Set(['name', 'purpose'])
      );
    });
  });
});

// def test_authorized_fields(test_enforcer):
//     admin = Actor(name="president")
//     guest = Actor(name="guest")
//     company = Company(id="1")
//     resource = Widget(id=company.id)
//     # Admin should be able to update all fields
//     assert set(test_enforcer.authorized_fields(admin, "update", resource)) == set(
//         ["name", "purpose", "private_field"]
//     )
//     # Guests should not be able to update fields
//     assert set(test_enforcer.authorized_fields(guest, "update", resource)) == set()
//     # Admins should be able to read all fields
//     assert set(test_enforcer.authorized_fields(admin, "read", resource)) == set(
//         ["name", "purpose", "private_field"]
//     )
//     # Guests should be able to read all public fields
//     assert set(test_enforcer.authorized_fields(guest, "read", resource)) == set(
//         ["name", "purpose"]
//     )

// def test_custom_errors():
//     class TestException(Exception):
//         def __init__(self, is_not_found):
//             self.is_not_found = is_not_found

//     policy = Policy()
//     enforcer = Enforcer(policy, get_error=lambda *args: TestException(*args))
//     with pytest.raises(TestException) as excinfo:
//         enforcer.authorize("graham", "frob", "bar")
//     assert excinfo.value.is_not_found

// def test_custom_read_action():
//     policy = Policy()
//     enforcer = Enforcer(policy, read_action="fetch")
//     with pytest.raises(AuthorizationError) as excinfo:
//         enforcer.authorize("graham", "frob", "bar")
//     assert excinfo.type == NotFoundError

//     # Allow user to "fetch" bar
//     policy.load_str("""allow("graham", "fetch", "bar");""")
//     with pytest.raises(AuthorizationError) as excinfo:
//         enforcer.authorize("graham", "frob", "bar")
//     assert excinfo.type == ForbiddenError

// if __name__ == "__main__":
//     pytest.main([__file__])
