require 'ffi'
require_relative 'polar_lib'

# test FFI
p = PolarLib.polar_new()
result = PolarLib.polar_load_str(p, "f(1);")
puts result

class Polar
    def initialize()
        @polar = PolarLib.polar_new()
        @loaded_files = {}
        @load_queue = []
    end
end