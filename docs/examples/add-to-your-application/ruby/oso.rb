require 'oso'

require_relative 'models'

OSO = Oso.new

OSO.register_class(User)
OSO.register_class(Repository)

OSO.load_files(["main.polar"])
