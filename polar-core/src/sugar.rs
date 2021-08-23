use std::collections::HashMap;

use lalrpop_util::ParseError as LalrpopError;

use super::error::{ParseError, PolarError, PolarResult, RuntimeError};
use super::kb::KnowledgeBase;
use super::lexer::Token;
use super::rules::*;
use super::terms::*;

// TODO(gj): round up longhand `has_permission/3` and `has_role/3` rules to incorporate their
// referenced permissions & roles (implied & implier side) into the exhaustiveness checks.

// TODO(gj): round up longhand `has_relation/3` rules to check that every declared `relation` has a
// corresponding `has_relation/3` implementation.

// TODO(gj): disallow same string to be declared as a perm/role and a relation.
// This'll come into play for "owner"-style actor relationships.

// This type is used as a pre-validation bridge between LALRPOP & Rust.
#[derive(Debug)]
pub enum Production {
    Roles(Term),                             // List<String>
    Permissions(Term),                       // List<String>
    Relations(Term),                         // Dict<Symbol, Symbol>
    Implication(Term, (Term, Option<Term>)), // (String, (String, Option<String>))
}

pub fn validate_relation_keyword(
    (keyword, relation): (Term, Term),
) -> Result<Term, LalrpopError<usize, Token, error::ParseError>> {
    if keyword.value().as_symbol().unwrap().0 == "on" {
        Ok(relation)
    } else {
        Err(LalrpopError::User {
            error: ParseError::ParseSugar {
                loc: keyword.offset(),
                msg: format!(
                    "Unexpected relation keyword '{}'. Did you mean 'on'?",
                    keyword
                ),
                ranges: vec![],
            },
        })
    }
}

pub fn validate_parsed_declaration(
    (name, term): (Symbol, Term),
) -> Result<Production, LalrpopError<usize, Token, error::ParseError>> {
    match (name.0.as_ref(), term.value()) {
        ("roles", Value::List(_)) => Ok(Production::Roles(term)),
        ("permissions", Value::List(_)) => Ok(Production::Permissions(term)),
        ("relations", Value::Dictionary(_)) => Ok(Production::Relations(term)),

        ("roles", Value::Dictionary(_)) | ("permissions", Value::Dictionary(_)) => {
            Err(LalrpopError::User {
                error: ParseError::ParseSugar {
                    loc: term.offset(),
                    msg: format!(
                        "Expected '{}' declaration to be a list of strings; found a dictionary:\n",
                        name
                    ),
                    ranges: vec![term.span().unwrap()],
                },
            })
        }
        ("relations", Value::List(_)) => Err(LalrpopError::User {
            error: ParseError::ParseSugar {
                loc: term.offset(),
                msg: "Expected 'relations' declaration to be a dictionary; found a list:\n".to_owned(),
                ranges: vec![term.span().unwrap()],
            },
        }),

        (_, Value::List(_)) => Err(LalrpopError::User {
            error: ParseError::ParseSugar {
                loc: term.offset(),
                msg: format!(
                    "Unexpected declaration '{}'. Did you mean for this to be 'roles = [ ... ];' or 'permissions = [ ... ];'?\n", name
                ),
                ranges: vec![term.span().unwrap()],
            },
        }),
        (_, Value::Dictionary(_)) => Err(LalrpopError::User {
            error: ParseError::ParseSugar {
                loc: term.offset(),
                msg: format!(
                    "Unexpected declaration '{}'. Did you mean for this to be 'relations = {{ ... }};'?\n", name
                ),
                ranges: vec![term.span().unwrap()],
            },
        }),
        _ => unreachable!(),
    }
}

