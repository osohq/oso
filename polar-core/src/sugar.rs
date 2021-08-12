use std::collections::HashMap;
use std::sync::Arc;

use super::error::{ParseError, PolarResult};
use super::kb::KnowledgeBase;
use super::parser::ResourceNamespace;
use super::rules::*;
use super::terms::*;

#[derive(Clone)]
pub enum Declaration {
    Role,
    Permission,
}

fn transform_declarations(
    roles: Option<Vec<Term>>,
    permissions: Option<Vec<Term>>,
) -> HashMap<Term, Declaration> {
    // Fold Vec<role> => HashMap<role, Declaration>
    let declarations = roles
        .into_iter()
        .flatten()
        .fold(HashMap::new(), |mut acc, role| {
            acc.insert(role, Declaration::Role);
            acc
        });

    // Fold Vec<permission> => HashMap<permission_or_role, Declaration>
    permissions
        .into_iter()
        .flatten()
        .fold(declarations, |mut acc, permission| {
            acc.insert(permission, Declaration::Permission);
            acc
        })
}

fn rewrite_implication(
    implied: Term,
    implier: Term,
    resource: Term,
    declarations: HashMap<Term, Declaration>,
) -> Rule {
    let resource_name = &resource.value().as_symbol().unwrap().0;
    let implier_resource = implier.clone_with_value(value!(sym!(resource_name.to_lowercase())));
    let implier_actor = implier.clone_with_value(value!(sym!("actor")));
    let implier_predicate = match declarations[&implier] {
        Declaration::Role => sym!("role"),
        Declaration::Permission => sym!("permission"),
    };
    let args = vec![implier_actor, implier.clone(), implier_resource];
    let body = implier.clone_with_value(value!(op!(
        And,
        implier.clone_with_value(value!(Call {
            name: implier_predicate,
            args,
            kwargs: None
        }))
    )));

    let rule_name = match declarations[&implied] {
        Declaration::Role => sym!("role"),
        Declaration::Permission => sym!("permission"),
    };
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
    pub fn add_resource_namespace(&mut self, namespace: ResourceNamespace) -> PolarResult<()> {
        let ResourceNamespace {
            resource,
            roles,
            permissions,
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

        // Check for duplicate resource namespace definitions.
        if self.resource_namespaces.contains_key(&resource) {
            return Err(ParseError::IntegerOverflow {
                loc: resource.offset(),
                // TODO(gj): better error message, e.g.:
                //               duplicate namespace declaration: Org { ... } defined on line XX of file YY
                //                                                previously defined on line AA of file BB
                token: format!("duplicate declaration of {} namespace", resource),
            }
            .into());
        }

        let declarations = transform_declarations(roles, permissions);
        self.resource_namespaces
            .insert(resource.clone(), declarations.clone());

        // TODO(gj): what to do for `on "parent_org"` if Org{} namespace hasn't
        // been processed yet? Whether w/ multiple load_file calls or some future
        // `import` feature, we probably don't want to force a specific load order
        // on folks if we don't have to. Maybe add as-of-yet uncheckable
        // implications into a queue that we check once all files are loaded /
        // imported? That might work for the future import case, but how would we
        // know when the final load_file call has been made? Answer: hax.

        if let Some(implications) = implications {
            for (implied, implier) in implications {
                let rule =
                    rewrite_implication(implied, implier, resource.clone(), declarations.clone());
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
    fn test_resource_namespace_rewrite_implications() {
        let roles = vec![term!("owner"), term!("member")];
        let permissions = vec![term!("invite"), term!("create_repo")];
        let declarations = transform_declarations(Some(roles), Some(permissions));
        let rewritten_role_role = rewrite_implication(
            term!("member"),
            term!("owner"),
            term!(sym!("Org")),
            declarations.clone(),
        );
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"role(actor, "member", org: Org{}) if role(actor, "owner", org);"#
        );

        let rewritten_permission_role = rewrite_implication(
            term!("invite"),
            term!("owner"),
            term!(sym!("Org")),
            declarations.clone(),
        );
        assert_eq!(
            rewritten_permission_role.to_polar(),
            r#"permission(actor, "invite", org: Org{}) if role(actor, "owner", org);"#
        );

        let rewritten_permission_permission = rewrite_implication(
            term!("create_repo"),
            term!("invite"),
            term!(sym!("Org")),
            declarations,
        );
        assert_eq!(
            rewritten_permission_permission.to_polar(),
            r#"permission(actor, "create_repo", org: Org{}) if permission(actor, "invite", org);"#
        );
    }
}
