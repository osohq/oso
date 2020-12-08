# frozen_string_literal: true

RSpec.describe Oso::Oso do # rubocop:disable Metrics/BlockLength
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

    it 'registers the class' do
      subject.register_class(User)
      subject.load_str('allow(u: User{}, 1, 2) if u.name = "alice";')
      allowed = subject.allowed?(actor: User.new(name: 'alice'), action: 1, resource: 2)
      expect(allowed).to be true
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
end
