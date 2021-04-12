require 'oso'

OSO ||= Oso.new

# context-start
class Env
  def self.var(variable)
    ENV[variable]
  end
end

OSO.register_class(Env)
# context-end
