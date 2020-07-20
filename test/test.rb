require 'oso'

oso = Oso.new

class A
  attr_reader :x

  def initialize(x:)
    @x = x
  end
end

oso.register_class A
oso.load_file __dir__ + '/test.polar'
oso.load_queued_files
puts 'Tests Pass'
