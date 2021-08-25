# rubocop:disable Metrics/BlockLength
# frozen_string_literal: true

class Actor
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

RSpec.describe Oso::Enforcer do
  let(:oso) do
    policy = Oso::Policy.new
    policy.register_class(Actor)
    policy.register_class(Widget)

    Oso::Enforcer.new(policy)
  end

  context '#authorize' do
    guest = Actor.new('guest')
    admin = Actor.new('admin')
    widget0 = Widget.new('0')
    widget1 = Widget.new('1')
    before(:each) do
      oso.policy.load_str(%|
        allow(_actor: Actor, "read", widget: Widget) if
          widget.id = "0";
        allow(actor: Actor, "update", _widget: Widget) if
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
  end

  context '#authorized_actions' do
    guest = Actor.new('guest')
    admin = Actor.new('admin')
    widget0 = Widget.new('0')
    widget1 = Widget.new('1')
    before(:each) do
      oso.policy.load_str(%|
        allow(_actor: Actor, "read", _widget: Widget);
        allow(_actor: Actor, "update", _widget: Widget{id: "0"});
        allow(actor: Actor, "update", _widget: Widget) if
          actor.name = "admin";
      |)
    end

    it 'returns a list of actions the user is allowed to take' do
      expect(oso.authorized_actions(guest, widget0)).to match_array(%w[read update])
      expect(oso.authorized_actions(guest, widget1)).to match_array(%w[read])
      expect(oso.authorized_actions(admin, widget1)).to match_array(%w[read update])
    end

    it 'throws an Oso::Error if there is a wildcard action' do
      oso.policy.load_str(%|
        allow(actor, _action, _widget: Widget) if actor.name = "superadmin";
      |)
      superadmin = Actor.new('superadmin')
      expect { oso.authorized_actions(superadmin, widget0) }.to raise_error(Oso::Error)
    end

    it 'returns a wildcard * if wildcard is explicitly allowed' do
      oso.policy.load_str(%|
        allow(actor, _action, _widget: Widget) if actor.name = "superadmin";
      |)
      superadmin = Actor.new('superadmin')
      expect(oso.authorized_actions(superadmin, widget0, allow_wildcard: true)).to eq(['*'])
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

    guest = Actor.new('guest')
    verified = Actor.new('verified')

    before(:each) do
      oso.policy.register_class(Request)
      oso.policy.load_str(%|
        allow_request(_: Actor{name: "guest"}, request: Request) if
            request.path.start_with?("/repos");
        allow_request(_: Actor{name: "verified"}, request: Request) if
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
    admin = Actor.new('admin')
    guest = Actor.new('guest')
    widget = Widget.new('0')

    before(:each) do
      oso.policy.load_str(%|
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
      expect(oso.authorized_fields(admin, 'update', widget)).to match_array(%w[name purpose private_field])
      # Admins should be able to read all fields
      expect(oso.authorized_fields(admin, 'read', widget)).to match_array(%w[name purpose private_field])
      # Guests should not be able to update any fields
      expect(oso.authorized_fields(guest, 'update', widget)).to eq([])
      # Guests should be able to read public fields
      expect(oso.authorized_fields(guest, 'read', widget)).to match_array(%w[name purpose])
    end
  end

  context 'configuration' do
    it 'get_error overrides the error that is thrown' do
      class TestError < StandardError
        attr_reader :is_not_found

        def initialize(is_not_found)
          super()
          @is_not_found = is_not_found
        end
      end

      policy = Oso::Policy.new
      enforcer = Oso::Enforcer.new(
        policy,
        get_error: ->(is_not_found) { TestError.new(is_not_found) }
      )

      expect { enforcer.authorize('graham', 'frob', 'bar') }.to raise_error(
        an_instance_of(TestError).and(having_attributes({ is_not_found: true }))
      )
    end

    it 'read_action overrides the read action used to differentiate not found and forbidden errors' do
      policy = Oso::Policy.new
      enforcer = Oso::Enforcer.new(policy, read_action: 'fetch')
      enforcer.policy.load_str('allow("graham", "fetch", "bar");')
      expect { enforcer.authorize('sam', 'frob', 'bar') }.to raise_error(Oso::NotFoundError)
      # A user who can "fetch" should get a ForbiddenError instead of a
      # NotFoundError
      expect { enforcer.authorize('graham', 'frob', 'bar') }.to raise_error(Oso::ForbiddenError)
    end
  end
end

# rubocop:enable Metrics/BlockLength
