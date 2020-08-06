# frozen_string_literal: true

require 'oso'

oso = Oso.new

# Application class with default kwargs constructor.
class A
  attr_reader :x

  def initialize(x:) # rubocop:disable Naming/MethodParameterName
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
oso.register_class(B::C, name: 'C') { |y:| B::C.new(y) }

oso.load_file File.expand_path(File.join(__dir__, '../../../../test/test.polar'))
oso.load_queued_files

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

oso.query_rule('specializers', D.new(x: 'hello'), B::C.new('hello')).next
oso.query_rule('floatLists').next
oso.query_rule('intDicts').next
oso.query_rule('comparisons').next
oso.query_rule('testForall').next
oso.query_rule('testRest').next
oso.query_rule('testMatches', A.new(x: 'hello')).next
oso.query_rule('testMethodCalls', A.new(x: 'hello'), B::C.new('hello')).next
oso.query_rule('testOr').next
oso.query_rule('testHttpAndPathMapper').next

# Test that cut doesn't return anything.
exception_thrown = false
begin
  oso.query_rule('testCut').next
rescue StopIteration
  exception_thrown = true
end
raise unless exception_thrown

# Test that a constant can be called.
oso.register_class String
oso.load_str '?= x = "" and x.length == String.send("new").length;'
