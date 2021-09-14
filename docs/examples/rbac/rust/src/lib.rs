use std::collections::HashSet;

use oso::{Oso, OsoError, PolarClass};

#[derive(Clone, Eq, Hash, PartialEq, PolarClass)]
struct Organization {
    name: String,
}

#[derive(Clone, Eq, Hash, PartialEq, PolarClass)]
struct Repository {
    name: String,
    #[polar(attribute)]
    organization: Organization,
}

#[derive(Clone, Eq, Hash, PartialEq)]
enum Resource {
    Organization(Organization),
    Repository(Repository),
}

#[derive(Clone, Eq, Hash, PartialEq, PolarClass)]
struct Role {
    #[polar(attribute)]
    name: String,
    #[polar(attribute)]
    resource: Resource,
}

#[derive(Clone, PolarClass)]
struct User {
    name: String,
    #[polar(attribute)]
    roles: HashSet<Role>,
}

impl User {
    fn assign_role_for_resource(&mut self, name: String, resource: Resource) {
        self.roles.insert(Role { name, resource });
    }
}

fn setup_oso() -> Result<Oso, OsoError> {
    let mut oso = Oso::new();

    oso.register_class(Organization::get_polar_class())?;
    oso.register_class(Repository::get_polar_class())?;
    oso.register_class(User::get_polar_class())?;

    oso.load_files(vec!["main.polar"])?;

    Ok(oso)
}

mod test {
    use super::*;

    #[test]
    fn test_policy() {
        let alpha_association = Organization {
            name: "Alpha Association".to_owned(),
        };
        let beta_business = Organization {
            name: "Beta Business".to_owned(),
        };

        let affine_types = Repository {
            name: "Affine Types".to_owned(),
            organization: alpha_association.clone(),
        };
        let allocator = Repository {
            name: "Allocator".to_owned(),
            organization: alpha_association.clone(),
        };
        let bubble_sort = Repository {
            name: "Bubble Sort".to_owned(),
            organization: beta_business.clone(),
        };
        let benchmarks = Repository {
            name: "Benchmarks".to_owned(),
            organization: beta_business,
        };

        let mut ariana = User {
            name: "Ariana".to_owned(),
            roles: HashSet::new(),
        };
        let mut bhavik = User {
            name: "Bhavik".to_owned(),
            roles: HashSet::new(),
        };

        ariana.assign_role_for_resource(
            "owner".to_owned(),
            Resource::Organization(alpha_association),
        );
        bhavik.assign_role_for_resource(
            "contributor".to_owned(),
            Resource::Repository(bubble_sort.clone()),
        );
        bhavik.assign_role_for_resource(
            "maintainer".to_owned(),
            Resource::Repository(benchmarks.clone()),
        );

        let oso = setup_oso().unwrap();

        assert!(oso
            .is_allowed(ariana.clone(), "read", affine_types.clone())
            .unwrap());
        assert!(oso
            .is_allowed(ariana.clone(), "push", affine_types.clone())
            .unwrap());
        assert!(oso
            .is_allowed(ariana.clone(), "read", allocator.clone())
            .unwrap());
        assert!(oso
            .is_allowed(ariana.clone(), "push", allocator.clone())
            .unwrap());
        assert!(!oso
            .is_allowed(ariana.clone(), "read", bubble_sort.clone())
            .unwrap());
        assert!(!oso
            .is_allowed(ariana.clone(), "push", bubble_sort.clone())
            .unwrap());
        assert!(!oso
            .is_allowed(ariana.clone(), "read", benchmarks.clone())
            .unwrap());
        assert!(!oso.is_allowed(ariana, "push", benchmarks.clone()).unwrap());

        assert!(!oso
            .is_allowed(bhavik.clone(), "read", affine_types.clone())
            .unwrap());
        assert!(!oso
            .is_allowed(bhavik.clone(), "push", affine_types)
            .unwrap());
        assert!(!oso
            .is_allowed(bhavik.clone(), "read", allocator.clone())
            .unwrap());
        assert!(!oso.is_allowed(bhavik.clone(), "push", allocator).unwrap());
        assert!(oso
            .is_allowed(bhavik.clone(), "read", bubble_sort.clone())
            .unwrap());
        assert!(!oso.is_allowed(bhavik.clone(), "push", bubble_sort).unwrap());
        assert!(oso
            .is_allowed(bhavik.clone(), "read", benchmarks.clone())
            .unwrap());
        assert!(oso.is_allowed(bhavik, "push", benchmarks).unwrap());
    }
}
