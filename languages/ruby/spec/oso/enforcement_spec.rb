# rubocop:disable Metrics/BlockLength
# frozen_string_literal: true

module EnforcementSpec
  class User
    attr_reader :name

    def initialize(name)
      @name = name
    end
  end

  class Widget
    attr_reader :id

    def initialize(id)
      @id = id
    end
  end
end

RSpec.describe Oso::Oso do
  let(:oso) do
    oso = Oso::Oso.new
    oso.register_class(User, name: 'User')
    oso.register_class(Widget, name: 'Widget')
    oso
  end

  before(:each) do
    stub_const('User', EnforcementSpec::User)
    stub_const('Widget', EnforcementSpec::Widget)
  end

  context '#authorize' do
    let(:guest) { User.new('guest') }
    let(:admin) { User.new('admin') }
    let(:widget0) { Widget.new('0') }
    let(:widget1) { Widget.new('1') }
    before(:each) do
      oso.load_str(%|
        allow(_actor: User, "read", widget: Widget) if
          widget.id = "0";
        allow(actor: User, "update", _widget: Widget) if
          actor.name = "admin";
      |)
    end

    it 'succeeds when the actor is allowed to perform the action' do
      oso.authorize(guest, 'read', widget0)
      oso.authorize(admin, 'update', widget1)
    end

    it 'throws a ForbiddenError when the actor is allowed to read' do
      expect { oso.authorize(guest, 'update', widget0) }.to raise_error(Oso::ForbiddenError)
    end

    it 'throws a NotFoundError when the actor is not allowed to read' do
      expect { oso.authorize(guest, 'read', widget1) }.to raise_error(Oso::NotFoundError)
      expect { oso.authorize(guest, 'update', widget1) }.to raise_error(Oso::NotFoundError)
    end

    it 'throws a ForbiddenError when check_read is false' do
      expect { oso.authorize(guest, 'read', widget1, check_read: false) }.to raise_error(Oso::ForbiddenError)
      expect { oso.authorize(guest, 'update', widget1, check_read: false) }.to raise_error(Oso::ForbiddenError)
    end
  end

  context '#authorized_actions' do
    let(:guest) { User.new('guest') }
    let(:admin) { User.new('admin') }
    let(:widget0) { Widget.new('0') }
    let(:widget1) { Widget.new('1') }
    before(:each) do
      oso.load_str(%|
        allow(_actor: User, "read", _widget: Widget);
        allow(_actor: User, "update", _widget: Widget{id: "0"});
        allow(actor: User, "update", _widget: Widget) if
          actor.name = "admin";
      |)
    end

    it 'returns a list of actions the user is allowed to take' do
      expect(oso.authorized_actions(guest, widget0).to_a).to match_array(%w[read update])
      expect(oso.authorized_actions(guest, widget1).to_a).to match_array(%w[read])
      expect(oso.authorized_actions(admin, widget1).to_a).to match_array(%w[read update])
    end

    it 'throws an Oso::Error if there is a wildcard action' do
      oso.clear_rules
      oso.load_str(%|
        allow(_actor: User, "read", _widget: Widget);
        allow(_actor: User, "update", _widget: Widget{id: "0"});
        allow(actor: User, "update", _widget: Widget) if
          actor.name = "admin";
        allow(actor, _action, _widget: Widget) if actor.name = "superadmin";
      |)
      superadmin = User.new('superadmin')
      expect { oso.authorized_actions(superadmin, widget0) }.to raise_error(Oso::Error)
    end

    it 'returns a wildcard * if wildcard is explicitly allowed' do
      oso.clear_rules
      oso.load_str(%|
        allow(_actor: User, "read", _widget: Widget);
        allow(_actor: User, "update", _widget: Widget{id: "0"});
        allow(actor: User, "update", _widget: Widget) if
          actor.name = "admin";
        allow(actor, _action, _widget: Widget) if actor.name = "superadmin";
      |)
      superadmin = User.new('superadmin')
      expect(oso.authorized_actions(superadmin, widget0, allow_wildcard: true).to_a).to eq(['*'])
    end
  end

  context '#authorize_request' do
    class Request
      attr_reader :method, :path

      def initialize(method, path)
        @method = method
        @path = path
      end
    end

    let(:guest) { User.new('guest') }
    let(:verified) { User.new('verified') }

    before(:each) do
      oso.register_class(Request)
      oso.load_str(%|
        allow_request(_: User{name: "guest"}, request: Request) if
            request.path.start_with?("/repos");
        allow_request(_: User{name: "verified"}, request: Request) if
            request.path.start_with?("/account");
      |)
    end

    it 'throws a ForbiddenError only if request is not allowed' do
      oso.authorize_request(guest, Request.new('GET', '/repos/1'))
      expect do
        oso.authorize_request(guest, Request.new('GET', '/other'))
      end.to raise_error(Oso::ForbiddenError)

      oso.authorize_request(verified, Request.new('GET', '/account'))
      expect do
        oso.authorize_request(guest, Request.new('GET', '/account'))
      end.to raise_error(Oso::ForbiddenError)
    end
  end

  context 'field-level authorization' do
    let(:admin) { User.new('admin') }
    let(:guest) { User.new('guest') }
    let(:widget) { Widget.new('0') }

    before(:each) do
      oso.load_str(%|
        # Admins can update all fields
        allow_field(actor: User, "update", _widget: Widget, field) if
            actor.name = "admin" and
            field in ["name", "purpose", "private_field"];
        # Anybody who can update a field can also read it
        allow_field(actor, "read", widget: Widget, field) if
            allow_field(actor, "update", widget, field);
        # Anybody can read public fields
        allow_field(_: User, "read", _: Widget, field) if
            field in ["name", "purpose"];
      |)
    end

    it 'authorize_field throws a ForbiddenError only if request is not allowed' do
      oso.authorize_field(admin, 'update', widget, 'purpose')
      expect do
        oso.authorize_field(admin, 'update', widget, 'foo')
      end.to raise_error(Oso::ForbiddenError)

      oso.authorize_field(guest, 'read', widget, 'purpose')
      expect do
        oso.authorize_field(guest, 'read', widget, 'private_field')
      end.to raise_error(Oso::ForbiddenError)
    end

    it 'authorized_fields returns a list of allowed fields' do
      # Admins should be able to update all fields
      expect(oso.authorized_fields(admin, 'update', widget).to_a).to match_array(%w[name purpose private_field])
      # Admins should be able to read all fields
      expect(oso.authorized_fields(admin, 'read', widget).to_a).to match_array(%w[name purpose private_field])
      # Guests should not be able to update any fields
      expect(oso.authorized_fields(guest, 'update', widget).to_a).to eq([])
      # Guests should be able to read public fields
      expect(oso.authorized_fields(guest, 'read', widget).to_a).to match_array(%w[name purpose])
    end
  end

  context 'configuration' do
    it 'can override the error that is thrown' do
      class TestNotFound < StandardError
      end
      class TestForbidden < StandardError
      end

      oso = Oso::Oso.new(
        not_found_error: TestNotFound,
        forbidden_error: TestForbidden
      )

      oso.load_str('allow("graham", "read", "bar");')
      expect { oso.authorize('sam', 'frob', 'bar') }.to raise_error(TestNotFound)
      expect { oso.authorize('graham', 'frob', 'bar') }.to raise_error(TestForbidden)
    end

    it 'read_action overrides the read action used to differentiate not found and forbidden errors' do
      oso = Oso::Oso.new(read_action: 'fetch')
      oso.load_str('allow("graham", "fetch", "bar");')
      expect { oso.authorize('sam', 'frob', 'bar') }.to raise_error(Oso::NotFoundError)
      # A user who can "fetch" should get a ForbiddenError instead of a
      # NotFoundError
      expect { oso.authorize('graham', 'frob', 'bar') }.to raise_error(Oso::ForbiddenError)
    end
  end
end

# rubocop:enable Metrics/BlockLength
