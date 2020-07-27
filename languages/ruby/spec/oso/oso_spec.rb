# frozen_string_literal: true

RSpec.describe Oso::Oso do
  context '#register_class' do
    before do
      stub_const('User', Class.new do
        attr_accessor :name, :special
        def initialize(name:)
          @name = name
          @special = false
        end
      end)
    end

    context 'when no constructor is passed' do
      it 'registers the class with the default constructor' do
        subject.register_class(User)
        subject.load_str('allow(u: User{}, 1, 2) if u.name = "alice";')
        allowed = subject.allowed?(actor: User.new(name: 'alice'), action: 1, resource: 2)
        expect(allowed).to be true
      end
    end

    context 'when a custom constructor is passed' do
      it 'registers the class with the custom constructor' do
        subject.register_class(User) do |**args|
          User.new(**args).tap { |u| u.special = true }
        end
        subject.load_str('allow(u: User{}, 1, 2) if x = new User{name: "alice"} and x.name = u.name and x.special = true;')
        allowed = subject.allowed?(actor: User.new(name: 'alice'), action: 1, resource: 2)
        expect(allowed).to be true
      end
    end
  end

  context '#allow' do
    it 'controls access appropriately' do
      subject.load_str('allow(1, 2, 3);')
      allowed = subject.allowed?(actor: 1, action: 2, resource: 3)
      expect(allowed).to be true
      allowed = subject.allowed?(actor: 3, action: 2, resource: 1)
      expect(allowed).to be false
    end
  end

  context '#query_rule' do
    it 'calls through to the allow rule' do
      subject.load_str('allow(1, 2, 3);')
      result = subject.query_rule('allow', 1, 2, 3)
      expect(result.next).to eq({})
    end
  end

  context 'Extras' do
    context 'PathMapper' do
      context '#map' do
        it 'extracts matches into a hash' do
          mapper = Oso::PathMapper.new(template: '/widget/{id}')
          expect(mapper.map('/widget/12')).to eq({ 'id' => '12' })
          expect(mapper.map('/widget/12/frob')).to eq({})
        end
      end
    end

    context 'PathMapper + Http' do
      it 'can map Http resources' do
        stub_const('Widget', Class.new do
          attr_reader :id
          def initialize(id:)
            @id = id
          end
        end)
        subject.register_class(Widget)
        subject.load_str <<~POLAR
          allow(actor, "get", _: Http{path: path}) if
              new PathMapper{template: "/widget/{id}"}.map(path) = {id: id} and
              allow(actor, "get", new Widget{id: id});
          allow(_actor, "get", widget: Widget{}) if widget.id = "12";
        POLAR
        widget12 = Oso::Http.new(path: '/widget/12')
        allowed = subject.allowed?(actor: 'sam', action: 'get', resource: widget12)
        expect(allowed).to eq true
        widget13 = Oso::Http.new(path: '/widget/13')
        expect(subject.allowed?(actor: 'sam', action: 'get', resource: widget13)).to eq false
      end
    end
  end
end
