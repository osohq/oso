require "oso"

OSO ||= Oso.new

class Env
  def self.var(variable)
    ENV[variable]
  end
end

OSO.register_class(Env)
