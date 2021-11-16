use oso::PolarClass;
mod common;
use common::OsoTest;

#[derive(Clone, Copy)]
struct Folder {
    pub name: &'static str,
}

impl oso::PolarClass for Folder {
    fn get_polar_class() -> oso::Class {
        Self::get_polar_class_builder()
            .set_equality_check(|f1, f2| f1.name == f2.name)
            .build()
    }
}

#[derive(Clone, Copy, PolarClass)]
struct Document {
    #[polar(attribute)]
    folder: Folder,
}

#[test]
fn test_is_subclass() {
    common::setup();
    let policy = r#"
        actor String {}

        resource Folder {
            permissions = [ "edit" ];
            roles = [ "admin" ];
            "edit" if "admin";
        }

        resource Document {
            permissions = [ "edit" ];
            roles = [ "admin" ];
            "edit" if "admin";
            relations = { parent: Folder };
            "admin" if "admin" on "parent";
        }

        allow(actor, action: String, resource: Resource) if
            has_permission(actor, action, resource);
        has_relation(folder: Folder, "parent", doc: Document)
            if folder == doc.folder;
        has_role("folder_admin", "admin", _resource: Folder);
    "#;
    let mut test = OsoTest::new();
    test.oso
        .register_class(Document::get_polar_class())
        .unwrap();
    test.oso.register_class(Folder::get_polar_class()).unwrap();
    test.load_str(policy);
    assert!(test
        .oso
        .is_allowed(
            "folder_admin",
            "edit",
            Document {
                folder: Folder { name: "a_folder" },
            },
        )
        .unwrap());
}
