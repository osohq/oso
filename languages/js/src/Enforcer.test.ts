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
    const widget1 = new Widget('1');
    beforeEach(async () => {
      await oso.policy.loadStr(`
        allow(_actor: Actor, "read", widget: Widget) if
          widget.id = "0";

        allow(actor: Actor, "update", _widget: Widget) if
          actor.name = "admin";
      `);
    });

    test('succeeds when the actor is allowed to perform the action', async () => {
      await oso.authorize(guest, 'read', widget0);
      await oso.authorize(admin, 'update', widget1);
    });

    test('throws a ForbiddenError when the actor is allowed to read', async () => {
      await expect(oso.authorize(guest, 'update', widget0)).rejects.toThrow(
        ForbiddenError
      );
    });

    test('throws a NotFoundError when the actor is not allowed to read, unless skipped', async () => {
      await expect(oso.authorize(guest, 'read', widget1)).rejects.toThrow(
        NotFoundError
      );
      await expect(oso.authorize(guest, 'update', widget1)).rejects.toThrow(
        NotFoundError
      );
      await expect(
        oso.authorize(guest, 'update', widget1, { checkRead: false })
      ).rejects.toThrow(ForbiddenError);
    });
  });

  describe('#authorizedActions', () => {
    const guest = new Actor('guest');
    const admin = new Actor('admin');
    const widget0 = new Widget('0');
    const widget1 = new Widget('1');
    beforeEach(async () => {
      await oso.policy.loadStr(`
        allow(_actor: Actor, "read", _widget: Widget);
        allow(_actor: Actor, "update", _widget: Widget{id: "0"});
        allow(actor: Actor, "update", _widget: Widget) if
          actor.name = "admin";
      `);
    });

    test('returns a list of actions the user is allowed to take', async () => {
      expect(new Set(await oso.authorizedActions(guest, widget0))).toEqual(
        new Set(['read', 'update'])
      );
      expect(new Set(await oso.authorizedActions(guest, widget1))).toEqual(
        new Set(['read'])
      );
      expect(new Set(await oso.authorizedActions(admin, widget1))).toEqual(
        new Set(['read', 'update'])
      );
    });

    test('throws an OsoError if there is a wildcard action', async () => {
      await oso.policy.loadStr(`
        allow(actor, _action, _widget: Widget) if actor.name = "superadmin";
      `);
      const superadmin = new Actor('superadmin');
      await expect(oso.authorizedActions(superadmin, widget0)).rejects.toThrow(
        OsoError
      );
    });

    test('returns a wildcard * if wildcard is explicitly allowed', async () => {
      await oso.policy.loadStr(`
        allow(actor, _action, _widget: Widget) if actor.name = "superadmin";
      `);
      const superadmin = new Actor('superadmin');
      expect(
        await oso.authorizedActions(superadmin, widget0, {
          allowWildcard: true,
        })
      ).toEqual(['*']);
    });
  });

  describe('#authorizeRequest', () => {
    class Request {
      constructor(public method: string, public path: string) {}
    }
    const guest = new Actor('guest');
    const verified = new Actor('verified');

    beforeEach(async () => {
      oso.policy.registerClass(Request);
      await oso.policy.loadStr(`
        allow_request(_: Actor{name: "guest"}, request: Request) if
            request.path.startsWith("/repos");

        allow_request(_: Actor{name: "verified"}, request: Request) if
            request.path.startsWith("/account");
      `);
    });

    test('throws a ForbiddenError only if request is not allowed', async () => {
      await oso.authorizeRequest(guest, new Request('GET', '/repos/1'));
      await expect(() =>
        oso.authorizeRequest(guest, new Request('GET', '/other'))
      ).rejects.toThrow(ForbiddenError);

      await oso.authorizeRequest(verified, new Request('GET', '/account'));
      await expect(() =>
        oso.authorizeRequest(guest, new Request('GET', '/account'))
      ).rejects.toThrow(ForbiddenError);
    });
  });

  describe('field-level authorization', () => {
    const admin = new Actor('admin');
    const guest = new Actor('guest');
    const widget = new Widget('0');

    beforeEach(async () => {
      await oso.policy.loadStr(`
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
      await expect(() =>
        oso.authorizeField(admin, 'update', widget, 'foo')
      ).rejects.toThrow(ForbiddenError);

      await oso.authorizeField(guest, 'read', widget, 'purpose');
      await expect(() =>
        oso.authorizeField(guest, 'read', widget, 'private_field')
      ).rejects.toThrow(ForbiddenError);
    });

    test('authorizedFields returns a list of allowed fields', async () => {
      // Admins should be able to update all fields
      expect(
        new Set(await oso.authorizedFields(admin, 'update', widget))
      ).toEqual(new Set(['name', 'purpose', 'private_field']));
      // Admins should be able to read all fields
      expect(
        new Set(await oso.authorizedFields(admin, 'read', widget))
      ).toEqual(new Set(['name', 'purpose', 'private_field']));
      // Guests should not be able to update any fields
      expect(await oso.authorizedFields(guest, 'update', widget)).toHaveLength(
        0
      );
      // Guests should be able to read public fields
      expect(
        new Set(await oso.authorizedFields(guest, 'read', widget))
      ).toEqual(new Set(['name', 'purpose']));
    });
  });

  describe('configuration', () => {
    test('getError overrides the error that is thrown', async () => {
      class TestNotFound extends Error {}
      class TestForbidden extends Error {}
      const policy = new Policy();
      policy.loadStr(`allow("graham", "read", "bar");`);
      const enforcer = new Enforcer(policy, {
        notFoundError: TestNotFound,
        forbiddenError: TestForbidden,
      });

      await expect(enforcer.authorize('graham', 'frob', 'foo')).rejects.toThrow(
        TestNotFound
      );
      await expect(enforcer.authorize('graham', 'frob', 'bar')).rejects.toThrow(
        TestForbidden
      );
    });

    test('readAction overrides the read action used to differentiate not found and forbidden errors', async () => {
      const policy = new Policy();
      const enforcer = new Enforcer(policy, {
        readAction: 'fetch',
      });
      await policy.loadStr(`allow("graham", "fetch", "bar");`);
      await expect(enforcer.authorize('sam', 'frob', 'bar')).rejects.toThrow(
        NotFoundError
      );
      // A user who can "fetch" should get a ForbiddenError instead of a
      // NotFoundError
      await expect(enforcer.authorize('graham', 'frob', 'bar')).rejects.toThrow(
        ForbiddenError
      );
    });
  });
});
