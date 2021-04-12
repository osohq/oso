EXPENSES = [
  { submitted_by: 'alice', amount: 500, location: 'NYC', project_id: 2 }
].freeze

OSO ||= Oso.new

# expense-class-start
class Expense
  attr_accessor :amount, :submitted_by, :location, :project_id
  def initialize(amount:, submitted_by:, location:, project_id:)
    @amount = amount
    @submitted_by = submitted_by
    @location = location
    @project_id = project_id
  end

  def self.id(id)
    if id < EXPENSES.length
      Expense.new(**EXPENSES[id])
    else
      Expense.new
    end
  end
end

OSO.register_class(Expense)
# expense-class-end

MANAGERS = {
  'cora' => ['bhavik'],
  'bhavik' => ['alice']
}.freeze

# user-class-start
class User
  attr_accessor :name, :location
  def initialize(name, location = nil)
    @name = name
    @location = (location or 'NYC')
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
# user-class-end

class Project
  attr_accessor :id, :team_id
  def initialize(id:, team_id:)
    @id = id
    @team_id = team_id
  end

  def self.id(id)
    Project.new(id: id, team_id: 0)
  end
end

OSO.register_class(Project)

class Team
  attr_accessor :organization_id
  def initialize(organization_id:)
    @organization_id = organization_id
  end

  def self.id(_id)
    Team.new(organization_id: 0)
  end
end

OSO.register_class(Team)

class Organization
  attr_accessor :name
  def initialize(name:)
    @name = name
  end

  def self.id(_id)
    Organization.new(name: 'ACME')
  end
end

OSO.register_class(Organization)
