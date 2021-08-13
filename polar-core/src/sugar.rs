use std::collections::HashMap;
use std::sync::Arc;

use super::error::{ParseError, PolarResult};
use super::kb::KnowledgeBase;
use super::parser::Namespace;
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

fn transform_declarations(
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

fn rewrite_implication(
    implication: (Term, Term, Option<Term>),
    resource: Term,
    namespaces: HashMap<Term, HashMap<Term, Declaration>>,
) -> Rule {
    let (implied, implier, relation) = implication;
    let body = if let Some(relation) = relation {
        let resource_name = &resource.value().as_symbol().unwrap().0;
        let implier_resource = implier.clone_with_value(value!(sym!(resource_name.to_lowercase())));
        let implier_actor = implier.clone_with_value(value!(sym!("actor")));
        let related_resource = &namespaces[&resource][&relation];
        let (implier_predicate, xxx) = if let Declaration::Relation(relation) = related_resource {
            match namespaces[relation][&implier] {
                Declaration::Role => (sym!("role"), relation),
                Declaration::Permission => (sym!("permission"), relation),
                _ => unreachable!(),
            }
        } else {
            panic!();
        };

        let related_resource_name = &xxx.value().as_symbol().unwrap().0;
        let related_resource =
            relation.clone_with_value(value!(sym!(related_resource_name.to_lowercase())));

        let relation_predicate = relation.clone_with_value(value!(Call {
            name: sym!("relation"),
            args: vec![related_resource.clone(), relation.clone(), implier_resource],
            kwargs: None
        }));

        let args = vec![implier_actor, implier.clone(), related_resource];
        implier.clone_with_value(value!(op!(
            And,
            relation_predicate,
            implier.clone_with_value(value!(Call {
                name: implier_predicate,
                args,
                kwargs: None
            }))
        )))
    } else {
        let resource_name = &resource.value().as_symbol().unwrap().0;
        let implier_resource = implier.clone_with_value(value!(sym!(resource_name.to_lowercase())));
        let implier_actor = implier.clone_with_value(value!(sym!("actor")));
        let implier_predicate = match namespaces[&resource][&implier] {
            Declaration::Role => sym!("role"),
            Declaration::Permission => sym!("permission"),
            _ => unreachable!(),
        };
        let args = vec![implier_actor, implier.clone(), implier_resource];
        implier.clone_with_value(value!(op!(
            And,
            implier.clone_with_value(value!(Call {
                name: implier_predicate,
                args,
                kwargs: None
            }))
        )))
    };

    let rule_name = match namespaces[&resource][&implied] {
        Declaration::Role => sym!("role"),
        Declaration::Permission => sym!("permission"),
        _ => unreachable!(),
    };
    let resource_name = &resource.value().as_symbol().unwrap().0;
    let params = vec![
        Parameter {
            parameter: implied.clone_with_value(value!(sym!("actor"))),
            specializer: None,
        },
        Parameter {
            parameter: implied.clone(),
            specializer: None,
        },
        Parameter {
            parameter: implied.clone_with_value(value!(sym!(resource_name.to_lowercase()))),
            specializer: Some(
                resource.clone_with_value(value!(pattern!(instance!(resource_name)))),
            ),
        },
    ];
    Rule::new_from_parser(0, 0, 0, rule_name, params, body)
}

impl KnowledgeBase {
    pub fn add_namespace(&mut self, namespace: Namespace) -> PolarResult<()> {
        let Namespace {
            resource,
            roles,
            permissions,
            relations,
            implications,
        } = namespace;

        // TODO(gj): no way to know in the core if `resource` was registered as a class
        // or a constant.
        if !self.is_constant(resource.value().as_symbol()?) {
            return Err(ParseError::IntegerOverflow {
                loc: resource.offset(),
                // TODO(gj): better error message
                token: format!(
                    "namespace {} must be registered as a class",
                    resource.to_polar()
                ),
            }
            .into());
        }

        // Check for duplicate namespace definitions.
        if self.namespaces.contains_key(&resource) {
            return Err(ParseError::IntegerOverflow {
                loc: resource.offset(),
                // TODO(gj): better error message, e.g.:
                //               duplicate namespace declaration: Org { ... } defined on line XX of file YY
                //                                                previously defined on line AA of file BB
                token: format!("duplicate declaration of {} namespace", resource),
            }
            .into());
        }

        let declarations = transform_declarations(roles, permissions, relations);
        self.namespaces.insert(resource.clone(), declarations);

        // TODO(gj): what to do for `on "parent_org"` if Org{} namespace hasn't
        // been processed yet? Whether w/ multiple load_file calls or some future
        // `import` feature, we probably don't want to force a specific load order
        // on folks if we don't have to. Maybe add as-of-yet uncheckable
        // implications into a queue that we check once all files are loaded /
        // imported? That might work for the future import case, but how would we
        // know when the final load_file call has been made? Answer: hax.

        if let Some(implications) = implications {
            for implication in implications {
                let rule =
                    rewrite_implication(implication, resource.clone(), self.namespaces.clone());
                let generic_rule = self
                    .rules
                    .entry(rule.name.clone())
                    .or_insert_with(|| GenericRule::new(rule.name.clone(), vec![]));
                generic_rule.add_rule(Arc::new(rule));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatting::ToPolarString;

    #[test]
    fn test_namespace_local_rewrite_implications() {
        let roles = term!(["owner", "member"]);
        let permissions = term!(["invite", "create_repo"]);
        let declarations = transform_declarations(Some(roles), Some(permissions), None);
        let mut namespaces = HashMap::new();
        namespaces.insert(term!(sym!("Org")), declarations);
        let rewritten_role_role = rewrite_implication(
            (term!("member"), term!("owner"), None),
            term!(sym!("Org")),
            namespaces.clone(),
        );
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"role(actor, "member", org: Org{}) if role(actor, "owner", org);"#
        );

        let rewritten_permission_role = rewrite_implication(
            (term!("invite"), term!("owner"), None),
            term!(sym!("Org")),
            namespaces.clone(),
        );
        assert_eq!(
            rewritten_permission_role.to_polar(),
            r#"permission(actor, "invite", org: Org{}) if role(actor, "owner", org);"#
        );

        let rewritten_permission_permission = rewrite_implication(
            (term!("create_repo"), term!("invite"), None),
            term!(sym!("Org")),
            namespaces,
        );
        assert_eq!(
            rewritten_permission_permission.to_polar(),
            r#"permission(actor, "create_repo", org: Org{}) if permission(actor, "invite", org);"#
        );
    }

    #[test]
    fn test_namespace_nonlocal_rewrite_implications() {
        let repo_roles = term!(["reader"]);
        let repo_relations = term!(btreemap! { sym!("parent") => term!(sym!("Org")) });
        let repo_declarations =
            transform_declarations(Some(repo_roles), None, Some(repo_relations));
        let org_roles = term!(["member"]);
        let org_declarations = transform_declarations(Some(org_roles), None, None);
        let mut namespaces = HashMap::new();
        namespaces.insert(term!(sym!("Repo")), repo_declarations);
        namespaces.insert(term!(sym!("Org")), org_declarations);
        let rewritten_role_role = rewrite_implication(
            (term!("reader"), term!("member"), Some(term!("parent"))),
            term!(sym!("Repo")),
            namespaces,
        );
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"role(actor, "reader", repo: Repo{}) if relation(org, "parent", repo) and role(actor, "member", org);"#
        );
    }
}
