use oso::{Oso, PolarClass};

use std::collections::HashMap;

fn types() -> anyhow::Result<()> {
    let mut oso = Oso::new();

    #[derive(Clone, PolarClass)]
    struct User1 {
        #[polar(attribute)]
        name: String,
        #[polar(attribute)]
        is_admin: bool,
    }
    oso.register_class(User1::get_polar_class())?;
    oso.load_str(r#"allow(actor: User1, _action, _resource) if actor.is_admin;"#)?;
    let user1 = User1 {
        name: "alice".to_string(),
        is_admin: true,
    };
    assert!(oso.is_allowed(user1, "foo", "bar")?);

    let mut oso = Oso::new();

    #[derive(Clone, PolarClass)]
    struct User2 {
        #[polar(attribute)]
        name: String,
        #[polar(attribute)]
        is_admin: bool,
    }

    impl User2 {
        fn new(name: String, is_admin: bool) -> Self {
            Self { name, is_admin }
        }

        fn is_called_alice(&self) -> bool {
            self.name == "alice"
        }
    }

    oso.register_class(
        User2::get_polar_class_builder()
            .set_constructor(User2::new)
            .add_method("is_called_alice", User2::is_called_alice)
            .build(),
    )?;
    oso.load_str(
        r#"
        allow(user: User2, _, _) if user.is_admin;
        ?= allow(new User2("bob", true), "foo", "bar");
        ?= new User2("alice", true).is_called_alice();
    "#,
    )?;

    let mut oso = Oso::new();

    #[derive(Clone, PolarClass)]
    struct User3 {
        #[polar(attribute)]
        name: String,
        #[polar(attribute)]
        is_admin: bool,
    }
    oso.register_class(User3::get_polar_class())?;
    oso.load_str(r#"allow(actor, _action, _resource) if actor matches User3{name: "alice"};"#)?;
    let user3 = User3 {
        name: "alice".to_string(),
        is_admin: true,
    };
    assert!(oso.is_allowed(user3, "foo", "bar")?);
    assert!(!oso.is_allowed("notauser", "foo", "bar")?);

    Ok(())
}

fn strings() -> anyhow::Result<()> {
    let mut oso = Oso::new();

    #[derive(PolarClass, Clone)]
    struct User {
        #[polar(attribute)]
        pub username: String,
    }

    oso.register_class(User::get_polar_class())?;

    oso.load_str(
        r#"allow(actor, _action, _resource) if actor.username.ends_with("example.com");"#,
    )?;

    let user = User {
        username: "alice@example.com".to_owned(),
    };
    assert!(oso.is_allowed(user, "foo", "bar")?);

    Ok(())
}

fn vecs() -> anyhow::Result<()> {
    let mut oso = Oso::new();

    #[derive(Clone, PolarClass)]
    struct User {
        #[polar(attribute)]
        pub groups: Vec<String>,
    }

    oso.register_class(User::get_polar_class()).unwrap();

    oso.load_str(r#"allow(actor, _action, _resource) if "HR" in actor.groups;"#)?;

    let user = User {
        groups: vec!["HR".to_string(), "payroll".to_string()],
    };
    assert!(oso.is_allowed(user, "foo", "bar")?);
    Ok(())
}

fn maps() -> anyhow::Result<()> {
    let mut oso = Oso::new();

    #[derive(Clone, PolarClass)]
    struct User {
        #[polar(attribute)]
        pub roles: HashMap<String, String>,
    }

    oso.register_class(User::get_polar_class())?;
    oso.load_str(r#"allow(actor, _action, _resource) if actor.roles.project1 = "admin";"#)?;

    let user = User {
        roles: maplit::hashmap! { "project1".to_string() => "admin".to_string() },
    };
    assert!(oso.is_allowed(user, "foo", "bar")?);

    Ok(())
}

fn enums() -> anyhow::Result<()> {
    let mut oso = Oso::new();

    #[derive(Clone)]
    enum UserType {
        Admin,
        Guest,
    }

    impl oso::PolarClass for UserType {}

    oso.register_class(
        oso::Class::builder::<UserType>()
            .add_method("is_admin", |u: &UserType| matches!(u, UserType::Admin))
            .build(),
    )?;
    oso.load_str(r#"allow(actor, _action, _resource) if actor.is_admin();"#)?;

    let user = UserType::Admin;
    assert!(oso.is_allowed(user, "foo", "bar")?);
    assert!(!oso.is_allowed(UserType::Guest, "foo", "bar")?);

    Ok(())
}

fn iters() -> anyhow::Result<()> {
    let mut oso = Oso::new();

    #[derive(Clone, PolarClass)]
    struct User {
        groups: Vec<String>,
    }

    oso.register_class(
        User::get_polar_class_builder()
            .add_iterator_method("get_group", |u: &User| u.groups.clone().into_iter())
            .build(),
    )
    .unwrap();

    oso.load_str(r#"allow(actor, _action, _resource) if "payroll" in actor.get_group();"#)?;

    let user = User {
        groups: vec!["HR".to_string(), "payroll".to_string()],
    };
    assert!(oso.is_allowed(user, "foo", "bar")?);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    strings()?;
    vecs()?;
    maps()?;
    enums()?;
    iters()?;
    types()?;
    println!("Examples passed");

    Ok(())
}
