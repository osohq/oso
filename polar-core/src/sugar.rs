use std::collections::{HashMap, HashSet};

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

// This type is used as a pre-validation bridge between Rust & LALRPOP.
#[derive(Debug)]
pub enum Expr {
    Roles(Term),                             // List<String>
    Permissions(Term),                       // List<String>
    Relations(Term),                         // Dict<Symbol, Symbol>
    Implication(Term, (Term, Option<Term>)), // (String, (String, Option<String>))
}

pub fn declaration_to_expr(
    (name, term): (Symbol, Term),
) -> Result<Expr, LalrpopError<usize, Token, error::ParseError>> {
    match (name.0.as_ref(), term.value()) {
        ("roles", Value::List(_)) => Ok(Expr::Roles(term)),
        ("permissions", Value::List(_)) => Ok(Expr::Permissions(term)),
        ("relations", Value::Dictionary(_)) => Ok(Expr::Relations(term)),

        ("roles", Value::Dictionary(_)) | ("permissions", Value::Dictionary(_)) => {
            Err(LalrpopError::User {
                error: ParseError::ParseSugar {
                    loc: term.offset(),
                    msg: format!(
                        "Expected '{}' declaration to be a list of strings; found a dictionary:",
                        name
                    ),
                    ranges: vec![term.span().unwrap()],
                },
            })
        }
        ("relations", Value::List(_)) => Err(LalrpopError::User {
            error: ParseError::ParseSugar {
                loc: term.offset(),
                msg: "Expected 'relations' declaration to be a dictionary; found a list:".to_owned(),
                ranges: vec![term.span().unwrap()],
            },
        }),

        (_, Value::List(_)) => Err(LalrpopError::User {
            error: ParseError::ParseSugar {
                loc: term.offset(),
                msg: format!(
                    "Encountered unexpected declaration '{}'. Did you mean for this to be 'roles = [ ... ];' or 'permissions = [ ... ];'?", name
                ),
                ranges: vec![term.span().unwrap()],
            },
        }),
        (_, Value::Dictionary(_)) => Err(LalrpopError::User {
            error: ParseError::ParseSugar {
                loc: term.offset(),
                msg: format!(
                    "Encountered unexpected declaration '{}'. Did you mean for this to be 'relations = {{ ... }};'?", name
                ),
                ranges: vec![term.span().unwrap()],
            },
        }),
        _ => unreachable!(),
    }
}

