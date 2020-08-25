class Expense {
  constructor(amount, description, submittedBy) {
    this.amount = amount;
    this.description = description;
    this.submittedBy = submittedBy;
  }
}

const EXPENSES = {
  1: new Expense(500, 'coffee', 'alice@example.com'),
  2: new Expense(5000, 'software', 'alice@example.com'),
  3: new Expense(50000, 'flight', 'bhavik@example.com'),
};

module.exports = { Expense, EXPENSES };
