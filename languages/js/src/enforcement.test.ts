import { Oso } from './Oso';
import { BaseActor, User, Widget } from '../test/classes';
import { ForbiddenError, NotFoundError, OsoError } from './errors';

describe(Oso, () => {
  let oso: Oso<BaseActor, String>;

  beforeEach(() => {
    oso = new Oso();
    oso.registerClass(BaseActor);
    oso.registerClass(User);
    oso.registerClass(Widget);
  });

  describe('#authorize', () => {
    const guest = new BaseActor('guest');
    const admin = new BaseActor('admin');
    const widget0 = new Widget('0');
    const widget1 = new Widget('1');
    beforeEach(async () => {
      await oso.loadStr(`
        allow(_actor: BaseActor, "read", widget: Widget) if
          widget.id = "0";

        allow(actor: BaseActor, "update", _widget: Widget) if
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
        oso.authorize(guest, 'read', widget1, { checkRead: false })
      ).rejects.toThrow(ForbiddenError);
      await expect(
        oso.authorize(guest, 'update', widget1, { checkRead: false })
      ).rejects.toThrow(ForbiddenError);
    });
  });

  describe('#authorizedActions', () => {
    const guest = new BaseActor('guest');
    const admin = new BaseActor('admin');
    const widget0 = new Widget('0');
    const widget1 = new Widget('1');

    test('returns a list of actions the user is allowed to take', async () => {
      await oso.loadStr(`
        allow(_actor: BaseActor, "read", _widget: Widget);
        allow(_actor: BaseActor, "update", _widget: Widget{id: "0"});
        allow(actor: BaseActor, "update", _widget: Widget) if
          actor.name = "admin";
      `);
      expect(await oso.authorizedActions(guest, widget0)).toEqual(
        new Set(['read', 'update'])
      );
      expect(await oso.authorizedActions(guest, widget1)).toEqual(
        new Set(['read'])
      );
      expect(await oso.authorizedActions(admin, widget1)).toEqual(
        new Set(['read', 'update'])
      );
    });

    test('throws an OsoError if there is a wildcard action', async () => {
      await oso.loadStr(`
        allow(_actor: BaseActor, "read", _widget: Widget);
        allow(_actor: BaseActor, "update", _widget: Widget{id: "0"});
        allow(actor: BaseActor, "update", _widget: Widget) if
          actor.name = "admin";

        allow(actor, _action, _widget: Widget) if actor.name = "superadmin";
      `);
      const superadmin = new BaseActor('superadmin');
      await expect(oso.authorizedActions(superadmin, widget0)).rejects.toThrow(
        OsoError
      );
    });

    test('returns a wildcard * if wildcard is explicitly allowed', async () => {
      await oso.loadStr(`
        allow(_actor: BaseActor, "read", _widget: Widget);
        allow(_actor: BaseActor, "update", _widget: Widget{id: "0"});
        allow(actor: BaseActor, "update", _widget: Widget) if
          actor.name = "admin";

        allow(actor, _action, _widget: Widget) if actor.name = "superadmin";
      `);
      const superadmin = new BaseActor('superadmin');
      expect(
        await oso.authorizedActions(superadmin, widget0, {
          allowWildcard: true,
        })
      ).toEqual(new Set(['*']));
    });
  });

  describe('#authorizeRequest', () => {
    class Request {
      constructor(public method: string, public path: string) {}
    }
    const guest = new BaseActor('guest');
    const verified = new BaseActor('verified');

    beforeEach(async () => {
      oso.registerClass(Request);
      await oso.loadStr(`
        allow_request(_: BaseActor{name: "guest"}, request: Request) if
            request.path.startsWith("/repos");

        allow_request(_: BaseActor{name: "verified"}, request: Request) if
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
    const admin = new BaseActor('admin');
    const guest = new BaseActor('guest');
    const widget = new Widget('0');

    beforeEach(async () => {
      await oso.loadStr(`
        # Admins can update all fields
        allow_field(actor: BaseActor, "update", _widget: Widget, field) if
            actor.name = "admin" and
            field in ["name", "purpose", "private_field"];

        # Anybody who can update a field can also read it
        allow_field(actor, "read", widget: Widget, field) if
            allow_field(actor, "update", widget, field);

        # Anybody can read public fields
        allow_field(_: BaseActor, "read", _: Widget, field) if
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
      expect(await oso.authorizedFields(admin, 'update', widget)).toEqual(
        new Set(['name', 'purpose', 'private_field'])
      );
      // Admins should be able to read all fields
      expect(await oso.authorizedFields(admin, 'read', widget)).toEqual(
        new Set(['name', 'purpose', 'private_field'])
      );
      // Guests should not be able to update any fields
      expect(await oso.authorizedFields(guest, 'update', widget)).toEqual(
        new Set()
      );
      // Guests should be able to read public fields
      expect(await oso.authorizedFields(guest, 'read', widget)).toEqual(
        new Set(['name', 'purpose'])
      );
    });
  });

  describe('configuration', () => {
    test('getError overrides the error that is thrown', async () => {
      class TestNotFound extends Error {}
      class TestForbidden extends Error {}
      const oso = new Oso({
        notFoundError: TestNotFound,
        forbiddenError: TestForbidden,
      });
      oso.loadStr('allow("graham", "read", "bar");');

      await expect(oso.authorize('graham', 'frob', 'foo')).rejects.toThrow(
        TestNotFound
      );
      await expect(oso.authorize('graham', 'frob', 'bar')).rejects.toThrow(
        TestForbidden
      );
    });

    test('readAction overrides the read action used to differentiate not found and forbidden errors', async () => {
      const oso = new Oso({
        readAction: 'fetch',
      });
      await oso.loadStr('allow("graham", "fetch", "bar");');
      await expect(oso.authorize('sam', 'frob', 'bar')).rejects.toThrow(
        NotFoundError
      );
      // A user who can "fetch" should get a ForbiddenError instead of a
      // NotFoundError
      await expect(oso.authorize('graham', 'frob', 'bar')).rejects.toThrow(
        ForbiddenError
      );
    });
  });
});
