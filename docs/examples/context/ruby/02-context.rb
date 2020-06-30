require "oso"

OSO ||= Oso.new

class Env
  def var(variable)
    ENV[variable]
  end
end

OSO.register_class(Env)
