use oso::PolarClass;
mod common;
use common::OsoTest;

#[derive(Clone, PolarClass)]
struct Document {
    #[polar(attribute)]
    project: Project,
}

#[derive(Clone, PolarClass)]
struct Project {
    #[polar(attribute)]
    public: bool,
}

#[test]
fn test_is_subclass() {
    common::setup();
    let policy = r#"
        resource Document {}
        resource Project {}

        allow(_user, "read", document: Document) if
            has_relation(document.project, "parent", document);

        has_relation(project: Project, "parent", _document: Document) if
            is_public(project);

        is_public(project: Project) if project.public;
        is_public(_document: Document) if false;
    "#;
    let mut test = OsoTest::new();
    test.oso
        .register_class(Document::get_polar_class())
        .unwrap();
    test.oso.register_class(Project::get_polar_class()).unwrap();
    test.load_str(policy);
    assert!(!test
        .oso
        .is_allowed(
            "anybody",
            "read",
            Document {
                project: Project { public: false },
            },
        )
        .unwrap());
    assert!(test
        .oso
        .is_allowed(
            "anybody",
            "read",
            Document {
                project: Project { public: true },
            },
        )
        .unwrap());
}