pub fn turn_productions_into_namespace(
    resource: Term,
    productions: Vec<Production>,
) -> Result<Namespace, LalrpopError<usize, Token, error::ParseError>> {
    let mut roles: Option<Term> = None;
    let mut permissions: Option<Term> = None;
    let mut relations: Option<Term> = None;
    let mut implications = vec![];

    let make_error = |name: &str, previous: &Term, new: &Term| {
        let msg = format!(
            "Multiple '{}' declarations in {} namespace.\n",
            name,
            resource.to_polar()
        );
        ParseError::ParseSugar {
            loc: new.offset(),
            msg,
            // TODO(gj): Create a Parsed<Term> or something that _always_ has source info.
            ranges: vec![(previous.span().unwrap()), (new.span().unwrap())],
        }
    };

    for production in productions {
        match production {
            Production::Roles(new) => {
                if let Some(previous) = roles {
                    let error = make_error("roles", &previous, &new);
                    return Err(LalrpopError::User { error });
                }
                roles = Some(new);
            }
            Production::Permissions(new) => {
                if let Some(previous) = permissions {
                    let error = make_error("permissions", &previous, &new);
                    return Err(LalrpopError::User { error });
                }
                permissions = Some(new);
            }
            Production::Relations(new) => {
                if let Some(previous) = relations {
                    let error = make_error("relations", &previous, &new);
                    return Err(LalrpopError::User { error });
                }
                relations = Some(new);
            }
            Production::Implication(head, body) => {
                // TODO(gj): Warn the user on duplicate implication definitions.
                implications.push(Implication { head, body });
            }
        }
    }

    Ok(Namespace {
        resource,
        roles,
        permissions,
        relations,
        implications,
    })
}

#[derive(Clone, Debug)]
pub enum Declaration {
    Role,
    Permission,
    /// `Term` is a `Symbol` that is the (registered) type of the relation. E.g., `Org` in `parent: Org`.
    Relation(Term),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Implication {
    /// `Term` is a `String`. E.g., `"member"` in `"member" if "owner";`.
    pub head: Term,
    /// Both terms are strings. The former is the 'implier' and the latter is the 'relation', e.g.,
    /// `"owner"` and `"parent"`, respectively, in `"writer" if "owner" on "parent";`.
    pub body: (Term, Option<Term>),
}

impl Implication {
    pub fn into_rule(self, resource: &Term, namespaces: &Namespaces) -> PolarResult<Rule> {
        let Self { head, body } = self;
        // Copy SourceInfo from head of implication.
        // TODO(gj): assert these can only be None in tests.
        let src_id = head.get_source_id().unwrap_or(0);
        let (start, end) = head.span().unwrap_or((0, 0));

        let name = namespaces.local_declaration_to_rule_name(&head, resource)?;
        let params = implication_head_into_params(head, resource);
        let body = implication_body_into_rule_body(body, resource, namespaces)?;

        Ok(Rule::new_from_parser(
            src_id, start, end, name, params, body,
        ))
    }
}

type Declarations = HashMap<Term, Declaration>;

impl Declaration {
    fn as_relation(&self) -> PolarResult<&Term> {
        if let Declaration::Relation(relation) = self {
            Ok(relation)
        } else {
            Err(RuntimeError::TypeError {
                msg: format!("Expected Relation; got: {:?}", self),
                stack_trace: None, // @TODO
            }
            .into())
        }
    }

