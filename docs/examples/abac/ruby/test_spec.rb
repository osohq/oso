require "oso"
require_relative '01-simple'

EXPENSES_DEFAULT = {
    "location": "NYC",
    "amount": 50,
    "project_id": 2,
}

RSpec.describe "example" do
  def load_file(example_name)
      file = File.join(File.dirname(__FILE__), '..', example_name)
      OSO.load_file(file)
      OSO
  end

  context "01-simple" do
    it "parses" do
      oso = load_file("01-simple.polar")
      oso.load_queued_files
    end

    it "works" do
      oso = load_file("01-simple.polar")

      expense = Expense.new(**EXPENSES_DEFAULT, submitted_by: "sam")
      expect(oso.allow(actor: User.new(name: "sam"), action: "view", resource: expense)).to be true

      expense = Expense.new(**EXPENSES_DEFAULT, submitted_by: "steve")
      expect(oso.allow(actor: User.new(name: "sam"), action: "view", resource: expense)).to be false
    end
  end

  context "02-rbac" do
    it "parses" do
      oso = load_file("02-rbac.polar")
      oso.load_queued_files
    end

    it "works" do
      oso = load_file("02-rbac.polar")
      oso.load_str('role(_: User { name: "sam" }, "admin", __: Project { id: 2 });')

      expense = Expense.new(location: "NYC", amount: 50, project_id: 0, submitted_by: "steve")
      expect(oso.allow(actor: User.new(name: "sam"), action: "view", resource: expense)).to be false

      expense = Expense.new(location: "NYC", amount: 50, project_id: 2, submitted_by: "steve")
      expect(oso.allow(actor: User.new(name: "sam"), action: "view", resource: expense)).to be true
    end
  end

  context "03-hierarchy" do
    it "parses" do
      oso = load_file("03-hierarchy.polar")
      oso.load_queued_files
    end

    it "works" do
      oso = load_file("03-hierarchy.polar")

      expect(oso.allow(actor: User.new(name: "bhavik"),
                       action: "view",
                       resource: Expense.new(**EXPENSES_DEFAULT, submitted_by: "alice"))).to be true
    end
  end

  context User do
    #u = User.new(name: "cora")
    #expect (u.employees().next().name).to be "bhavik"
  end
end
