class Env
  def var(variable)
    ENV[variable]
  end
end

def setup(oso)
  oso.register_class(Env)
end