// Turn a set of parsed expressions into a `Namespace` (or die validating).
pub fn exprs_to_namespace(
    resource: Term,
    exprs: Vec<Expr>,
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

    for expr in exprs {
        match expr {
            Expr::Roles(new) => {
                if let Some(previous) = roles {
                    let error = make_error("roles", &previous, &new);
                    return Err(LalrpopError::User { error });
                }
                roles = Some(new);
            }
            Expr::Permissions(new) => {
                if let Some(previous) = permissions {
                    let error = make_error("permissions", &previous, &new);
                    return Err(LalrpopError::User { error });
                }
                permissions = Some(new);
            }
            Expr::Relations(new) => {
                if let Some(previous) = relations {
                    let error = make_error("relations", &previous, &new);
                    return Err(LalrpopError::User { error });
                }
                relations = Some(new);
            }
            Expr::Implication(head, body) => {
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
    Relation(Term),
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct Implication {
    pub head: Term,
    pub body: (Term, Option<Term>),
}

impl Eq for Implication {}

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
            Declaration::Role => sym!("role"),
            Declaration::Permission => sym!("permission"),
            Declaration::Relation(_) => sym!("relation"),
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
        let mut rules = vec![];
        let mut errors = vec![];
        for (resource, implications) in self.rewrite_me_pls.drain() {
            for implication in implications {
                match implication.into_rule(&resource, &self.namespaces) {
                    Ok(rule) => rules.push(rule),
                    Err(error) => errors.push(error),
                }
            }
        }

        // Copy the existing rules in the KB in case we encounter an error and need to revert.
        let existing = self.get_rules().clone();

        // Add the rewritten rules to the KB.
        for rule in rules {
            self.add_rule(rule);
        }

        errors.append(&mut check_exhaustiveness_for_declarations(self));

        // TODO(gj): Emit all errors instead of just the first.
        if !errors.is_empty() {
            self.set_rules(existing);
            return Err(errors[0].clone());
        }

        Ok(())
    }
}

fn rule_name_is_cool(n: &Symbol) -> bool {
    n == &sym!("role") || n == &sym!("permission") || n == &sym!("relation")
}

// TODO(gj): how do bodiless rules factor into exhaustiveness? E.g., the only reference to the
// "foo" role is in `has_role(_: Actor, "foo", _: Org);`. I guess that's fine; it's a bit funky,
// but it's saying that all Actors have the "foo" role on all Orgs.
fn check_exhaustiveness_for_declarations(kb: &KnowledgeBase) -> Vec<PolarError> {
    let mut errors = vec![];

    let mut x = HashSet::new();
    for (resource, declarations) in &kb.namespaces.declarations {
        for (declaration, kind) in declarations {
            x.insert((
                kind.as_predicate(),
                declaration.clone(),
                Some(resource.clone_with_value(value!(pattern!(instance!(
                    &resource.value().as_symbol().unwrap().0
                ))))),
            ));
        }
    }

    for generic_rule in kb.get_rules().values() {
        for rule in generic_rule.rules.values() {
            eprintln!("Checking: {}", rule.to_polar());
            let Rule {
                name, params, body, ..
            } = rule.as_ref();
            eprintln!("  [HEAD] name = {}", name);
            if rule_name_is_cool(name) {
                // TODO(gj): Is this length check obviated by rule prototypes? Are 'built-in' rule
                // prototypes extendable by users?
                eprintln!("  [HEAD] params.len() = {}", params.len());
                if params.len() == 3 {
                    eprintln!(
                        "  [HEAD] ({}, {}, {:?})",
                        name,
                        params[1].parameter,
                        params[2].specializer.as_ref().map(|x| x.to_polar())
                    );
                    let removed = x.remove(&(
                        name.clone(),
                        params[1].parameter.clone(),
                        params[2].specializer.clone(),
                    ));
                    if removed {
                        eprintln!("\tREMOVED via HEAD check");
                    }
                }
            }
            for clause in &body.value().as_expression().unwrap().args {
                // TODO(gj): Don't think I need to worry about kwargs being non-empty... right?
                if let Ok(Call { name, args, .. }) = clause.value().as_call() {
                    if rule_name_is_cool(name) {
                        // TODO(gj): same question about checking length as above.
                        if args.len() == 3 {
                            let removed = x.remove(&(
                                name.clone(),
                                args[1].clone(),
                                // TODO(gj): how to check the arg against the resource specializer?
                                // Do I need to query this rule with unbounds and then check the
                                // constraints for it?
                                params[2].specializer.clone(),
                            ));
                            if removed {
                                eprintln!("\tREMOVED via BODY check");
                            }
                        }
                    }
                }
            }
        }
    }

    for (rule_name, declaration, resource) in x {
        let resource = match resource.as_ref().unwrap().value().as_pattern().unwrap() {
            Pattern::Instance(InstanceLiteral { tag, .. }) => tag,
            _ => unreachable!(),
        };
        errors.push(
            ParseError::ParseSugar {
                loc: declaration.offset(),
                msg: format!(
                    "{}: {} {} declared but never referenced.",
                    resource,
                    rule_name,
                    declaration.to_polar(),
                ),
                ranges: vec![],
            }
            .into(),
        );
    }

    errors
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
        let related_resource_var = relation.clone_with_value(resource_as_var(related_type));

        let relation_call = relation.clone_with_value(value!(Call {
            name: sym!("relation"),
            // For example: vec![org, "parent", repo]
            args: vec![related_resource_var.clone(), relation.clone(), resource_var],
            kwargs: None
        }));

        let implier_call = implier.clone_with_value(value!(Call {
            name: namespaces.cross_resource_predicate_name(&implier, &relation, resource)?,
            // For example: vec![actor, "owner", org]
            args: vec![actor_var, implier.clone(), related_resource_var],
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
fn check_that_namespace_resource_is_registered(
    kb: &KnowledgeBase,
    resource: &Term,
) -> PolarResult<()> {
    if !kb.is_constant(resource.value().as_symbol()?) {
        // TODO(gj): UnregisteredClassError in the core.
        return Err(ParseError::ParseSugar {
            loc: resource.offset(),
            // TODO(gj): better error message
            msg: format!(
                "{} namespace must be registered as a class",
                resource.to_polar()
            ),
            ranges: vec![],
        }
        .into());
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

        let declarations = index_declarations(roles, permissions, relations);

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

        // TODO(gj): what to do for `on "parent_org"` if Org{} namespace hasn't
        // been processed yet? Whether w/ multiple load_file calls or some future
        // `import` feature, we probably don't want to force a specific load order
        // on folks if we don't have to. Maybe add as-of-yet uncheckable
        // implications into a queue that we check once all files are loaded /
        // imported? That might work for the future import case, but how would we
        // know when the final load_file call has been made? Answer: hax.

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
        let repo_roles = term!(["reader"]);
        let repo_relations = term!(btreemap! { sym!("parent") => term!(sym!("org")) });
        let repo_declarations = index_declarations(Some(repo_roles), None, Some(repo_relations));

        let org_roles = term!(["member"]);
        let org_declarations = index_declarations(Some(org_roles), None, None);

        let mut namespaces = Namespaces::new();
        namespaces.add(term!(sym!("repo")), repo_declarations);
        namespaces.add(term!(sym!("org")), org_declarations);
        let implication = Implication {
            head: term!("reader"),
            body: (term!("member"), Some(term!("parent"))),
        };
        let rewritten_role_role = implication
            .into_rule(&term!(sym!("repo")), &namespaces)
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
        let implication = Implication {
            head: term!("member"),
            body: (term!("owner"), None),
        };
        let rewritten_role_role = implication
            .into_rule(&term!(sym!("Org")), &namespaces)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"role(actor, "member", org: Org{}) if role(actor, "owner", org);"#
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
            r#"permission(actor, "invite", org: Org{}) if role(actor, "owner", org);"#
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
        let implication = Implication {
            head: term!("reader"),
            body: (term!("member"), Some(term!("parent"))),
        };
        let rewritten_role_role = implication
            .into_rule(&term!(sym!("Repo")), &namespaces)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"role(actor, "reader", repo: Repo{}) if relation(org, "parent", repo) and role(actor, "member", org);"#
        );
    }

    #[test]
    fn test_namespace_must_be_registered() {
        let p = Polar::new();
        let valid_policy = "Org{}";
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
        let invalid_policy = "Org{}Org{}";
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(&p, invalid_policy, "duplicate declaration of Org namespace");
    }

    #[test]
    fn test_namespace_permission_exhaustiveness_checks() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));
        let invalid_policy = r#"
            Org { permissions=["invite","create_repo","ban"]; }
            permission(actor, "invite", org: Org) if permission(actor, "ban", org);"#;
        // TODO(gj): can we ever actually know this? What if someone wrote has_permission/3 with a
        // variable for the second argument? Maybe this will be disallowed by rule prototypes if we
        // specialize the second argument as a String (and as a `T::Permission where T: Resource`
        // in the future)?
        let expected = r#"Org: permission "create_repo" declared but never referenced."#;
        expect_error(&p, invalid_policy, expected);
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
            r#"Encountered unexpected declaration 'foo'. Did you mean for this to be 'roles = [ ... ];' or 'permissions = [ ... ];'?"#,
        );
        expect_error(
            &p,
            r#"Org{foo={};}"#,
            r#"Encountered unexpected declaration 'foo'. Did you mean for this to be 'relations = { ... };'?"#,
        );
    }

    #[test]
    fn test_namespace_declaration_keywords_are_not_reserved_words() {
        let p = Polar::new();
        p.load_str("roles(permissions) if permissions.relations;")
            .unwrap();
    }
}
