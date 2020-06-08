require 'ffi'
require_relative 'polar_lib'

p = PolarLib.polar_new()
result = PolarLib.polar_load_str(p, "f(1);")
puts result