    fn as_predicate(&self) -> Symbol {
        match self {
            Declaration::Role => sym!("has_role"),
            Declaration::Permission => sym!("has_permission"),
            Declaration::Relation(_) => sym!("has_relation"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Namespace {
    pub resource: Term,
    pub roles: Option<Term>,
    pub permissions: Option<Term>,
    pub relations: Option<Term>,
    pub implications: Vec<Implication>,
}

#[derive(Clone, Default)]
pub struct Namespaces {
    /// Map from resource (`Symbol`) to the declarations for that resource.
    declarations: HashMap<Term, Declarations>,
}

impl Namespaces {
    pub fn new() -> Self {
        Self {
            declarations: HashMap::new(),
        }
    }

    fn add(&mut self, resource: Term, declarations: Declarations) -> Option<Declarations> {
        self.declarations.insert(resource, declarations)
    }

    fn exists(&self, resource: &Term) -> bool {
        self.declarations.contains_key(resource)
    }

    fn clear(&mut self) {
        self.declarations.clear();
    }

    fn get_declaration(&self, name: &Term, resource: &Term) -> PolarResult<&Declaration> {
        // TODO(gj): .get(resource) instead of [resource]
        if let Some(declaration) = self.declarations[resource].get(name) {
            Ok(declaration)
        } else {
            // TODO(gj): message isn't totally accurate when going across resources. E.g., with
            // policy:
            // Org{roles=["foo"];} Repo{permissions=["bar"]; relations={parent:Org}; "bar" if "baz"
            // on "parent";}
            Err(ParseError::ParseSugar {
                loc: name.offset(),
                msg: format!(
                    "Undeclared term {} referenced in implication in {} namespace. \
                        Did you mean to declare it as a role, permission, or relation?",
                    name.to_polar(),
                    resource
                ),
                ranges: vec![],
            }
            .into())
        }
    }

    fn get_related_type(&self, relation: &Term, resource: &Term) -> PolarResult<&Term> {
        self.get_declaration(relation, resource)?.as_relation()
    }

    fn local_declaration_to_rule_name(&self, name: &Term, resource: &Term) -> PolarResult<Symbol> {
        Ok(self.get_declaration(name, resource)?.as_predicate())
    }

    fn cross_resource_predicate_name(
        &self,
        name: &Term,
        relation: &Term,
        resource: &Term,
    ) -> PolarResult<Symbol> {
        let related_type = self.get_related_type(relation, resource)?;
        self.local_declaration_to_rule_name(name, related_type)
    }
}

impl KnowledgeBase {
    pub fn rewrite_implications(&mut self) -> PolarResult<()> {
        let mut errors = vec![];

        errors.append(&mut check_all_relation_types_have_been_registered(self));

        let mut rules = vec![];
        for (resource, implications) in self.rewrite_me_pls.drain() {
            for implication in implications {
                match implication.into_rule(&resource, &self.namespaces) {
                    Ok(rule) => rules.push(rule),
                    Err(error) => errors.push(error),
                }
            }
        }

        // Add the rewritten rules to the KB.
        for rule in rules {
            self.add_rule(rule);
        }

        // TODO(gj): Emit all errors instead of just the first.
        if !errors.is_empty() {
            self.namespaces.clear();
            return Err(errors[0].clone());
        }

        Ok(())
    }
}

fn check_all_relation_types_have_been_registered(kb: &KnowledgeBase) -> Vec<PolarError> {
    let mut errors = vec![];
    for declarations in kb.namespaces.declarations.values() {
        for (declaration, kind) in declarations {
            if let Declaration::Relation(related_type) = kind {
                errors.extend(relation_type_is_registered(kb, (declaration, related_type)).err());
            }
        }
    }
    errors
}

fn index_declarations(
    roles: Option<Term>,
    permissions: Option<Term>,
    relations: Option<Term>,
    resource: &Term,
) -> PolarResult<HashMap<Term, Declaration>> {
    let mut declarations = HashMap::new();

    if let Some(roles) = roles {
        for role in roles.value().as_list()? {
            if declarations
                .insert(role.clone(), Declaration::Role)
                .is_some()
            {
                return Err(ParseError::ParseSugar {
                    loc: role.offset(),
                    msg: format!(
                        "{}: Duplicate declaration of {} in the roles list.",
                        resource.to_polar(),
                        role.to_polar()
                    ),
                    ranges: vec![],
                }
                .into());
            }
        }
    }

    if let Some(permissions) = permissions {
        for permission in permissions.value().as_list()? {
            if let Some(previous) = declarations.insert(permission.clone(), Declaration::Permission)
            {
                let msg = if matches!(previous, Declaration::Permission) {
                    format!(
                        "{}: Duplicate declaration of {} in the permissions list.",
                        resource.to_polar(),
                        permission.to_polar()
                    )
                } else {
                    format!(
                        "{}: {} declared as a permission but it was previously declared as a role.",
                        resource.to_polar(),
                        permission.to_polar()
                    )
                };
                return Err(ParseError::ParseSugar {
                    loc: permission.offset(),
                    msg,
                    ranges: vec![],
                }
                .into());
            }
        }
    }

    if let Some(relations) = relations {
        for (relation, related_type) in &relations.value().as_dict()?.fields {
            // Stringify relation so that we can index into the declarations map with a string
            // reference to the relation. E.g., relation `creator: User` gets stored as `"creator"
            // => Relation(User)` so that when we encounter an implication `"admin" if "creator";`
            // we can easily look up what type of declaration `"creator"` is.
            let stringified_relation = related_type.clone_with_value(value!(relation.0.as_str()));
            let declaration = Declaration::Relation(related_type.clone());
            if let Some(previous) = declarations.insert(stringified_relation, declaration) {
                let msg = match previous {
                    Declaration::Role => format!(
                        "{}: '{}' declared as a relation but it was previously declared as a role.",
                        resource.to_polar(),
                        relation.to_polar()
                    ),
                    Declaration::Permission => format!(
                        "{}: '{}' declared as a relation but it was previously declared as a permission.",
                        resource.to_polar(),
                        relation.to_polar()
                    ),
                    _ => unreachable!("duplicate dict keys aren't parseable"),
                };
                return Err(ParseError::ParseSugar {
                    loc: related_type.offset(),
                    msg,
                    ranges: vec![],
                }
                .into());
            }
        }
    }
    Ok(declarations)
}

fn resource_as_var(resource: &Term) -> Value {
    let name = &resource.value().as_symbol().expect("sym").0;
    let mut lowercased = name.to_lowercase();

    // If the resource's name is already lowercase, append "_instance" to distinguish the variable
    // name from the resource's name.
    if &lowercased == name {
        lowercased += "_instance";
    }

    value!(sym!(lowercased))
}

fn implication_body_into_rule_body(
    body: (Term, Option<Term>),
    resource: &Term,
    namespaces: &Namespaces,
) -> PolarResult<Term> {
    let (implier, relation) = body;
    let resource_var = implier.clone_with_value(resource_as_var(resource));
    let actor_var = implier.clone_with_value(value!(sym!("actor")));
    if let Some(relation) = relation {
        // TODO(gj): what if the relation is with the same type? E.g.,
        // `Dir { relations = { parent: Dir }; }`. This might cause Polar to loop.
        let related_type = namespaces.get_related_type(&relation, resource)?;
        let related_type_var = relation.clone_with_value(resource_as_var(related_type));

        let relation_call = relation.clone_with_value(value!(Call {
            name: sym!("has_relation"),
            // For example: vec![org, "parent", repo]
            args: vec![related_type_var.clone(), relation.clone(), resource_var],
            kwargs: None
        }));

        let implier_call = implier.clone_with_value(value!(Call {
            name: namespaces.cross_resource_predicate_name(&implier, &relation, resource)?,
            // For example: vec![actor, "owner", org]
            args: vec![actor_var, implier.clone(), related_type_var],
            kwargs: None
        }));
        Ok(implier.clone_with_value(value!(op!(And, relation_call, implier_call))))
    } else {
        let implier_call = implier.clone_with_value(value!(Call {
            name: namespaces.local_declaration_to_rule_name(&implier, resource)?,
            args: vec![actor_var, implier.clone(), resource_var],
            kwargs: None
        }));
        Ok(implier.clone_with_value(value!(op!(And, implier_call))))
    }
}

fn implication_head_into_params(head: Term, resource: &Term) -> Vec<Parameter> {
    let resource_name = &resource.value().as_symbol().expect("sym").0;
    vec![
        Parameter {
            parameter: head.clone_with_value(value!(sym!("actor"))),
            specializer: None,
        },
        Parameter {
            parameter: head.clone(),
            specializer: None,
        },
        Parameter {
            parameter: head.clone_with_value(resource_as_var(resource)),
            specializer: Some(
                resource.clone_with_value(value!(pattern!(instance!(resource_name)))),
            ),
        },
    ]
}

fn check_for_duplicate_namespaces(namespaces: &Namespaces, resource: &Term) -> PolarResult<()> {
    if namespaces.exists(resource) {
        return Err(ParseError::ParseSugar {
            loc: resource.offset(),
            // TODO(gj): better error message, e.g.:
            //               duplicate namespace declaration: Org { ... } defined on line XX of file YY
            //                                                previously defined on line AA of file BB
            msg: format!("duplicate declaration of {} namespace", resource),
            ranges: vec![],
        }
        .into());
    }
    Ok(())
}

// TODO(gj): no way to know in the core if `resource` was registered as a class or a constant.
fn is_registered_class(kb: &KnowledgeBase, x: &Term) -> PolarResult<bool> {
    Ok(kb.is_constant(x.value().as_symbol()?))
}

fn check_that_namespace_resource_is_registered(
    kb: &KnowledgeBase,
    resource: &Term,
) -> PolarResult<()> {
    if !is_registered_class(kb, resource)? {
        // TODO(gj): better error message
        let msg = format!(
            "{} namespace must be registered as a class.",
            resource.to_polar()
        );
        let (loc, ranges) = (resource.offset(), vec![]);
        // TODO(gj): UnregisteredClassError in the core.
        return Err(ParseError::ParseSugar { loc, msg, ranges }.into());
    }
    Ok(())
}

fn relation_type_is_registered(
    kb: &KnowledgeBase,
    (relation, kind): (&Term, &Term),
) -> PolarResult<()> {
    if !is_registered_class(kb, kind)? {
        let msg = format!(
            "Type '{}' in relation '{}: {}' must be registered as a class.",
            kind.to_polar(),
            relation.value().as_string()?,
            kind.to_polar(),
        );
        let (loc, ranges) = (relation.offset(), vec![]);
        // TODO(gj): UnregisteredClassError in the core.
        return Err(ParseError::ParseSugar { loc, msg, ranges }.into());
    }
    Ok(())
}

fn check_that_implication_heads_are_declared_locally(
    implications: &[Implication],
    declarations: &Declarations,
    resource: &Term,
) -> Vec<PolarError> {
    let mut errors = vec![];
    for Implication { head, .. } in implications {
        if !declarations.contains_key(head) {
            let msg = format!(
                "Undeclared term {} referenced in implication in {} namespace. \
                Did you mean to declare it as a role, permission, or relation?",
                head.to_polar(),
                resource
            );
            let error = ParseError::ParseSugar {
                loc: head.offset(),
                msg,
                ranges: vec![],
            };
            errors.push(error.into());
        }
    }
    errors
}

impl Namespace {
    // TODO(gj): Add 'includes' feature to ensure we have a clean hook for validation _after_ all
    // Polar rules are loaded.
    pub fn add_to_kb(self, kb: &mut KnowledgeBase) -> PolarResult<()> {
        let mut errors = vec![];
        errors.extend(check_that_namespace_resource_is_registered(kb, &self.resource).err());
        errors.extend(check_for_duplicate_namespaces(&kb.namespaces, &self.resource).err());

        let Namespace {
            resource,
            roles,
            permissions,
            relations,
            implications,
        } = self;

        let declarations = index_declarations(roles, permissions, relations, &resource)?;

        errors.append(&mut check_that_implication_heads_are_declared_locally(
            &implications,
            &declarations,
            &resource,
        ));

        // TODO(gj): Emit all errors instead of just the first.
        if !errors.is_empty() {
            return Err(errors[0].clone());
        }

        kb.namespaces.add(resource.clone(), declarations);
        kb.rewrite_me_pls.insert(resource, implications);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use permute::permute;

    use std::collections::HashSet;

    use super::*;
    use crate::parser::{parse_lines, Line};
    use crate::polar::Polar;

    #[track_caller]
    fn expect_error(p: &Polar, policy: &str, expected: &str) {
        assert!(matches!(
            p.load_str(policy).unwrap_err(),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::ParseSugar {
                    msg,
                    ..
                }),
                ..
            } if msg.contains(expected)
        ));
    }

    #[test]
    fn test_namespace_rewrite_implications_with_lowercase_resource_specializer() {
        let repo_resource = term!(sym!("repo"));
        let repo_roles = term!(["reader"]);
        let repo_relations = term!(btreemap! { sym!("parent") => term!(sym!("org")) });
        let repo_declarations =
            index_declarations(Some(repo_roles), None, Some(repo_relations), &repo_resource);

        let org_resource = term!(sym!("org"));
        let org_roles = term!(["member"]);
        let org_declarations = index_declarations(Some(org_roles), None, None, &org_resource);

        let mut namespaces = Namespaces::new();
        namespaces.add(repo_resource, repo_declarations.unwrap());
        namespaces.add(org_resource, org_declarations.unwrap());
        let implication = Implication {
            head: term!("reader"),
            body: (term!("member"), Some(term!("parent"))),
        };
        let rewritten_role_role = implication
            .into_rule(&term!(sym!("repo")), &namespaces)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"has_role(actor, "reader", repo_instance: repo{}) if has_relation(org_instance, "parent", repo_instance) and has_role(actor, "member", org_instance);"#
        );
    }

    #[test]
    fn test_namespace_local_rewrite_implications() {
        let resource = term!(sym!("Org"));
        let roles = term!(["owner", "member"]);
        let permissions = term!(["invite", "create_repo"]);
        let declarations = index_declarations(Some(roles), Some(permissions), None, &resource);
        let mut namespaces = Namespaces::new();
        namespaces.add(resource, declarations.unwrap());
        let implication = Implication {
            head: term!("member"),
            body: (term!("owner"), None),
        };
        let rewritten_role_role = implication
            .into_rule(&term!(sym!("Org")), &namespaces)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"has_role(actor, "member", org: Org{}) if has_role(actor, "owner", org);"#
        );

        let implication = Implication {
            head: term!("invite"),
            body: (term!("owner"), None),
        };
        let rewritten_permission_role = implication
            .into_rule(&term!(sym!("Org")), &namespaces)
            .unwrap();
        assert_eq!(
            rewritten_permission_role.to_polar(),
            r#"has_permission(actor, "invite", org: Org{}) if has_role(actor, "owner", org);"#
        );

        let implication = Implication {
            head: term!("create_repo"),
            body: (term!("invite"), None),
        };
        let rewritten_permission_permission = implication
            .into_rule(&term!(sym!("Org")), &namespaces)
            .unwrap();
        assert_eq!(
            rewritten_permission_permission.to_polar(),
            r#"has_permission(actor, "create_repo", org: Org{}) if has_permission(actor, "invite", org);"#
        );
    }

    #[test]
    fn test_namespace_nonlocal_rewrite_implications() {
        let repo_resource = term!(sym!("Repo"));
        let repo_roles = term!(["reader"]);
        let repo_relations = term!(btreemap! { sym!("parent") => term!(sym!("Org")) });
        let repo_declarations =
            index_declarations(Some(repo_roles), None, Some(repo_relations), &repo_resource);
        let org_resource = term!(sym!("Org"));
        let org_roles = term!(["member"]);
        let org_declarations = index_declarations(Some(org_roles), None, None, &org_resource);
        let mut namespaces = Namespaces::new();
        namespaces.add(repo_resource, repo_declarations.unwrap());
        namespaces.add(org_resource, org_declarations.unwrap());
        let implication = Implication {
            head: term!("reader"),
            body: (term!("member"), Some(term!("parent"))),
        };
        let rewritten_role_role = implication
            .into_rule(&term!(sym!("Repo")), &namespaces)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"has_role(actor, "reader", repo: Repo{}) if has_relation(org, "parent", repo) and has_role(actor, "member", org);"#
        );
    }

