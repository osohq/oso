require 'oso'

oso = Oso.new

class A
  attr_reader :x

  def initialize(x:)
    @x = x
  end
end

module B
  class C
    attr_reader :y

    def initialize(y)
      @y = y
    end
  end
end

oso.register_class A
oso.register_class(B::C, name: "C") { |y:| B::C.new(y) }
oso.load_str <<~POLAR
  c(instance, y) if instance = new C{y: y};
  ?= c(instance, "hello") and instance.y = "hello";
POLAR
oso.load_file __dir__ + '/test.polar'
oso.load_queued_files
puts 'Tests Pass'
