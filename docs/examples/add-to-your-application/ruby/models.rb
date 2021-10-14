class Repository
  attr_reader :name

  def initialize(name)
    @name  = name
  end

  def self.get_by_name(name)
    REPOS_DB[name]
  end
end

REPOS_DB = {
  "gmail" => Repository.new("gmail")
}

# docs: start
class Role
  attr_reader :name, :repository

  def initialize(name, repository)
    @name  = name
    @repository = repository
  end
end

class User
  attr_reader :roles

  def initialize(roles)
    @roles = roles
  end
end

USERS_DB = {
  "larry" => User.new([Role.new("admin", REPOS_DB["gmail"])])
}
# docs: end
