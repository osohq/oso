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

      subject.load_str <<~POLAR
        test(A{});
        test(B{});

        try(v: B{}, res) := res = 2;
        try(v: C{}, res) := res = 3;
        try(v: A{}, res) := res = 1;
      POLAR

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

    context 'animal tests' do
      class Animal
        attr_reader :family, :genus, :species

        def initialize(family: nil, genus: nil, species: nil)
          @family = family
          @genus = genus
          @species = species
        end
      end
      let(:wolf) { 'Animal{species: "canis lupus", genus: "canis", family: "canidae"}' }
      let(:dog) {  'Animal{species: "canis familiaris", genus: "canis", family: "canidae"}' }
      let(:canine) { 'Animal{genus: "canis", family: "canidae"}' }
      let(:canid) {  'Animal{family: "canidae"}' }
      let(:animal) { 'Animal{}' }

      before(:example) { subject.register_class(Animal) }

      it 'can specialize on dict fields' do
        subject.load_str <<~POLAR
          what_is(animal: {genus: "canis"}, r) := r = "canine";
          what_is(animal: {species: "canis lupus", genus: "canis"}, r) := r = "wolf";
          what_is(animal: {species: "canis familiaris", genus: "canis"}, r) := r = "dog";
        POLAR
        expect(qvar(subject, "what_is(#{wolf}, r)", 'r')).to eq(%w[wolf canine])
        expect(qvar(subject, "what_is(#{dog}, r)", 'r')).to eq(%w[dog canine])
        expect(qvar(subject, "what_is(#{canine}, r)", 'r')).to eq(['canine'])
      end

      it 'can specialize on class fields' do
        subject.load_str <<~POLAR
          what_is(animal: Animal{}, r) := r = "animal";
          what_is(animal: Animal{genus: "canis"}, r) := r = "canine";
          what_is(animal: Animal{family: "canidae"}, r) := r = "canid";
          what_is(animal: Animal{species: "canis lupus", genus: "canis"}, r) := r = "wolf";
          what_is(animal: Animal{species: "canis familiaris", genus: "canis"}, r) := r = "dog";
          what_is(animal: Animal{species: s, genus: "canis"}, r) := r = s;
        POLAR
        expect(qvar(subject, "what_is(#{wolf}, r)", 'r')).to eq(['wolf', 'canis lupus', 'canine', 'canid', 'animal'])
        expect(qvar(subject, "what_is(#{dog}, r)", 'r')).to eq(['dog', 'canis familiaris', 'canine', 'canid', 'animal'])
        expect(qvar(subject, "what_is(#{canine}, r)", 'r')).to eq([None, 'canine', 'canid', 'animal'])
        expect(qvar(subject, "what_is(#{canid}, r)", 'r')).to eq(%w[canid animal])
        expect(qvar(subject, "what_is(#{animal}, r)", 'r')).to eq(['animal'])
      end

      it 'can specialize with a mix of class and dict fields' do
        subject.load_str <<~POLAR
          what_is(animal: Animal{}, r) := r = "animal_class";
          what_is(animal: Animal{genus: "canis"}, r) := r = "canine_class";
          what_is(animal: {genus: "canis"}, r) := r = "canine_dict";
          what_is(animal: Animal{family: "canidae"}, r) := r = "canid_class";
          what_is(animal: {species: "canis lupus", genus: "canis"}, r) := r = "wolf_dict";
          what_is(animal: {species: "canis familiaris", genus: "canis"}, r) := r = "dog_dict";
          what_is(animal: Animal{species: "canis lupus", genus: "canis"}, r) := r = "wolf_class";
          what_is(animal: Animal{species: "canis familiaris", genus: "canis"}, r) := r = "dog_class";
        POLAR

        wolf_dict = '{species: "canis lupus", genus: "canis", family: "canidae"}'
        dog_dict = '{species: "canis familiaris", genus: "canis", family: "canidae"}'
        canine_dict = '{genus: "canis", family: "canidae"}'

        # test rule ordering for instances
        expect(qvar(subject, "what_is(#{wolf}, r)", 'r')).to eq(%w[wolf_class canine_class canid_class animal_class
                                                                   wolf_dict canine_dict])
        expect(qvar(subject, "what_is(#{dog}, r)", 'r')).to eq(%w[dog_class canine_class canid_class animal_class
                                                                  dog_dict canine_dict])
        expect(qvar(subject, "what_is(#{canine}, r)", 'r')).to eq(%w[canine_class canid_class animal_class canine_dict])

        # test rule ordering for dicts
        expect(qvar(subject, "what_is(#{wolf_dict}, r)", 'r')).to eq(%w[wolf_dict canine_dict])
        expect(qvar(subject, "what_is(#{dog_dict}, r)", 'r')).to eq(%w[dog_dict canine_dict])
        expect(qvar(subject, "what_is(#{canine_dict}, r)", 'r')).to eq(['canine_dict'])
      end
    end
  end

  context 'when loading a file with inline queries' do
    it 'succeeds if all inline queries succeed' do
      subject.load_str('f(1); f(2); ?= f(1); ?= !f(3);')
    end

    it 'fails if an inline query fails' do
      pending "Don't handle inline queries yet"
      expect { subject.load_str('g(1); ?= g(2);') }.to raise_error Errors::PolarException
    end
  end

  context 'when parsing' do
    before(:example) { pending "Don't handle parser errors yet" }

    it 'raises on IntegerOverflow errors' do
      int = '18446744073709551616'
      rule = <<~POLAR
        f(a) := a = #{int};
      POLAR
      expect { subject.load_str(rule) }.to raise_error do |e|
        expect(e).to be_an Errors::IntegerOverflow
        expect(e.value).to eq("('#{int}', [1, 16])")
      end
    end

    context 'raises InvalidTokenCharacter' do
      it 'on unexpected newlines' do
        rule = <<~POLAR
          f(a) := a = "this is not
          allowed";
        POLAR
        expect { subject.load_str(rule) }.to raise_error do |e|
          expect(e).to be_an Errors::InvalidTokenCharacter
          expect(e.value).to eq("('this is not', '\\n', [1, 28])")
        end
      end

      it 'on null bytes' do
        rule = <<~POLAR
          f(a) := a = "this is not allowed\0
        POLAR
        expect { subject.load_str(rule) }.to raise_error do |e|
          expect(e).to be_an Errors::InvalidTokenCharacter
          expect(e.value).to eq("('this is not allowed', '\\x00', [1, 16])")
        end
      end
    end

    # Not sure what causes this.
    it 'raises on InvalidToken'

    it 'raises on UnrecognizedEOF errors' do
      rule = <<~POLAR
        f(a)
      POLAR
      expect { subject.load_str(rule) }.to raise_error do |e|
        expect(e).to be_an Errors::UnrecognizedEOF
        expect(e.value).to eq('[1, 8]')
      end
    end

    it 'raises on UnrecognizedToken errors' do
      rule = <<~POLAR
        1;
      POLAR
      expect { subject.load_str(rule) }.to raise_error do |e|
        expect(e).to be_an Errors::UnrecognizedToken
        expect(e.value).to eq("('1', [1, 4])")
      end
    end

    # Not sure what causes this.
    it 'raises on ExtraToken'
  end
end
