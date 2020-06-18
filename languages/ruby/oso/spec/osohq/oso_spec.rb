# frozen_string_literal: true

RSpec.describe Osohq::Oso::Oso do
  it 'handles allow queries' do
    expect(false).to eq(true)
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

    it 'maps resources' do
      subject.load_str('
        allow(actor, "get", Http{path: path}) :=
          debug(),
          PathMapper{template: "/widget/{id}"}.map(path) = {id: id},
          allow(actor, "get", Widget{id: id});
      ')
      subject.load_str('allow(actor, "get", widget) := widget.id = 12;')

      expect(subject.allow(actor: Actor.new('sam'), action: 'get', resource: Osohq::Oso::Http.new(path: '/widget/12'))).to eq true
      expect(subject.allow(actor: Actor.new('sam'), action: 'get', resource: Osohq::Oso::Http.new(path: '/widget/13'))).to eq false
    end
  end
end
