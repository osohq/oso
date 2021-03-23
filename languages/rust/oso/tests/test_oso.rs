use oso::PolarClass;
use std::path::{Path, PathBuf};

mod common;

use common::OsoTest;

fn test_file_path() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"));
    path.join(Path::new("tests/test_oso.polar"))
}

#[derive(PolarClass, Debug, Clone, PartialEq)]
struct Actor {
    #[polar(attribute)]
    name: String,
}

impl Actor {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn companies(&self) -> Vec<Company> {
        vec![Company { id: 1 }]
    }
}

#[derive(PolarClass, Debug, Clone, PartialEq)]
struct Widget {
    #[polar(attribute)]
    id: i64,
}

impl Widget {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

#[derive(PolarClass, Debug, Clone, PartialEq)]
struct Company {
    #[polar(attribute)]
    id: i64,
}

impl Company {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn role(&self, actor: Actor) -> String {
        if actor.name == "president" {
            return "admin".to_string();
        } else {
            return "guest".to_string();
        }
    }
}

fn test_oso() -> OsoTest {
    let mut test = OsoTest::new();
    test.oso
        .register_class(
            Actor::get_polar_class_builder()
                .set_constructor(Actor::new)
                .add_method("companies", Actor::companies)
                .build(),
        )
        .unwrap();
    test.oso.register_class(Widget::get_polar_class()).unwrap();
    test.oso
        .register_class(
            Company::get_polar_class_builder()
                .set_constructor(Company::new)
                .add_method("role", Company::role)
                .with_equality_check()
                .build(),
        )
        .unwrap();

    let path = test_file_path();
    test.oso.load_file(path).unwrap();
    test
}

#[test]
fn test_is_allowed() -> oso::Result<()> {
    let oso = test_oso();

    let actor = Actor::new(String::from("guest"));
    let resource = Widget::new(1);
    let action = "get";
    assert!(oso.oso.is_allowed(actor, action, resource)?);

    let actor = Actor::new(String::from("president"));
    let resource = Company::new(1);
    let action = "create";
    assert!(oso.oso.is_allowed(actor, action, resource)?);

    Ok(())
}

#[test]
fn test_query_rule() -> oso::Result<()> {
    let oso = test_oso();

    let actor = Actor::new(String::from("guest"));
    let resource = Widget::new(1);
    let action = "get";
    let mut query = oso.oso.query_rule("allow", (actor, action, resource))?;
    assert!(query.next().is_some());

    Ok(())
}

#[test]
fn test_fail() -> oso::Result<()> {
    let oso = test_oso();

    let actor = Actor::new(String::from("guest"));
    let resource = Widget::new(1);
    let action = "not_allowed";
    assert!(!oso.oso.is_allowed(actor, action, resource)?);

    Ok(())
}

#[test]
fn test_instance_from_external_call() -> oso::Result<()> {
    let oso = test_oso();

    let actor = Actor::new(String::from("guest"));
    let resource = Company::new(1);
    assert!(oso.oso.is_allowed(actor, "frob", resource)?);

    Ok(())
}

#[test]
fn test_allow_model() -> oso::Result<()> {
    let oso = test_oso();

    let actor = Actor::new(String::from("auditor"));
    let resource = Company::new(1);
    assert!(oso.oso.is_allowed(actor, "list", resource)?);

    let actor = Actor::new(String::from("auditor"));
    let resource = Widget::new(1);
    assert!(!oso.oso.is_allowed(actor, "list", resource)?);

    Ok(())
}
