# frozen_string_literal: true

require 'oso'

oso = Oso.new

# Test that a custom error type is thrown.
exception_thrown = false
begin
  oso.load_str 'missingSemicolon()'
rescue Oso::Polar::ParseError::UnrecognizedEOF => e
  exception_thrown = true
  raise unless e.message == 'hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 19'
end
raise unless exception_thrown

# Test that a built in string method can be called.
oso.load_str <<~POLAR
  ?= x = "hello world!" and x.end_with?("world!");
POLAR

oso.clear_rules

# Test that a constant can be called.
oso.register_constant Math, name: 'MyMath'
oso.load_str '?= MyMath.acos(1.0) = 0.0;'

oso.clear_rules

# Application class with default constructor.
class A
  attr_reader :x

  def initialize(x) # rubocop:disable Naming/MethodParameterName
    @x = x
  end

  def foo
    -1
  end
end
oso.register_class A

class D < A; end

# Namespaced application class (to be aliased) with custom
# constructor.
module B
  class C
    attr_reader :y

    def initialize(y) # rubocop:disable Naming/MethodParameterName
      @y = y
    end

    def foo
      -1
    end
  end
end
oso.register_class(B::C, name: 'C')

class E
  def self.sum(args)
    args.sum
  end
end

oso.register_class(E)

# Test deref behaviour
oso.load_str '?= x = 1 and E.sum([x, 2, x]) = 4 and [3, 2, x].index(1) = 2;'

oso.clear_rules

oso.load_file File.expand_path(File.join(__dir__, '../../../../test/test.polar'))

raise unless oso.allowed?(actor: 'a', action: 'b', resource: 'c')

raise if oso.query_rule('specializers', D.new('hello'), B::C.new('hello')).first.nil?
raise if oso.query_rule('floatLists').first.nil?
raise if oso.query_rule('intDicts').first.nil?
raise if oso.query_rule('comparisons').first.nil?
raise if oso.query_rule('testForall').first.nil?
raise if oso.query_rule('testRest').first.nil?
raise if oso.query_rule('testMatches', A.new('hello')).first.nil?
raise if oso.query_rule('testMethodCalls', A.new('hello'), B::C.new('hello')).first.nil?
raise if oso.query_rule('testOr').first.nil?
raise if oso.query_rule('testUnifyClass', A).first.nil?

# Test that cut doesn't return anything.
raise unless oso.query_rule('testCut').first.nil?

# Test built-in type specializers.
raise if oso.query('builtinSpecializers(true, "Boolean")').first.nil?
raise unless oso.query('builtinSpecializers(false, "Boolean")').first.nil?
raise if oso.query('builtinSpecializers(2, "Integer")').first.nil?
raise if oso.query('builtinSpecializers(1, "Integer")').first.nil?
raise unless oso.query('builtinSpecializers(0, "Integer")').first.nil?
raise unless oso.query('builtinSpecializers(-1, "Integer")').first.nil?
raise if oso.query('builtinSpecializers(1.0, "Float")').first.nil?
raise unless oso.query('builtinSpecializers(0.0, "Float")').first.nil?
raise unless oso.query('builtinSpecializers(-1.0, "Float")').first.nil?
raise if oso.query('builtinSpecializers(["foo", "bar", "baz"], "List")').first.nil?
raise unless oso.query('builtinSpecializers(["bar", "foo", "baz"], "List")').first.nil?
raise if oso.query('builtinSpecializers({foo: "foo"}, "Dictionary")').first.nil?
raise unless oso.query('builtinSpecializers({foo: "bar"}, "Dictionary")').first.nil?
raise if oso.query('builtinSpecializers("foo", "String")').first.nil?
raise unless oso.query('builtinSpecializers("bar", "String")').first.nil?

raise if oso.query('builtinSpecializers(1, "IntegerWithFields")').first.nil?
raise unless oso.query('builtinSpecializers(2, "IntegerWithGarbageFields")').first.nil?
raise unless oso.query('builtinSpecializers({}, "DictionaryWithFields")').first.nil?
raise unless oso.query('builtinSpecializers({z: 1}, "DictionaryWithFields")').first.nil?
raise if oso.query('builtinSpecializers({y: 1}, "DictionaryWithFields")').first.nil?

# test iterables work
raise if oso.query_rule('testIterables').first.nil?

# Test unspecialized rule ordering
result = oso.query_rule('testUnspecializedRuleOrder', 'foo', 'bar', Oso::Polar::Variable.new('z'))
raise unless result.map { |res| res['z'] }.to_a == [1, 2, 3]
