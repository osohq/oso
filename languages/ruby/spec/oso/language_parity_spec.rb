# frozen_string_literal: true

require 'oso'

oso = Oso.new

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

oso.load_file File.expand_path(File.join(__dir__, '../../../../test/test.polar'))

raise unless oso.allowed?(actor: 'a', action: 'b', resource: 'c')

# Test that a built in string method can be called.
oso.load_str <<~POLAR
  ?= x = "hello world!" and x.end_with?("world!");
POLAR

# Test that a custom error type is thrown.
exception_thrown = false
begin
  oso.load_str 'missingSemicolon()'
rescue Oso::Polar::ParseError::UnrecognizedEOF => e
  exception_thrown = true
  raise unless e.message == 'hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 19'
end
raise unless exception_thrown

oso.query_rule('specializers', D.new('hello'), B::C.new('hello')).next
oso.query_rule('floatLists').next
oso.query_rule('intDicts').next
oso.query_rule('comparisons').next
oso.query_rule('testForall').next
oso.query_rule('testRest').next
oso.query_rule('testMatches', A.new('hello')).next
oso.query_rule('testMethodCalls', A.new('hello'), B::C.new('hello')).next
oso.query_rule('testOr').next
oso.query_rule('testUnifyClass', A).next

# Test that cut doesn't return anything.
raise unless oso.query_rule('testCut').to_a.empty?

# Test that a constant can be called.
oso.register_constant Math, name: 'MyMath'
oso.load_str '?= MyMath.acos(1.0) = 0.0;'

# Test built-in type specializers.
# rubocop:disable Layout/EmptyLineAfterGuardClause
oso.query('builtinSpecializers(true, "Boolean")').next
raise unless oso.query('builtinSpecializers(false, "Boolean")').to_a.empty?
oso.query('builtinSpecializers(2, "Integer")').next
oso.query('builtinSpecializers(1, "Integer")').next
raise unless oso.query('builtinSpecializers(0, "Integer")').to_a.empty?
raise unless oso.query('builtinSpecializers(-1, "Integer")').to_a.empty?
oso.query('builtinSpecializers(1.0, "Float")').next
raise unless oso.query('builtinSpecializers(0.0, "Float")').to_a.empty?
raise unless oso.query('builtinSpecializers(-1.0, "Float")').to_a.empty?
oso.query('builtinSpecializers(["foo", "bar", "baz"], "List")').next
raise unless oso.query('builtinSpecializers(["bar", "foo", "baz"], "List")').to_a.empty?
oso.query('builtinSpecializers({foo: "foo"}, "Dictionary")').next
raise unless oso.query('builtinSpecializers({foo: "bar"}, "Dictionary")').to_a.empty?
oso.query('builtinSpecializers("foo", "String")').next
raise unless oso.query('builtinSpecializers("bar", "String")').to_a.empty?

oso.query('builtinSpecializers(1, "IntegerWithFields")').next
raise unless oso.query('builtinSpecializers(2, "IntegerWithGarbageFields")').to_a.empty?
raise unless oso.query('builtinSpecializers({}, "DictionaryWithFields")').to_a.empty?
raise unless oso.query('builtinSpecializers({z: 1}, "DictionaryWithFields")').to_a.empty?
oso.query('builtinSpecializers({y: 1}, "DictionaryWithFields")').next

# test iterables work
oso.query_rule('testIterables').next

# Test deref behaviour
oso.load_str '?= x = 1 and E.sum([x, 2, x]) = 4 and [3, 2, x].index(1) = 2;'

# Test unspecialized rule ordering
result = oso.query_rule('testUnspecializedRuleOrder', 'foo', 'bar', Oso::Polar::Variable.new('z'))
raise unless result.next['z'] == 1
raise unless result.next['z'] == 2
raise unless result.next['z'] == 3
# rubocop:enable Layout/EmptyLineAfterGuardClause
