class Expense
  attr_reader :amount, :description, :submitted_by

  def initialize(amount:, description:, submitted_by:)
    @amount = amount
    @description = description
    @submitted_by = submitted_by
  end
end

EXPENSES = {
  1 => Expense.new(amount: 500, description: "coffee", submitted_by:   "alice@example.com"),
  2 => Expense.new(amount: 5000, description: "software", submitted_by: "alice@example.com"),
  3 => Expense.new(amount: 50_000, description:"flight", submitted_by: "bhavik@example.com")
}
