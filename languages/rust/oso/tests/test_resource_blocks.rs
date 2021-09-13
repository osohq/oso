use oso::{PolarClass, PolarValue, Query, ResultSet, ToPolar};
mod common;
use common::OsoTest;

#[derive(Clone, PolarClass, Eq)]
struct Org {
    #[polar(attribute)]
    pub name: String,
}

impl PartialEq for Org {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone, PolarClass, Eq)]
struct Repo {
    #[polar(attribute)]
    pub name: String,
    #[polar(attribute)]
    pub org: Org,
}

impl PartialEq for Repo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.org == other.org
    }
}

#[derive(Clone, PolarClass)]
struct Issue {
    #[polar(attribute)]
    pub name: String,
    #[polar(attribute)]
    pub repo: Repo,
}

#[derive(Clone, PolarClass)]
struct Role {
    #[polar(attribute)]
    pub name: String,
    #[polar(attribute)]
    pub resource: PolarValue,
}

#[derive(Clone, PolarClass)]
struct User {
    #[polar(attribute)]
    pub name: String,
    #[polar(attribute)]
    pub roles: Vec<Role>,
}

fn roles_test_oso() -> OsoTest {
    let mut test = OsoTest::new();
    test.oso
        .register_class(Org::get_polar_class_builder().with_equality_check().build())
        .unwrap();
    test.oso
        .register_class(
            Repo::get_polar_class_builder()
                .with_equality_check()
                .build(),
        )
        .unwrap();
    test.oso.register_class(Issue::get_polar_class()).unwrap();
    test.oso.register_class(User::get_polar_class()).unwrap();

    test
}

#[test]
fn test_resource_blocks() {
    common::setup();
    let mut test = roles_test_oso();
    let pol = r#"
      allow(actor, action, resource) if
        has_permission(actor, action, resource);

      has_role(user: User, name: String, resource: Resource) if
        role in user.roles and
        role.name = name and
        role.resource = resource;

      actor User {}

      resource Org {
        roles = [ "owner", "member" ];
        permissions = [ "invite", "create_repo" ];

        "create_repo" if "member";
        "invite" if "owner";

        "member" if "owner";
      }

      resource Repo {
        roles = [ "writer", "reader" ];
        permissions = [ "push", "pull" ];
        relations = { parent: Org };

        "pull" if "reader";
        "push" if "writer";

        "reader" if "writer";

        "reader" if "member" on "parent";
        "writer" if "owner" on "parent";
      }

      has_relation(org: Org, "parent", repo: Repo) if
        org = repo.org;

      resource Issue {
        permissions = [ "edit" ];
        relations = { parent: Repo };

        "edit" if "writer" on "parent";
      }

      has_relation(repo: Repo, "parent", issue: Issue) if
        repo = issue.repo;
    "#;

    test.load_str(pol);

    let osohq = Org {
        name: "oso".to_string(),
    };
    let apple = Org {
        name: "apple".to_string(),
    };
    let oso = Repo {
        name: "oso".to_string(),
        org: osohq.clone(),
    };
    let ios = Repo {
        name: "ios".to_string(),
        org: apple,
    };
    let bug = Issue {
        name: "bug".to_string(),
        repo: oso.clone(),
    };
    let laggy = Issue {
        name: "laggy".to_string(),
        repo: ios,
    };

    let osohq_owner = Role {
        name: "owner".to_string(),
        resource: osohq.clone().to_polar(),
    };
    let osohq_member = Role {
        name: "member".to_string(),
        resource: osohq.clone().to_polar(),
    };

    let gwen = User {
        name: "gwen".to_string(),
        roles: vec![osohq_member.clone()],
    };
    let dave = User {
        name: "dave".to_string(),
        roles: vec![osohq_owner.clone()],
    };

    fn empty(i: oso::Result<Query>) -> bool {
        i.unwrap()
            .collect::<oso::Result<Vec<ResultSet>>>()
            .unwrap()
            .is_empty()
    }

    assert!(!empty(
        test.oso
            .query_rule("allow", (dave.clone(), "invite", osohq.clone()))
    ));
    assert!(!empty(test.oso.query_rule(
        "allow",
        (dave.clone(), "create_repo", osohq.clone())
    )));
    assert!(!empty(
        test.oso
            .query_rule("allow", (dave.clone(), "push", oso.clone()))
    ));
    assert!(!empty(
        test.oso
            .query_rule("allow", (dave.clone(), "pull", oso.clone()))
    ));
    assert!(!empty(
        test.oso
            .query_rule("allow", (dave.clone(), "edit", bug.clone()))
    ));

    assert!(empty(
        test.oso
            .query_rule("allow", (gwen.clone(), "invite", osohq.clone()))
    ));
    assert!(!empty(
        test.oso
            .query_rule("allow", (gwen.clone(), "create_repo", osohq))
    ));
    assert!(empty(
        test.oso
            .query_rule("allow", (gwen.clone(), "push", oso.clone()))
    ));
    assert!(!empty(
        test.oso.query_rule("allow", (gwen.clone(), "pull", oso))
    ));
    assert!(empty(
        test.oso
            .query_rule("allow", (gwen.clone(), "edit", bug.clone()))
    ));

    assert!(empty(
        test.oso.query_rule("allow", (dave, "edit", laggy.clone()))
    ));
    assert!(empty(test.oso.query_rule("allow", (gwen, "edit", laggy))));

    let gabe = User {
        name: "gabe".to_string(),
        roles: vec![],
    };
    assert!(empty(
        test.oso.query_rule("allow", (gabe, "edit", bug.clone()))
    ));
    let gabe = User {
        name: "gabe".to_string(),
        roles: vec![osohq_member],
    };
    assert!(empty(
        test.oso.query_rule("allow", (gabe, "edit", bug.clone()))
    ));
    let gabe = User {
        name: "gabe".to_string(),
        roles: vec![osohq_owner],
    };
    assert!(!empty(test.oso.query_rule("allow", (gabe, "edit", bug))));
}