    #[test]
    fn test_namespace_must_be_registered() {
        let p = Polar::new();
        let valid_policy = "Org{}";
        expect_error(
            &p,
            valid_policy,
            "Org namespace must be registered as a class.",
        );
        p.register_constant(sym!("Org"), term!("unimportant"));
        assert!(p.load_str(valid_policy).is_ok());
    }

    #[test]
    fn test_namespace_duplicate_namespaces() {
        let p = Polar::new();
        let invalid_policy = "Org{}Org{}";
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(&p, invalid_policy, "duplicate declaration of Org namespace");
    }

    #[test]
    fn test_namespace_with_implication_head_not_declared_locally() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(
            &p,
            r#"Org{"member" if "owner";}"#,
            r#"Undeclared term "member" referenced in implication in Org namespace. Did you mean to declare it as a role, permission, or relation?"#,
        );
    }

    #[test]
    fn test_namespace_with_relationless_implier_term_not_declared_locally() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(
            &p,
            r#"Org {
                roles=["member"];
                "member" if "owner";
            }"#,
            r#"Undeclared term "owner" referenced in implication in Org namespace. Did you mean to declare it as a role, permission, or relation?"#,
        );
    }

    #[test]
    #[ignore = "probably easier after the entity PR goes in"]
    fn test_namespace_resource_relations_can_only_appear_after_on() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"));
        expect_error(
            &p,
            r#"Repo {
                roles = ["owner"];
                relations = { parent: Org };
                "parent" if "owner";
            }"#,
            r#"Repo: resource relation "parent" can only appear in an implication following the keyword 'on'."#,
        );
    }

    #[test]
    fn test_namespace_with_undeclared_related_resource() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"));
        p.register_constant(sym!("Org"), term!("unimportant"));
        let policy = r#"
            Org { relations = { owner: User }; }
            Repo { relations = { parent: Org };
                   roles = ["writer"];
                   "writer" if "owner" on "parent"; }
        "#;
        panic!("{}", p.load_str(policy).unwrap_err());

        // let policy = r#"Repo {
        //     roles = [ "writer" ];
        //     relations = { parent: Org };
        //     "writer" if "owner" on "parent";
        // }"#;
        // panic!("{}", p.load_str(policy).unwrap_err());
    }

    #[test]
    fn test_namespace_with_circular_implications() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"));
        let policy = r#"Repo {
            roles = [ "writer" ];
            "writer" if "writer";
        }"#;
        panic!("{}", p.load_str(policy).unwrap_err());

        // let policy = r#"Repo {
        //     roles = [ "writer", "reader" ];
        //     "writer" if "reader";
        //     "reader" if "writer";
        // }"#;
        // panic!("{}", p.load_str(policy).unwrap_err());
        //
        // let policy = r#"Repo {
        //     roles = [ "writer", "reader", "admin" ];
        //     "admin" if "reader";
        //     "writer" if "admin";
        //     "reader" if "writer";
        // }"#;
        // panic!("{}", p.load_str(policy).unwrap_err());
    }

    #[test]
    fn test_namespace_with_undeclared_cross_resource_implier_term() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"));
        let policy = r#"Repo {
                roles = ["writer"];
                relations = { parent: Org };
                "writer" if "owner" on "parent";
            }"#;
        panic!("{}", p.load_str(policy).unwrap_err());
        // expect_error(
        //     &p,
        //     r#"Undeclared term "owner" referenced in implication in Org namespace. Did you mean to declare it as a role, permission, or relation?"#,
        // );
    }

    #[test]
    fn test_namespace_with_unregistered_relation_type() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"));
        expect_error(
            &p,
            r#"Repo { relations = { parent: Org }; }"#,
            "Type 'Org' in relation 'parent: Org' must be registered as a class.",
        );
    }

    #[test]
    fn test_namespace_with_clashing_declarations() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));

        expect_error(
            &p,
            r#"Org{
              roles = ["egg","egg"];
              "egg" if "egg";
            }"#,
            r#"Org: Duplicate declaration of "egg" in the roles list."#,
        );

        expect_error(
            &p,
            r#"Org{
              roles = ["egg","tootsie"];
              permissions = ["spring","egg"];

              "egg" if "tootsie";
              "tootsie" if "spring";
            }"#,
            r#"Org: "egg" declared as a permission but it was previously declared as a role."#,
        );

        expect_error(
            &p,
            r#"Org{
              permissions = [ "egg" ];
              relations = { egg: Roll };
            }"#,
            r#"Org: 'egg' declared as a relation but it was previously declared as a permission."#,
        );
    }

    #[test]
    fn test_namespace_parsing_permutations() {
        use std::iter::FromIterator;

        // Policy pieces
        let roles = r#"roles = ["writer", "reader"];"#;
        let permissions = r#"permissions = ["push", "pull"];"#;
        let relations = r#"relations = { creator: User, parent: Org };"#;
        let implications = vec![
            r#""pull" if "reader";"#,
            r#""push" if "writer";"#,
            r#""writer" if "creator";"#,
            r#""reader" if "member" on "parent";"#,
        ];

        // Maximal namespace
        let namespace = Namespace {
            resource: term!(sym!("Repo")),
            roles: Some(term!(["writer", "reader"])),
            permissions: Some(term!(["push", "pull"])),
            relations: Some(term!(btreemap! {
                sym!("creator") => term!(sym!("User")),
                sym!("parent") => term!(sym!("Org")),
            })),
            implications: vec![
                // TODO(gj): implication! macro
                Implication {
                    head: term!("pull"),
                    body: (term!("reader"), None),
                },
                Implication {
                    head: term!("push"),
                    body: (term!("writer"), None),
                },
                Implication {
                    head: term!("writer"),
                    body: (term!("creator"), None),
                },
                Implication {
                    head: term!("reader"),
                    body: (term!("member"), Some(term!("parent"))),
                },
            ],
        };

        // Helpers

        let equal = |line: &Line, expected: &Namespace| match line {
            Line::Namespace(parsed) => {
                let parsed_implications: HashSet<&Implication> =
                    HashSet::from_iter(&parsed.implications);
                let expected_implications = HashSet::from_iter(&expected.implications);
                parsed.resource == expected.resource
                    && parsed.roles == expected.roles
                    && parsed.permissions == expected.permissions
                    && parsed.relations == expected.relations
                    && parsed_implications == expected_implications
            }
            _ => false,
        };

        let test_case = |parts: Vec<&str>, expected: &Namespace| {
            for permutation in permute(parts).into_iter() {
                let mut policy = "Repo {\n".to_owned();
                policy += &permutation.join("\n");
                policy += "}";
                assert!(equal(&parse_lines(0, &policy).unwrap()[0], expected));
            }
        };

        // Test each case with and without implications.
        let test_cases = |parts: Vec<&str>, expected: &Namespace| {
            let mut parts_with_implications = parts.clone();
            parts_with_implications.append(&mut implications.clone());
            test_case(parts_with_implications, expected);

            let expected_without_implications = Namespace {
                implications: vec![],
                ..expected.clone()
            };
            test_case(parts, &expected_without_implications);
        };

        // Cases

        // Roles, Permissions, Relations
        test_cases(vec![roles, permissions, relations], &namespace);

        // Roles, Permissions, _________
        let expected = Namespace {
            relations: None,
            ..namespace.clone()
        };
        test_cases(vec![roles, permissions], &expected);

        // Roles, ___________, Relations
        let expected = Namespace {
            permissions: None,
            ..namespace.clone()
        };
        test_cases(vec![roles, relations], &expected);

        // _____, Permissions, Relations
        let expected = Namespace {
            roles: None,
            ..namespace.clone()
        };
        test_cases(vec![permissions, relations], &expected);

        // Roles, ___________, _________
        let expected = Namespace {
            permissions: None,
            relations: None,
            ..namespace.clone()
        };
        test_cases(vec![roles], &expected);

        // _____, Permissions, _________
        let expected = Namespace {
            roles: None,
            relations: None,
            ..namespace.clone()
        };
        test_cases(vec![permissions], &expected);

        // _____, ___________, Relations
        let expected = Namespace {
            roles: None,
            permissions: None,
            ..namespace.clone()
        };
        test_cases(vec![relations], &expected);

        // _____, ___________, _________
        let expected = Namespace {
            roles: None,
            permissions: None,
            relations: None,
            ..namespace
        };
        test_cases(vec![], &expected);
    }

    #[test]
    fn test_namespace_declaration_keywords() {
        let p = Polar::new();
        expect_error(
            &p,
            r#"Org{roles={};}"#,
            r#"Expected 'roles' declaration to be a list of strings; found a dictionary:"#,
        );
        expect_error(
            &p,
            r#"Org{relations=[];}"#,
            r#"Expected 'relations' declaration to be a dictionary; found a list:"#,
        );
        expect_error(
            &p,
            r#"Org{foo=[];}"#,
            r#"Unexpected declaration 'foo'. Did you mean for this to be 'roles = [ ... ];' or 'permissions = [ ... ];'?"#,
        );
        expect_error(
            &p,
            r#"Org{foo={};}"#,
            r#"Unexpected declaration 'foo'. Did you mean for this to be 'relations = { ... };'?"#,
        );
        expect_error(
            &p,
            r#"Org{"foo" if "bar" onn "baz";}"#,
            r#"Unexpected relation keyword 'onn'. Did you mean 'on'?"#,
        );
    }

    #[test]
    fn test_namespace_declaration_keywords_are_not_reserved_words() {
        let p = Polar::new();
        p.load_str("roles(permissions, on) if permissions.relations = on;")
            .unwrap();
    }
}
