# frozen_string_literal: true

require 'tempfile'

require_relative './helpers'

Bar = Struct.new :id, :is_cool, :is_still_cool
Foo = Struct.new :id, :bar_id, :is_fooey, :numbers

Foo.class_eval do
  define_method(:bar) {
    Bars.find {|bar| bar.id == bar_id }
  }
end

Foos = [
  Foo.new('something', 'hello', false, []),
  Foo.new('another', 'hello',true, [1]),
  Foo.new('third', 'hello',true, [2]),
  Foo.new('fourth', 'goodbye', true, [2, 1]),
]

Bars = [
  Bar.new('hello', true, true),
  Bar.new('goodbye', false, true),
  Bar.new('hershey', false, false),
]



RSpec.configure do |c|
  c.include Helpers
end

RSpec.describe Oso::Polar::Polar do # rubocop:disable Metrics/BlockLength
  let(:test_file) { File.join(__dir__, 'test_file.polar') }
  let(:test_file_gx) { File.join(__dir__, 'test_file_gx.polar') }

  context 'data filtering' do
    context 'when filtering known values' do
      it 'works' do
        subject.load_str('allow(_, _, i) if i in [1, 2];')
        subject.load_str('allow(_, _, i) if i = {};')

        expect(subject.get_allowed_resources('gwen', 'get', Integer)).to eq([1,2])
        expect(subject.get_allowed_resources('gwen', 'get', Hash)).to eq([{}])
      end
    end

    context 'when filtering unknown values' do
      before do
        fetcher_for = ->(arr) { ->(cs) { arr.select {|x| cs.all? {|c| c.to_predicate[x] } } } }
        subject.register_class(
          Bar,
          fields: {'id' => String, 'is_cool' => PolarBoolean, 'is_still_cool' => PolarBoolean },
          fetcher: fetcher_for[Bars]
        )

        subject.register_class(
          Foo,
          fields: {'id' => String, 'bar_id' => String, 'is_fooey' => PolarBoolean, 'numbers' => Array,
                   'bar' => ::Oso::Polar::DataFiltering::Relationship.new(kind: 'parent', other_type: 'Bar', my_field: 'bar_id', other_field: 'id')
        },
          fetcher: fetcher_for[Foos]
        )
      end

      context 'without relationships' do
        it 'works' do
          policy = 'allow("gwen", "get", foo: Foo) if foo.is_fooey = true;'
          subject.load_str(policy)
          results = subject.get_allowed_resources('gwen', 'get', Foo)
          expected = Foos.select(&:is_fooey)
          expect(expected).not_to be_empty
          expect(unord_eq results, expected).to be true
        end
      end

      context 'with relationships' do
        it 'works' do
          policy = 'allow("gwen", "get", foo: Foo) if foo.bar = bar and bar.is_cool = true and foo.is_fooey = true;'
          subject.load_str(policy)
          results = subject.get_allowed_resources('gwen', 'get', Foo)
          expected = Foos.select {|foo| foo.bar.is_cool and foo.is_fooey }
          expect(expected).not_to be_empty
          expect(unord_eq results, expected).to be true
        end
      end
    end

  end

end
