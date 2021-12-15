# frozen_string_literal: true

FOO_REPR = 'SPECIAL FOO REPR'

RSpec.describe Oso::Polar::Polar do # rubocop:disable Metrics/BlockLength
  before do
    stub_const('Foo', Class.new do
      def to_s
        FOO_REPR
      end
    end)
    subject.register_class(Foo)
  end

  context 'Instance reprs' do # rubocop:disable Metrics/BlockLength
    it 'test_repr_when_logging' do
      old_polar_log = ENV['POLAR_LOG']
      ENV['POLAR_LOG'] = '1'
      subject.load_str('f(_foo: Foo) if 1 = 1;')
      expect do
        subject.query_rule('f', Foo.new).to_a
      end.to output(/QUERY RULE: f\(#{FOO_REPR} TYPE `Foo`\)/).to_stdout
      ENV.delete('POLAR_LOG') unless old_polar_log
    end

    it 'test_repr_in_error' do
      # This will throw an error because foo.hello is not allowed
      subject.load_str('f(foo: Foo) if foo.hello;')
      expect do
        subject.query_rule('f', Foo.new).to_a
      end.to raise_error(
        an_instance_of(Oso::Polar::PolarRuntimeError)
        .and(having_attributes(message: /f\(#{FOO_REPR} TYPE `Foo`\)/))
      )
    end

    it 'test_repr_when_debugging' do
      subject.load_str('f(_foo: Foo) if debug() and 1 = 1;')
      input = StringIO.new("bindings\n")
      $stdin = input
      expect do
        subject.query_rule('f', Foo.new).to_a
      end.to output(/_foo_[0-9]+ = #{FOO_REPR} TYPE `Foo`/).to_stdout
      $stdin = STDIN
    end
  end
end
