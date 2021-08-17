use std::collections::{HashMap, HashSet};

use super::error::{ParseError, PolarResult, RuntimeError};
use super::kb::KnowledgeBase;
use super::rules::*;
use super::terms::*;

// TODO(gj): disallow same string to be declared as a perm/role and a relation.
// This'll come into play for "owner"-style actor relationships.

#[derive(Clone, Debug)]
pub enum Declaration {
    Role,
    Permission,
    Relation(Term),
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
            Declaration::Role => sym!("role"),
            Declaration::Permission => sym!("permission"),
            Declaration::Relation(_) => sym!("relation"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Namespace {
    pub resource: Term,
    // TODO(gj): maybe HashSet instead of Vec so we can easily catch duplicates?
    pub roles: Option<Term>,
    pub permissions: Option<Term>,
    pub relations: Option<Term>,
    pub implications: HashSet<(Term, Term, Option<Term>)>,
}

#[derive(Clone, Default)]
pub struct Namespaces {
    inner: HashMap<Term, Declarations>,
}

impl Namespaces {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    fn add(&mut self, resource: Term, declarations: Declarations) -> Option<Declarations> {
        self.inner.insert(resource, declarations)
    }

    fn remove(&mut self, resource: &Term) -> Option<Declarations> {
        self.inner.remove(resource)
    }

    fn exists(&self, resource: &Term) -> bool {
        self.inner.contains_key(resource)
    }

    fn get_declaration(&self, resource: &Term, name: &Term) -> PolarResult<&Declaration> {
        if let Some(declaration) = self.inner[resource].get(name) {
            Ok(declaration)
        } else {
            Err(ParseError::IntegerOverflow {
                loc: name.offset(),
                token: format!(
                    "Undeclared term {} referenced in implication in {} namespace. \
                        Did you mean to declare it as a role, permission, or relation?",
                    name.to_polar(),
                    resource
                ),
            }
            .into())
        }
    }

    fn get_related_type(&self, resource: &Term, relation: &Term) -> PolarResult<&Term> {
        self.get_declaration(resource, relation)?.as_relation()
    }

    fn local_predicate_name(&self, resource: &Term, name: &Term) -> PolarResult<Symbol> {
        Ok(self.get_declaration(resource, name)?.as_predicate())
    }

    fn cross_resource_predicate_name(
        &self,
        resource: &Term,
        relation: &Term,
        name: &Term,
    ) -> PolarResult<Symbol> {
        let related_type = self.get_related_type(resource, relation)?;
        self.local_predicate_name(related_type, name)
    }
}

fn index_declarations(
    roles: Option<Term>,
    permissions: Option<Term>,
    relations: Option<Term>,
) -> HashMap<Term, Declaration> {
    // Fold List<role> => HashMap<role, Declaration>
    let declarations = roles
        .into_iter()
        .flat_map(|inner| inner.value().as_list().unwrap().clone())
        .fold(HashMap::new(), |mut acc, role| {
            acc.insert(role, Declaration::Role);
            acc
        });

    // Fold List<permission> => HashMap<permission_or_role, Declaration>
    let declarations = permissions
        .into_iter()
        .flat_map(|inner| inner.value().as_list().unwrap().clone())
        .fold(declarations, |mut acc, permission| {
            acc.insert(permission, Declaration::Permission);
            acc
        });

    // Fold Dict<relation, resource> => HashMap<permission_or_role_or_relation, Declaration>
    relations
        .into_iter()
        .flat_map(|inner| inner.value().as_dict().unwrap().fields.clone())
        .fold(declarations, |mut acc, (relation, resource)| {
            acc.insert(
                resource.clone_with_value(value!(relation.0.as_str())),
                Declaration::Relation(resource),
            );
            acc
        })
}

fn resource_as_var(resource: &Term) -> PolarResult<Value> {
    let name = &resource.value().as_symbol()?.0;
    let mut lowercased = name.to_lowercase();

    // If the resource's name is already lowercase, append "_instance" to distinguish the variable
    // name from the resource's name.
    if &lowercased == name {
        lowercased += "_instance";
    }

    Ok(value!(sym!(lowercased)))
}

fn rewrite_implier_as_rule_body(
    implier: &Term,
    relation: &Option<Term>,
    namespaces: &Namespaces,
    resource: &Term,
) -> PolarResult<Term> {
    let resource_var = implier.clone_with_value(resource_as_var(resource)?);
    let actor_var = implier.clone_with_value(value!(sym!("actor")));
    if let Some(relation) = relation {
        // TODO(gj): what if the relation is with the same type? E.g.,
        // `Dir { relations = { parent: Dir }; }`. This might cause Polar to loop.
        let related_type = namespaces.get_related_type(resource, relation)?;
        let related_resource_var = relation.clone_with_value(resource_as_var(related_type)?);

        let relation_call = relation.clone_with_value(value!(Call {
            name: sym!("relation"),
            // For example: vec![org, "parent", repo]
            args: vec![related_resource_var.clone(), relation.clone(), resource_var],
            kwargs: None
        }));

        let implier_call = implier.clone_with_value(value!(Call {
            name: namespaces.cross_resource_predicate_name(resource, relation, implier)?,
            // For example: vec![actor, "owner", org]
            args: vec![actor_var, implier.clone(), related_resource_var],
            kwargs: None
        }));
        Ok(implier.clone_with_value(value!(op!(And, relation_call, implier_call))))
    } else {
        let implier_call = implier.clone_with_value(value!(Call {
            name: namespaces.local_predicate_name(resource, implier)?,
            args: vec![actor_var, implier.clone(), resource_var],
            kwargs: None
        }));
        Ok(implier.clone_with_value(value!(op!(And, implier_call))))
    }
}

fn rewrite_implied_as_rule_params(implied: &Term, resource: &Term) -> PolarResult<Vec<Parameter>> {
    let resource_name = &resource.value().as_symbol()?.0;
    Ok(vec![
        Parameter {
            parameter: implied.clone_with_value(value!(sym!("actor"))),
            specializer: None,
        },
        Parameter {
            parameter: implied.clone(),
            specializer: None,
        },
        Parameter {
            parameter: implied.clone_with_value(resource_as_var(resource)?),
            specializer: Some(
                resource.clone_with_value(value!(pattern!(instance!(resource_name)))),
            ),
        },
    ])
}

fn rewrite_implication(
    implication: (Term, Term, Option<Term>),
    resource: &Term,
    namespaces: &Namespaces,
) -> PolarResult<Rule> {
    let (implied, implier, relation) = implication;
    let rule_name = namespaces.local_predicate_name(resource, &implied)?;
    let params = rewrite_implied_as_rule_params(&implied, resource)?;
    let body = rewrite_implier_as_rule_body(&implier, &relation, namespaces, resource)?;

    // TODO(gj): I think this will only be None in tests. Assert that.
    let src_id = resource.get_source_id().unwrap_or(0);
    let start = implied.offset();
    let end = relation.map_or_else(|| implier.offset_to_end(), |r| r.offset_to_end());
    Ok(Rule::new_from_parser(
        src_id, start, end, rule_name, params, body,
    ))
}

fn check_for_duplicate_namespaces(namespaces: &Namespaces, resource: &Term) -> PolarResult<()> {
    if namespaces.exists(resource) {
        return Err(ParseError::IntegerOverflow {
            loc: resource.offset(),
            // TODO(gj): better error message, e.g.:
            //               duplicate namespace declaration: Org { ... } defined on line XX of file YY
            //                                                previously defined on line AA of file BB
            token: format!("duplicate declaration of {} namespace", resource),
        }
        .into());
    }
    Ok(())
}

// TODO(gj): no way to know in the core if `resource` was registered as a class or a constant.
fn check_that_namespace_resource_is_registered(
    kb: &KnowledgeBase,
    resource: &Term,
) -> PolarResult<()> {
    if !kb.is_constant(resource.value().as_symbol()?) {
        return Err(ParseError::IntegerOverflow {
            loc: resource.offset(),
            // TODO(gj): better error message
            token: format!(
                "{} namespace must be registered as a class",
                resource.to_polar()
            ),
        }
        .into());
    }
    Ok(())
}

// fn caret_me_captain(string: &str, implier_too: bool) -> {
// source_lines(source, offset, 0)
// }

fn check_for_empty_namespace(namespace: &Namespace) -> PolarResult<()> {
    let Namespace {
        resource,
        roles,
        permissions,
        relations,
        implications,
    } = namespace;
    if roles.is_none() && permissions.is_none() && relations.is_none() && implications.is_empty() {
        let loc = resource.offset();
        let token = format!(
            "{} namespace is empty. Please add roles, permissions, and/or relations, or delete it.",
            resource
        );
        return Err(ParseError::IntegerOverflow { loc, token }.into());
    }
    Ok(())
}

fn check_empty_declarations(namespace: &Namespace) -> PolarResult<()> {
    let Namespace {
        resource,
        roles,
        permissions,
        relations,
        ..
    } = namespace;

    let roles_empty = roles
        .as_ref()
        .map_or(false, |roles| roles.value().as_list().unwrap().is_empty());
    let permissions_empty = permissions.as_ref().map_or(false, |permissions| {
        permissions.value().as_list().unwrap().is_empty()
    });
    let relations_empty = relations.as_ref().map_or(false, |relations| {
        relations.value().as_dict().unwrap().is_empty()
    });

    match (roles_empty, permissions_empty, relations_empty) {
        (true, true, true) => Err(ParseError::IntegerOverflow {
            loc: resource.offset(),
            token: format!(
                "{} namespace contains empty roles, permissions, and relations declarations. \
                        Please add roles, permissions, and relations or delete the declarations.",
                resource
            ),
        }
        .into()),
        (true, true, _) => Err(ParseError::IntegerOverflow {
            loc: resource.offset(),
            token: format!(
                "{} namespace contains empty roles and permissions declarations. \
                        Please add roles and permissions or delete the declarations.",
                resource
            ),
        }
        .into()),
        (true, _, true) => Err(ParseError::IntegerOverflow {
            loc: resource.offset(),
            token: format!(
                "{} namespace contains empty roles and relations declarations. \
                        Please add roles and relations or delete the declarations.",
                resource
            ),
        }
        .into()),
        (_, true, true) => Err(ParseError::IntegerOverflow {
            loc: resource.offset(),
            token: format!(
                "{} namespace contains empty permissions and relations declarations. \
                        Please add permissions and relations or delete the declarations.",
                resource
            ),
        }
        .into()),
        (true, _, _) => Err(ParseError::IntegerOverflow {
            loc: roles.as_ref().unwrap().offset(),
            token: format!(
                "{} namespace contains an empty roles declaration. \
                        Please add roles or delete the declaration.",
                resource
            ),
        }
        .into()),
        (_, true, _) => Err(ParseError::IntegerOverflow {
            loc: resource.offset(),
            token: format!(
                "{} namespace contains an empty permissions declaration. \
                        Please add permissions or delete the declaration.",
                resource
            ),
        }
        .into()),
        (_, _, true) => Err(ParseError::IntegerOverflow {
            loc: resource.offset(),
            token: format!(
                "{} namespace contains an empty relations declaration. \
                        Please add relations or delete the declaration.",
                resource
            ),
        }
        .into()),
        (false, false, false) => Ok(()),
    }
}

fn check_all_permissions_involved_in_implications(namespace: &Namespace) -> PolarResult<()> {
    let Namespace {
        resource,
        permissions,
        implications,
        ..
    } = namespace;
    if let Some(ref permissions_list) = permissions {
        let permissions = permissions_list.value().as_list()?;

        if implications.is_empty() {
            return Err(ParseError::IntegerOverflow {
                loc: permissions_list.offset(),
                token: format!(
                    "{}: all permissions must be involved in at least one implication.",
                    resource.to_polar(),
                ),
            }
            .into());
        }

        for permission in permissions.iter() {
            let implication_references_permission =
                implications
                    .iter()
                    .any(|(implied, implier, maybe_relation)| {
                        // Permission is referenced on the 'implied' side of an implication or
                        // on the 'implier' side of a _local_ implication. If permission shows
                        // up on the 'implier' side of a non-local implication, that's actually
                        // a reference to a permission of the same name declared in the other
                        // resource namespace.
                        permission == implied || (permission == implier && maybe_relation.is_none())
                    });

            if !implication_references_permission {
                return Err(ParseError::IntegerOverflow {
                    loc: permission.offset(),
                    token: format!(
                        "{}: permission {} must be involved in at least one implication.",
                        resource.to_polar(),
                        permission.to_polar()
                    ),
                }
                .into());
            }
        }
    }
    Ok(())
}

impl KnowledgeBase {
    pub fn add_namespace(&mut self, namespace: Namespace) -> PolarResult<()> {
        check_that_namespace_resource_is_registered(self, &namespace.resource)?;
        check_for_duplicate_namespaces(&self.namespaces, &namespace.resource)?;
        check_for_empty_namespace(&namespace)?;
        check_empty_declarations(&namespace)?;
        check_all_permissions_involved_in_implications(&namespace)?;

        let Namespace {
            resource,
            roles,
            permissions,
            relations,
            implications,
        } = namespace;

        let declarations = index_declarations(roles, permissions, relations);
        self.namespaces.add(resource.clone(), declarations);

        // TODO(gj): what to do for `on "parent_org"` if Org{} namespace hasn't
        // been processed yet? Whether w/ multiple load_file calls or some future
        // `import` feature, we probably don't want to force a specific load order
        // on folks if we don't have to. Maybe add as-of-yet uncheckable
        // implications into a queue that we check once all files are loaded /
        // imported? That might work for the future import case, but how would we
        // know when the final load_file call has been made? Answer: hax.

        for implication in implications {
            let rewritten = rewrite_implication(implication, &resource, &self.namespaces);
            let rule = match rewritten {
                Ok(rule) => rule,
                Err(e) => {
                    // If we error out at this point, remove the namespace entry.
                    self.namespaces.remove(&resource);
                    return Err(e);
                }
            };
            self.add_rule(rule);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use permute::permute;

    use super::*;
    use crate::parser::{parse_lines, Line};
    use crate::polar::Polar;

    #[track_caller]
    fn expect_error(p: &Polar, policy: &str, expected: &str) {
        assert!(matches!(
            p.load_str(policy).unwrap_err(),
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::IntegerOverflow {
                    token,
                    ..
                }),
                ..
            } if token == expected
        ));
    }

    #[test]
    fn test_namespace_rewrite_implications_with_lowercase_resource_specializer() {
        let repo_roles = term!(["reader"]);
        let repo_relations = term!(btreemap! { sym!("parent") => term!(sym!("org")) });
        let repo_declarations = index_declarations(Some(repo_roles), None, Some(repo_relations));

        let org_roles = term!(["member"]);
        let org_declarations = index_declarations(Some(org_roles), None, None);

        let mut namespaces = Namespaces::new();
        namespaces.add(term!(sym!("repo")), repo_declarations);
        namespaces.add(term!(sym!("org")), org_declarations);
        let rewritten_role_role = rewrite_implication(
            (term!("reader"), term!("member"), Some(term!("parent"))),
            &term!(sym!("repo")),
            &namespaces,
        )
        .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"role(actor, "reader", repo_instance: repo{}) if relation(org_instance, "parent", repo_instance) and role(actor, "member", org_instance);"#
        );
    }

    #[test]
    fn test_namespace_local_rewrite_implications() {
        let roles = term!(["owner", "member"]);
        let permissions = term!(["invite", "create_repo"]);
        let declarations = index_declarations(Some(roles), Some(permissions), None);
        let mut namespaces = Namespaces::new();
        namespaces.add(term!(sym!("Org")), declarations);
        let rewritten_role_role = rewrite_implication(
            (term!("member"), term!("owner"), None),
            &term!(sym!("Org")),
            &namespaces,
        )
        .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"role(actor, "member", org: Org{}) if role(actor, "owner", org);"#
        );

        let rewritten_permission_role = rewrite_implication(
            (term!("invite"), term!("owner"), None),
            &term!(sym!("Org")),
            &namespaces,
        )
        .unwrap();
        assert_eq!(
            rewritten_permission_role.to_polar(),
            r#"permission(actor, "invite", org: Org{}) if role(actor, "owner", org);"#
        );

        let rewritten_permission_permission = rewrite_implication(
            (term!("create_repo"), term!("invite"), None),
            &term!(sym!("Org")),
            &namespaces,
        )
        .unwrap();
        assert_eq!(
            rewritten_permission_permission.to_polar(),
            r#"permission(actor, "create_repo", org: Org{}) if permission(actor, "invite", org);"#
        );
    }

