# frozen_string_literal: true

require_relative './helpers'
require 'osohq/polar/errors'

RSpec.configure do |c|
  c.include Helpers
end

RSpec.describe Osohq::Polar::Polar do
  let(:test_file) { File.join(__dir__, 'test_file.polar') }
  let(:test_file_gx) { File.join(__dir__, 'test_file_gx.polar') }

  it 'works' do
    subject.load_str('f(1);')
    results = subject.query_str('f(x)')
    expect(results.next).to eq({ 'x' => 1 })
    expect { results.next }.to raise_error StopIteration
  end

  it 'converts Polar values into Ruby values' do
    subject.load_str('f({x: [1, "two", true], y: {z: false}});')
    expect(qvar(subject, 'f(x)', 'x', one: true)).to eq({ 'x' => [1, 'two', true], 'y' => { 'z' => false } })
  end

  context '#load' do
    before(:example) { pending 'Polar#load is unimplemented' }

    it 'loads a Polar file' do
      subject.load(test_file)
      expect(qvar(subject, 'f(x)', 'x')).to eq([1, 2, 3])
    end

    it 'raises if given a non-Polar file' do
      expect { subject.load('other.ext') }.to raise_error Errors::BadFile
    end

    it 'is idempotent' do
      2.times { subject.load(test_file) }
      expect(qvar(subject, 'f(x)', 'x')).to eq([1, 2, 3])
    end

    it 'can load multiple files' do
      subject.load(test_file)
      subject.load(test_file_gx)
      expect(qvar(subject, 'f(x)', 'x')).to eq([1, 2, 3])
      expect(qvar(subject, 'g(x)', 'x')).to eq([1, 2, 3])
    end
  end

  context '#clear' do
    before(:example) { pending 'Polar#clear is unimplemented' }

    it 'clears the KB' do
      subject.load(test_file)
      subject.clear
      expect(query(subject, 'f(x)')).to be false
    end
  end

  context '#register_class' do
    before(:example) { pending 'Polar#register_class is unimplemented' }

    it 'registers a Ruby class with Polar' do
      class Bar
        def y
          'y'
        end
      end

      class Foo
        attr_reader :a

        def initialize(a)
          @a = a
        end

        def b
          Enumerator.new do |e|
            e.yield 'b'
          end
        end

        def c
          'c'
        end

        def d(x)
          x
        end

        def bar
          Bar.new
        end

        def e
          [1, 2, 3]
        end

        def f
          Enumerator.new do |e|
            e.yield [1, 2, 3]
            e.yield [4, 5, 6]
            e.yield 7
          end
        end

        def g
          { "hello": 'world' }
        end

        def h
          true
        end
      end

      def capital_foo
        Foo.new('A')
      end

      subject.register_class(Foo, from_polar: capital_foo)
      expect(qvar(subject, 'Foo{}.a = x', 'x', one: true)).to eq('A')
      expect(qvar(subject, 'Foo{}.a() = x', 'x', one: true)).to eq('A')
      expect(qvar(subject, 'Foo{}.b = x', 'x', one: true)).to eq('b')
      expect(qvar(subject, 'Foo{}.b() = x', 'x', one: true)).to eq('b')
      expect(qvar(subject, 'Foo{}.c = x', 'x', one: true)).to eq('c')
      expect(qvar(subject, 'Foo{}.c() = x', 'x', one: true)).to eq('c')
      expect(qvar(subject, 'Foo{} = f, f.a() = x', 'x', one: true)).to eq('A')
      expect(qvar(subject, 'Foo{}.bar().y() = x', 'x', one: true)).to eq('y')
      expect(qvar(subject, 'Foo{}.e = x', 'x')).to eq([[1, 2, 3]])
      expect(qvar(subject, 'Foo{}.f = x', 'x')).to eq([[1, 2, 3], [4, 5, 6], 7])
      expect(qvar(subject, 'Foo{}.g.hello = x', 'x', one: true)).to eq('world')
      expect(qvar(subject, 'Foo{}.h = x', 'x', one: true)).to be true
    end

    it 'respects the Ruby inheritance hierarchy for class specialization' do
      class A
        def a
          'A'
        end

        def x
          'A'
        end
      end

      class B < A
        def b
          'B'
        end

        def x
          'B'
        end
      end

      class C < B
        def c
          'C'
        end

        def x
          'C'
        end
      end

      class X
        def x
          'X'
        end
      end

      subject.register_class(A)
      subject.register_class(B)
      subject.register_class(C)
      subject.register_class(X)

      rules = <<~POLAR
        test(A{});
        test(B{});

        try(v: B{}, res) := res = 2;
        try(v: C{}, res) := res = 3;
        try(v: A{}, res) := res = 1;
      POLAR
      subject.load_str(rules)

      expect(qvar(subject, 'A{}.a = x', 'x', one: true)).to eq('A')
      expect(qvar(subject, 'A{}.x = x', 'x', one: true)).to eq('A')
      expect(qvar(subject, 'B{}.a = x', 'x', one: true)).to eq('A')
      expect(qvar(subject, 'B{}.b = x', 'x', one: true)).to eq('B')
      expect(qvar(subject, 'B{}.x = x', 'x', one: true)).to eq('B')
      expect(qvar(subject, 'C{}.a = x', 'x', one: true)).to eq('A')
      expect(qvar(subject, 'C{}.b = x', 'x', one: true)).to eq('B')
      expect(qvar(subject, 'C{}.c = x', 'x', one: true)).to eq('C')
      expect(qvar(subject, 'C{}.x = x', 'x', one: true)).to eq('C')
      expect(qvar(subject, 'X{}.x = x', 'x', one: true)).to eq('X')

      expect(query(subject, 'test(A{})').length).to be 1
      expect(query(subject, 'test(B{})').length).to be 2

      expect(qvar(subject, 'try(A{}, x)', 'x')).to eq([1])
      expect(qvar(subject, 'try(B{}, x)', 'x')).to eq([2, 1])
      expect(qvar(subject, 'try(C{}, x)', 'x')).to eq([3, 2, 1])
      expect(qvar(subject, 'try(X{}, x)', 'x')).to eq([])
    end
  end
end
