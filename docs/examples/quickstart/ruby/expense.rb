class Expense
  attr_reader :amount, :description, :submitted_by

  def initialize(amount, description, submitted_by)
    @amount = amount
    @description = description
    @submitted_by = submitted_by
  end
end

EXPENSES = {
  1 => Expense.new(500,   'coffee',   'alice@example.com'),
  2 => Expense.new(5000,  'software', 'alice@example.com'),
  3 => Expense.new(50_000, 'flight', 'bhavik@example.com')
}.freeze
