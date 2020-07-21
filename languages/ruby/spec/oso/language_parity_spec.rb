require 'oso'

oso = Oso.new

# Application class with default kwargs constructor.
class A
  attr_reader :x

  def initialize(x:)
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

    def initialize(y)
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

raise unless oso.allow(actor: 'a', action: 'b', resource: 'c')

oso.load_str <<~POLAR
  ?= x = "hello world!" and x.end_with?("world!");
POLAR

exception_thrown = false
begin
  oso.load_str 'missingSemicolon()'
rescue Oso::Polar::ParseError::UnrecognizedEOF => e
  exception_thrown = true
  raise unless e.message == 'hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 19'
end
raise unless exception_thrown

oso.query_predicate('specializers', D.new(x: 'hello'), B::C.new('hello')).next
oso.query_predicate('floatLists').next
oso.query_predicate('intDicts').next
oso.query_predicate('comparisons').next
oso.query_predicate('testForall').next
oso.query_predicate('testRest').next
oso.query_predicate('testMatches', A.new(x: 'hello')).next
oso.query_predicate('testMethodCalls', A.new(x: 'hello'), B::C.new('hello')).next
oso.query_predicate('testOr').next

exception_thrown = false
begin
  oso.query_predicate('testCut').next
rescue StopIteration
  exception_thrown = true
end
raise unless exception_thrown

oso.register_class String
oso.load_str '?= x = "" and x.length == String.send("new").length;'

oso.query_predicate('testHttpAndPathMapper').next
