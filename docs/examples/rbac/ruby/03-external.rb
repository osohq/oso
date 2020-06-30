class User:
  def initialize(name:)
    @name = name
  end

  def role
    db.query("SELECT role FROM user_roles WHERE username = ?", [@name])
  end
end
