# frozen_string_literal: true

RSpec.describe Osohq::Oso::Oso do
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
        subject.load_str('allow(u: User{}, 1, 2) := u.name = "alice";')
        allowed = subject.allow(actor: User.new(name: 'alice'), action: 1, resource: 2)
        expect(allowed).to be true
      end
    end

    context 'when a custom constructor is passed' do
      it 'registers the class with the custom constructor' do
        subject.register_class(User) do |**args|
          User.new(**args).tap { |u| u.special = true }
        end
        subject.load_str('allow(u: User{}, 1, 2) := x = new User{name: "alice"}, x.name = u.name, x.special = true;')
        allowed = subject.allow(actor: User.new(name: 'alice'), action: 1, resource: 2)
        expect(allowed).to be true
      end
    end
  end

  context 'PathMapper' do
    it 'properly maps paths' do
      mapper = Osohq::Oso::PathMapper.new(template: '/widget/{id}')
      path = '/widget/12'
      expect(mapper.map(path)).to eq({ 'id' => '12' })
    end
  end

  context 'when mapping resources' do
    before do
      stub_const('Widget', Class.new do
        attr_reader :id
        def initialize(id)
          @id = id
        end
      end)

      stub_const('Actor', Class.new do
        def initialize(n)
          @name = n
        end
      end)
      subject.register_class(Widget)
      subject.register_class(Actor)
    end

    it 'maps resources in Polar' do
      subject.load_str('test_map(path, id) := PathMapper{template: "/widget/{id}"}.map(path) = {id: id};')

      expect(subject.send(:query_pred, 'test_map', { args: ['/widget/12', '12'] }).to_a.length).to eq 1
    end

    it 'maps resources' do
      subject.load_str('
        allow(actor, "get", Http{path: path}) :=
          debug(),
          PathMapper{template: "/widget/{id}"}.map(path) = {id: id},
          allow(actor, "get", Widget{id: id});
      ')
      subject.load_str('allow(actor, "get", widget) := widget.id = "12";')

      expect(subject.allow(actor: Actor.new('sam'), action: 'get', resource: Osohq::Oso::Http.new(path: '/widget/12'))).to eq true
      expect(subject.allow(actor: Actor.new('sam'), action: 'get', resource: Osohq::Oso::Http.new(path: '/widget/13'))).to eq false
    end
  end
end
