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
    roles: Option<Vec<String>>,
    permissions: Option<Vec<String>>,
) -> HashMap<String, Declaration> {
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
    implied: String,
    implier: String,
    resource: Symbol,
    declarations: HashMap<String, Declaration>,
) -> Rule {
    let actor_var = term!(sym!("actor"));
    let resource_var = term!(sym!(resource.0.to_lowercase()));
    let body_call = match declarations[&implier] {
        Declaration::Role => sym!("role"),
        Declaration::Permission => sym!("permission"),
    };
    // TODO(gj): loc info
    let body = Term::new_from_parser(
        0,
        0,
        0,
        value!(op!(
            And,
            Term::new_from_parser(
                0,
                0,
                0,
                value!(Call {
                    name: body_call,
                    args: vec![
                        actor_var.clone(),
                        term!(implier.as_str()),
                        resource_var.clone(),
                    ],
                    kwargs: None
                })
            )
        )),
    );

    let rule_name = match declarations[&implied] {
        Declaration::Role => sym!("role"),
        Declaration::Permission => sym!("permission"),
    };
    let params = vec![
        Parameter {
            parameter: actor_var,
            specializer: None,
        },
        Parameter {
            parameter: term!(implied.as_str()),
            specializer: None,
        },
        Parameter {
            parameter: resource_var,
            specializer: Some(term!(resource)),
        },
    ];
    Rule::new_from_parser(0, 0, 0, rule_name, params, body)
}

impl KnowledgeBase {
    pub fn add_resource_namespace(&mut self, namespace: ResourceNamespace) -> PolarResult<()> {
        let ResourceNamespace {
            name,
            roles,
            permissions,
            implications,
        } = namespace;

        // TODO(gj): no way to know in the core if `name` was registered as a class
        // or a constant.
        if !self.is_constant(&name) {
            return Err(ParseError::IntegerOverflow {
                loc: 0, // TODO(gj): loc info
                // TODO(gj): better error message
                token: format!("namespace {} must be registered as a class", name),
            }
            .into());
        }

        // Check for duplicate resource namespace definitions.
        if self.resource_namespaces.contains_key(&name) {
            return Err(ParseError::IntegerOverflow {
                loc: 0, // TODO(gj): loc info
                // TODO(gj): better error message, e.g.:
                //               duplicate namespace declaration: Org { ... } defined on line XX of file YY
                //                                                previously defined on line AA of file BB
                token: format!("duplicate declaration of {} namespace", name),
            }
            .into());
        }

        let declarations = transform_declarations(roles, permissions);
        self.resource_namespaces
            .insert(name.clone(), declarations.clone());

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
                    rewrite_implication(implied, implier, name.clone(), declarations.clone());
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
        let roles = vec!["owner".to_owned(), "member".to_owned()];
        let permissions = vec!["invite".to_owned(), "create_repo".to_owned()];
        let declarations = transform_declarations(Some(roles), Some(permissions));
        let rewritten_role_role = rewrite_implication(
            "member".to_owned(),
            "owner".to_owned(),
            sym!("Org"),
            declarations.clone(),
        );
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"role(actor, "member", org: Org) if role(actor, "owner", org);"#
        );

        let rewritten_permission_role = rewrite_implication(
            "invite".to_owned(),
            "owner".to_owned(),
            sym!("Org"),
            declarations.clone(),
        );
        assert_eq!(
            rewritten_permission_role.to_polar(),
            r#"permission(actor, "invite", org: Org) if role(actor, "owner", org);"#
        );

        let rewritten_permission_permission = rewrite_implication(
            "create_repo".to_owned(),
            "invite".to_owned(),
            sym!("Org"),
            declarations,
        );
        assert_eq!(
            rewritten_permission_permission.to_polar(),
            r#"permission(actor, "create_repo", org: Org) if permission(actor, "invite", org);"#
        );
    }
}
