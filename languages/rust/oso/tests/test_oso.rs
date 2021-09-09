use oso::{Action, Oso, PolarClass};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

mod common;

fn test_file_path() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"));
    path.join(Path::new("tests/test_oso.polar"))
}

#[derive(PolarClass, Debug, Clone, PartialEq)]
struct User {
    #[polar(attribute)]
    name: String,
}

impl User {
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

    pub fn role(&self, actor: User) -> String {
        if actor.name == "president" {
            "admin".to_string()
        } else {
            "guest".to_string()
        }
    }
}

fn test_oso() -> Oso {
    let mut oso = Oso::new();
    oso.register_class(
        User::get_polar_class_builder()
            .set_constructor(User::new)
            .add_method("companies", User::companies)
            .build(),
    )
    .unwrap();
    oso.register_class(Widget::get_polar_class()).unwrap();
    oso.register_class(
        Company::get_polar_class_builder()
            .set_constructor(Company::new)
            .add_method("role", Company::role)
            .with_equality_check()
            .build(),
    )
    .unwrap();

    let path = test_file_path();
    oso.load_files(vec![path]).unwrap();

    oso
}

#[test]
fn test_is_allowed() -> oso::Result<()> {
    common::setup();
    let oso = test_oso();

    let actor = User::new(String::from("guest"));
    let resource = Widget::new(1);
    let action = "get";

    assert!(oso.is_allowed(actor, action, resource)?);

    let actor = User::new(String::from("president"));
    let resource = Company::new(1);
    let action = "create";

    assert!(oso.is_allowed(actor, action, resource)?);

    Ok(())
}

#[test]
fn test_query_rule() -> oso::Result<()> {
    common::setup();
    let oso = test_oso();

    let actor = User::new(String::from("guest"));
    let resource = Widget::new(1);
    let action = "get";
    let mut query = oso.query_rule("allow", (actor, action, resource))?;

    assert!(query.next().is_some());

    Ok(())
}

#[test]
fn test_fail() -> oso::Result<()> {
    common::setup();
    let oso = test_oso();

    let actor = User::new(String::from("guest"));
    let resource = Widget::new(1);
    let action = "not_allowed";

    assert!(!oso.is_allowed(actor, action, resource)?);

    Ok(())
}

#[test]
fn test_instance_from_external_call() -> oso::Result<()> {
    common::setup();
    let oso = test_oso();

    let guest = User::new("guest".to_string());
    let resource = Company::new(1);
    assert!(oso.is_allowed(guest, "frob", resource.clone())?);

    // if the guest user can do it, then the dict should
    // create an instance of the user and be allowed
    let mut user_dict = HashMap::new();
    user_dict.insert("username", "guest".to_string());
    assert!(oso.is_allowed(user_dict, "frob", resource)?);

    Ok(())
}

#[test]
#[ignore = "PartialEq is not yet implemented for `oso::host::Class`"]
fn test_allow_model() -> oso::Result<()> {
    common::setup();
    let oso = test_oso();

    let actor = User::new(String::from("auditor"));
    assert!(oso.is_allowed(actor, "list", Company::get_polar_class())?);

    let actor = User::new(String::from("auditor"));
    assert!(!oso.is_allowed(actor, "list", Widget::get_polar_class())?);

    Ok(())
}

#[test]
fn test_get_allowed_actions() -> oso::Result<()> {
    common::setup();
    let mut oso = Oso::new();

    oso.register_class(User::get_polar_class()).unwrap();
    oso.register_class(Widget::get_polar_class()).unwrap();

    oso.load_str(
        r#"allow(_actor: User{name: "sally"}, action, _resource: Widget{id: 1}) if
           action in ["CREATE", "READ"];"#,
    )?;

    let actor = User::new(String::from("sally"));
    let resource = Widget::new(1);
    let actions: HashSet<Action> = oso.get_allowed_actions(actor, resource)?;

    assert!(actions.len() == 2);
    assert!(actions.contains(&Action::Typed("CREATE".to_string())));
    assert!(actions.contains(&Action::Typed("READ".to_string())));

    let actor = User::new(String::from("sally"));
    let resource = Widget::new(1);
    let actions: HashSet<String> = oso.get_allowed_actions(actor, resource)?;

    assert!(actions.len() == 2);
    assert!(actions.contains("CREATE"));
    assert!(actions.contains("READ"));

    oso.clear_rules().unwrap();

    oso.load_str(
        r#"allow(_actor: User{name: "fred"}, action, _resource: Widget{id: 2}) if
           action in [1, 2, 3, 4];"#,
    )?;

    let actor = User::new(String::from("fred"));
    let resource = Widget::new(2);
    let actions: HashSet<i32> = oso.get_allowed_actions(actor, resource)?;

    assert!(actions.len() == 4);
    assert!(actions.contains(&1));
    assert!(actions.contains(&2));
    assert!(actions.contains(&3));
    assert!(actions.contains(&4));

    let actor = User::new(String::from("fred"));
    let resource = Widget::new(2);
    let actions: HashSet<Action<i32>> = oso.get_allowed_actions(actor, resource)?;

    assert!(actions.len() == 4);
    assert!(actions.contains(&Action::Typed(1)));
    assert!(actions.contains(&Action::Typed(2)));
    assert!(actions.contains(&Action::Typed(3)));
    assert!(actions.contains(&Action::Typed(4)));

    Ok(())
}
