EXPENSES = [
  { :submitted_by => "alice", :amount => 500, :location => "NYC", :project_id => 2 }
]

OSO ||= Oso.new

# expense-class-start
class Expense
  # expense-class-end
  attr_accessor :amount, :submitted_by, :location, :project_id
  def initialize(amount:, submitted_by:, location:, project_id:)
    @amount = amount
    @submitted_by = submitted_by
    @location = location
    @project_id = project_id
  end

  def self.by_id(id:)
    if id < EXPENSES.length
      Expense.new(**EXPENSES[id])
    else
      Expense.new
    end
  end
end

OSO.register_class(Expense) do |**kwargs|
  Expense.by_id(**kwargs)
end

MANAGERS = {
  "cora" => ["bhavik"],
  "bhavik" => ["alice"]
}

# user-class-start
class User
  attr_accessor :name, :location
  def initialize(name:, location: nil)
    @name = name # user-class-end
    @location = (location or "NYC")
  end

  def employees
    if MANAGERS.include?(@name)
      users = MANAGERS[@name].map do |name|
        User.new(name: name)
      end
      users.to_enum
    else
      [].each
    end
  end
end

OSO.register_class(User)


class Project
  attr_accessor :id, :team_id
  def initialize(id:, team_id:)
    @id = id
    @team_id = team_id
  end

  def self.by_id(id:)
    Project.new(id: id, team_id: 0)
  end
end

OSO.register_class(Project) do |**kwargs|
  Project.by_id(**kwargs)
end

class Team
  attr_accessor :organization_id
  def initialize(organization_id:)
    @organization_id = organization_id
  end

  def self.by_id(id:)
    Team.new(organization_id: 0)
  end
end

OSO.register_class(Team) do |**kwargs|
  Team.by_id(**kwargs)
end

class Organization
  attr_accessor :name
  def initialize(name:)
    @name = name
  end

  def self.by_id(id:)
    Organization.new(name: "ACME")
  end
end

OSO.register_class(Organization) do |**kwargs|
  Organization.by_id(**kwargs)
end
