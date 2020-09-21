use oso::{Oso, PolarClass};
use oso_derive::*;

use std::collections::HashMap;

fn strings() -> anyhow::Result<()> {
    let mut oso = Oso::new();

    #[derive(PolarClass, Clone)]
    struct User {
        #[polar(attribute)]
        pub username: String,
    }

    oso.register_class(User::get_polar_class())?;

    oso.load_str(r#"allow(actor, action, resource) if actor.username.ends_with("example.com");"#)?;

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

    oso.load_str(r#"allow(actor, action, resource) if "HR" in actor.groups;"#)?;

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
    oso.load_str(r#"allow(actor, action, resource) if actor.roles.project1 = "admin";"#)?;

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

    impl oso::HostClass for UserType {}

    oso.register_class(
        oso::Class::<UserType>::new()
            .add_method("is_admin", |u: &UserType| matches!(u, UserType::Admin))
            .build(),
    )?;
    oso.load_str(r#"allow(actor, action, resource) if actor.is_admin();"#)?;

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

    oso.load_str(r#"allow(actor, action, resource) if actor.get_group() = "payroll";"#)?;

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
    println!("Examples passed");

    Ok(())
}
