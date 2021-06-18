#![allow(dead_code)]
use super::bindings::Bindings;
use super::error::{PolarResult, ValidationError};
use super::terms::*;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ResultEvent {
    bindings: Bindings,
}

struct Action {
    typ: String,
    name: String,
}

struct Role {
    name: String,
    typ: String,
    actions: Vec<String>,
    implied_roles: Vec<String>,
}

struct Resource {
    typ: String,
    name: String,
    actions: Vec<String>,
    roles: HashMap<String, Role>,
}

pub const VALIDATE_ROLES_CONFIG_RESOURCES: &str = "resource(resource, name, actions, roles)";

pub fn validate_roles_config(validation_query_results: &str) -> PolarResult<()> {
    let roles_config: Vec<Vec<ResultEvent>> = serde_json::from_str(&validation_query_results)
        .map_err(|_| ValidationError("Invalid config query result".to_string()))?;

    let role_resources = roles_config
        .first()
        .ok_or_else(|| ValidationError("Need to define resources to use oso roles.".to_owned()))?;
    if role_resources.is_empty() {
        return Err(
            ValidationError("Need to define resources to use oso roles.".to_owned()).into(),
        );
    }

    let mut resources = HashMap::new();
    for result in role_resources {
        let resource_def = result
            .bindings
            .get(&Symbol::new("resource"))
            .unwrap()
            .value();
        let resource_name = result.bindings.get(&Symbol::new("name")).unwrap().value();
        let resource_actions = result
            .bindings
            .get(&Symbol::new("actions"))
            .unwrap()
            .value();
        let resource_roles = result.bindings.get(&Symbol::new("roles")).unwrap().value();

        let typ = {
            if let Value::Expression(Operation {
                operator: Operator::And,
                args: and_args,
            }) = resource_def
            {
                match &and_args[..] {
                    [arg] => {
                        if let Value::Expression(Operation {
                            operator: Operator::Isa,
                            args: isa_args,
                        }) = arg.value()
                        {
                            match &isa_args[..] {
                                [this_expr, typ_expr] => {
                                    if let Value::Variable(Symbol(sym)) = this_expr.value() {
                                        if sym != "_this" {
                                            return Err(ValidationError(
                                                "Invalid resource, no type specializer.".to_owned(),
                                            )
                                            .into());
                                        }
                                    } else {
                                        return Err(ValidationError(
                                            "Invalid resource, no type specializer.".to_owned(),
                                        )
                                        .into());
                                    }
                                    if let Value::Pattern(Pattern::Instance(InstanceLiteral {
                                        tag,
                                        ..
                                    })) = typ_expr.value()
                                    {
                                        tag.0.clone()
                                    } else {
                                        return Err(ValidationError(
                                            "Invalid resource, no type specializer.".to_owned(),
                                        )
                                        .into());
                                    }
                                }
                                _ => {
                                    return Err(ValidationError(
                                        "Invalid resource, no type specializer.".to_owned(),
                                    )
                                    .into())
                                }
                            }
                        } else {
                            return Err(ValidationError(
                                "Invalid resource, no type specializer.".to_owned(),
                            )
                            .into());
                        }
                    }
                    _ => {
                        return Err(ValidationError(
                            "Invalid resource, no type specializer.".to_owned(),
                        )
                        .into())
                    }
                }
            } else {
                return Err(
                    ValidationError("Invalid resource, no type specializer.".to_owned()).into(),
                );
            }
        };

        let name = {
            if let Value::String(name) = resource_name {
                name.clone()
            } else {
                return Err(
                    ValidationError("Invalid resource, name is not a string.".to_owned()).into(),
                );
            }
        };

        let actions: Vec<String> = {
            let mut action_strings = vec![];
            match resource_actions {
                Value::List(actions) => {
                    for a in actions {
                        if let Value::String(action) = a.value() {
                            action_strings.push(action.clone());
                        } else {
                            return Err(ValidationError(
                                "Invalid action, not a string.".to_owned(),
                            )
                            .into());
                        }
                    }
                }
                Value::Variable(_) => (),
                _ => return Err(ValidationError("Invalid actions.".to_owned()).into()),
            }
            action_strings
        };

        let mut acts = HashSet::new();
        for action in &actions {
            if acts.contains(action) {
                return Err(
                    ValidationError(format!("Duplicate action {} for {}.", action, typ)).into(),
                );
            }
            acts.insert(action.to_owned());
        }

        let mut role_definitions = HashMap::new();
        if let Value::Dictionary(Dictionary { fields: dict }) = resource_roles {
            for (name_sym, definition) in dict.iter() {
                let role_name = name_sym.0.clone();
                if let Value::Dictionary(Dictionary { fields: def_dict }) = definition.value() {
                    for key in def_dict.keys() {
                        if key.0 != "permissions" && key.0 != "implies" {
                            return Err(ValidationError(format!(
                                "Invalid key for role definition {}",
                                key.0
                            ))
                            .into());
                        }
                    }
                    let actions = {
                        let actions_value = def_dict.get(&Symbol::new("permissions"));
                        if let Some(actions_term) = actions_value {
                            if let Value::List(actions_list) = actions_term.value() {
                                let mut actions = vec![];
                                for action_term in actions_list {
                                    if let Value::String(action) = action_term.value() {
                                        actions.push(action.clone())
                                    } else {
                                        return Err(ValidationError(format!(
                                            "Invalid actions for role {}, must be a string.",
                                            role_name
                                        ))
                                        .into());
                                    }
                                }
                                actions
                            } else {
                                return Err(ValidationError(format!(
                                    "Invalid actions for role {}",
                                    role_name
                                ))
                                .into());
                            }
                        } else {
                            vec![]
                        }
                    };
                    let implications = {
                        let implications_value = def_dict.get(&Symbol::new("implies"));
                        if let Some(implications_term) = implications_value {
                            if let Value::List(implications_list) = implications_term.value() {
                                let mut implications = vec![];
                                for implies_term in implications_list {
                                    if let Value::String(implies) = implies_term.value() {
                                        implications.push(implies.clone())
                                    } else {
                                        return Err(ValidationError(format!(
                                            "Invalid implies for role {}, must be a string.",
                                            role_name
                                        ))
                                        .into());
                                    }
                                }
                                implications
                            } else {
                                return Err(ValidationError(format!(
                                    "Invalid implies for role {}",
                                    role_name
                                ))
                                .into());
                            }
                        } else {
                            vec![]
                        }
                    };
                    if actions.is_empty() && implications.is_empty() {
                        return Err(ValidationError(
                            "Must define actions or implications for a role.".to_owned(),
                        )
                        .into());
                    }
                    let role = Role {
                        name: role_name.clone(),
                        typ: typ.clone(),
                        actions,
                        implied_roles: implications,
                    };
                    if role_definitions.contains_key(&role_name) {
                        return Err(
                            ValidationError(format!("Duplicate role name {}.", role_name)).into(),
                        );
                    }
                    role_definitions.insert(role_name, role)
                } else {
                    return Err(ValidationError("Invalid role definitions".to_owned()).into());
                };
            }
        }

        if actions.is_empty() && role_definitions.is_empty() {
            return Err(ValidationError("Must define actions or roles.".to_owned()).into());
        }

        let resource = Resource {
            typ: typ.clone(),
            name: name.clone(),
            actions,
            roles: role_definitions,
        };
        if resources.contains_key(&name) {
            return Err(ValidationError(format!("Duplicate resource name {}.", name)).into());
        }
        resources.insert(name, resource);
    }

    Ok(())
}
