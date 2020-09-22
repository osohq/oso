use lazy_static::lazy_static;
use std::fmt;
use std::string::ToString;

use oso::*;

lazy_static! {
    pub static ref EXPENSES: [Expense; 3] = [
        Expense::new(500, "coffee", "alice@example.com"),
        Expense::new(5000, "software", "alice@example.com"),
        Expense::new(50000, "flight", "bhavik@example.com"),
    ];
}

#[derive(PolarClass, Debug, Clone)]
pub struct Expense {
    pub amount: i32,
    pub description: String,
    #[polar(attribute)]
    pub submitted_by: String,
}

impl Expense {
    pub fn new(amount: i32, description: &str, submitted_by: &str) -> Self {
        Self {
            amount,
            description: description.to_string(),
            submitted_by: submitted_by.to_string(),
        }
    }
}

impl fmt::Display for Expense {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Expense(amount={}, description='{}', submitted_by='{}')",
            self.amount, &self.description, &self.submitted_by
        )
    }
}
