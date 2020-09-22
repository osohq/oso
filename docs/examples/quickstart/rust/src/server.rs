use std::sync::{Arc, Mutex};

use rocket::get;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request, State};

use oso::{Oso, PolarClass};

use crate::expenses::{Expense, EXPENSES};

#[derive(Debug)]
struct User(String);

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = String;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        if let Some(user) = request.headers().get_one("user") {
            request::Outcome::Success(User(user.to_string()))
        } else {
            request::Outcome::Failure((Status::Unauthorized, "Unknown User".to_owned()))
        }
    }
}

#[get("/expenses/<id>")]
fn hello(oso: State<OsoState>, user: User, id: usize) -> Result<Option<String>, Status> {
    if let Some(expense) = EXPENSES.get(id) {
        if oso.is_allowed(user.0, "GET", expense.clone()) {
            Ok(Some(format!("{}", expense)))
        } else {
            Err(Status::Unauthorized)
        }
    } else {
        Ok(None)
    }
}

struct OsoState {
    oso: Arc<Mutex<Oso>>,
}

impl OsoState {
    pub fn is_allowed(&self, actor: String, action: &str, resource: Expense) -> bool {
        let mut guard = self.oso.lock().unwrap();
        guard
            .is_allowed(actor, action.to_string(), resource)
            .unwrap()
    }
}

pub fn run() {
    let mut oso = Oso::new();
    oso.register_class(Expense::get_polar_class()).unwrap();
    oso.load_file("expenses.polar").unwrap();

    let oso_state = OsoState {
        oso: Arc::new(Mutex::new(oso)),
    };

    rocket::ignite()
        .mount("/", routes![hello])
        .manage(oso_state)
        .launch();
}