    #[test]
    fn test_namespace_nonlocal_rewrite_implications() {
        let repo_roles = term!(["reader"]);
        let repo_relations = term!(btreemap! { sym!("parent") => term!(sym!("Org")) });
        let repo_declarations = index_declarations(Some(repo_roles), None, Some(repo_relations));
        let org_roles = term!(["member"]);
        let org_declarations = index_declarations(Some(org_roles), None, None);
        let mut namespaces = Namespaces::new();
        namespaces.add(term!(sym!("Repo")), repo_declarations);
        namespaces.add(term!(sym!("Org")), org_declarations);
        let rewritten_role_role = rewrite_implication(
            (term!("reader"), term!("member"), Some(term!("parent"))),
            &term!(sym!("Repo")),
            &namespaces,
        )
        .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"role(actor, "reader", repo: Repo{}) if relation(org, "parent", repo) and role(actor, "member", org);"#
        );
    }

    #[test]
    fn test_namespace_must_be_registered() {
        let p = Polar::new();
        let valid_policy = r#"Org{roles=["owner"];}"#;
        expect_error(
            &p,
            valid_policy,
            "Org namespace must be registered as a class",
        );
        p.register_constant(sym!("Org"), term!("unimportant"));
        assert!(p.load_str(valid_policy).is_ok());
    }

    #[test]
    fn test_namespace_duplicate_namespaces() {
        let p = Polar::new();
        let invalid_policy = r#"
            Org { roles=["owner"]; }
            Org { roles=["member"]; }
        "#;
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(&p, invalid_policy, "duplicate declaration of Org namespace");
    }

    #[test]
    fn test_namespace_empty() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(
            &p,
            "Org{}",
            "Org namespace is empty. Please add roles, permissions, and/or relations, or delete it."
        );
    }

    #[test]
    fn test_namespace_with_empty_declarations() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));

        expect_error(
            &p,
            "Org { roles=[]; permissions=[]; relations={}; }",
            "Org namespace contains empty roles, permissions, and relations declarations. \
            Please add roles, permissions, and relations or delete the declarations.",
        );

        expect_error(
            &p,
            "Org { roles=[]; }",
            "Org namespace contains an empty roles declaration. \
            Please add roles or delete the declaration.",
        );
    }

    #[test]
    fn test_namespace_with_permissions_but_no_implications() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(
            &p,
            r#"Org{permissions=["invite","create_repo"];}"#,
            r#"Org: all permissions must be involved in at least one implication."#,
        );
    }

    #[test]
    fn test_namespace_with_permission_not_involved_in_implication() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));

        expect_error(
            &p,
            r#"Org {
                permissions=["invite","create_repo","ban"];
                "invite" if "ban";
            }"#,
            r#"Org: permission "create_repo" must be involved in at least one implication."#,
        );
    }

    #[test]
    fn test_namespace_with_only_roles() {
        assert_eq!(
            parse_lines(0, r#"Org{roles=["owner",];}"#).unwrap()[0],
            Line::Namespace(Namespace {
                resource: term!(sym!("Org")),
                roles: Some(term!(["owner"])),
                permissions: None,
                relations: None,
                implications: hashset! {},
            })
        );
        assert_eq!(
            parse_lines(0, r#"Org{roles=["owner","member",];}"#).unwrap()[0],
            Line::Namespace(Namespace {
                resource: term!(sym!("Org")),
                roles: Some(term!(["owner", "member"])),
                permissions: None,
                relations: None,
                implications: hashset! {},
            })
        );
    }

    #[test]
    fn test_namespace_roles_and_role_implications() {
        assert_eq!(
            parse_lines(
                0,
                r#"Org {
                     roles=["owner","member"];
                     "member" if "owner";
                }"#
            )
            .unwrap()[0],
            Line::Namespace(Namespace {
                resource: term!(sym!("Org")),
                roles: Some(term!(["owner", "member"])),
                permissions: None,
                relations: None,
                implications: hashset! {(term!("member"), term!("owner"), None)},
            })
        );
    }

    #[test]
    fn test_namespace_with_only_implications() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(
            &p,
            r#"Org{"member" if "owner";}"#,
            r#"Undeclared term "member" referenced in implication in Org namespace. Did you mean to declare it as a role, permission, or relation?"#,
        );
    }

    #[test]
    fn test_namespace_with_implier_term_not_declared_locally() {
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
    fn test_namespace_parsing_permutations() {
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
            implications: hashset! {
                (term!("pull"), term!("reader"), None),
                (term!("push"), term!("writer"), None),
                (term!("writer"), term!("creator"), None),
                (term!("reader"), term!("member"), Some(term!("parent"))),
            },
        };

        // Helpers

        let test_case = |parts: Vec<&str>, expected: &Namespace| {
            for permutation in permute(parts).into_iter() {
                let mut policy = "Repo {\n".to_owned();
                policy += &permutation.join("\n");
                policy += "}";
                assert_eq!(
                    parse_lines(0, &policy).unwrap()[0],
                    Line::Namespace(expected.clone())
                );
            }
        };

        // Test each case with and without implications.
        let test_cases = |parts: Vec<&str>, expected: &Namespace| {
            let mut parts_with_implications = parts.clone();
            parts_with_implications.append(&mut implications.clone());
            test_case(parts_with_implications, expected);

            let expected_without_implications = Namespace {
                implications: hashset! {},
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
}
