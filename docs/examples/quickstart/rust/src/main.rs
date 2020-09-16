#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod expenses;
mod server;

use expenses::{Expense, EXPENSES};
use oso::{Oso, PolarClass};

fn no_policy() -> bool {
    let mut oso = Oso::new();
    oso.register_class(Expense::get_polar_class()).unwrap();

    let actor = "alice@example.com";
    let resource = EXPENSES[1].clone();
    let allowed = oso.is_allowed(actor, "GET", resource);
    println!("is_allowed => {}", allowed);
    allowed
}

fn with_policy() -> bool {
    let mut oso = Oso::new();
    oso.register_class(Expense::get_polar_class()).unwrap();

    let actor = "alice@example.com";
    let resource = EXPENSES[1].clone();
    oso.load_str(r#"allow("alice@example.com", "GET", _expense: Expense);"#)
        .unwrap();
    let allowed = oso.is_allowed(actor, "GET", resource.clone());
    println!("is_allowed => {}", allowed);
    allowed
}

fn main() {
    assert!(no_policy() == false);
    assert!(with_policy() == true);
    server::run();
}
