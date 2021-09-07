require 'oso'
require_relative '01-simple'

EXPENSES_DEFAULT = {
  "location": 'NYC',
  "amount": 50,
  "project_id": 2
}.freeze

RSpec.describe 'example' do
  def file_contents(example_name)
    File.open(File.join(File.dirname(__FILE__), '..', example_name), &:read)
  end

  it 'works' do
    oso = OSO

    # 01-simple
    policy = file_contents('01-simple.polar')
    oso.load_str policy

    expense = Expense.new(**EXPENSES_DEFAULT, submitted_by: 'sam')
    expect(oso.allowed?(actor: User.new('sam'), action: 'view', resource: expense)).to be true

    expense = Expense.new(**EXPENSES_DEFAULT, submitted_by: 'steve')
    expect(oso.allowed?(actor: User.new('sam'), action: 'view', resource: expense)).to be false

    oso.clear_rules

    # 02-rbac
    policy += file_contents('02-rbac.polar')
    policy += 'role(_: User { name: "sam" }, "admin", _: Project { id: 2 });'
    oso.load_str policy

    expense = Expense.new(location: 'NYC', amount: 50, project_id: 0, submitted_by: 'steve')
    expect(oso.allowed?(actor: User.new('sam'), action: 'view', resource: expense)).to be false

    expense = Expense.new(location: 'NYC', amount: 50, project_id: 2, submitted_by: 'steve')
    expect(oso.allowed?(actor: User.new('sam'), action: 'view', resource: expense)).to be true

    oso.clear_rules

    # 03-hierarchy
    policy += file_contents('03-hierarchy.polar')
    oso.load_str policy

    expect(oso.allowed?(actor: User.new('bhavik'),
                        action: 'view',
                        resource: Expense.new(**EXPENSES_DEFAULT, submitted_by: 'alice'))).to be true
  end
end
