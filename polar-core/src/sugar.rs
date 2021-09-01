use std::collections::{HashMap, HashSet};
use std::ops::Range;

use lalrpop_util::ParseError as LalrpopError;

use super::error::{ParseError, PolarError, PolarResult, RuntimeError};
use super::kb::KnowledgeBase;
use super::lexer::Token;
use super::rules::*;
use super::terms::*;

// TODO(gj): if a user imports the built-in rule prototypes, we should emit an error if the user
// hasn't registered at least a single Actor and Resource type by the time loading is complete.
// Maybe only if they've defined at least one rule matching one of the rule prototypes? Otherwise,
// the rule prototypes will always trigger. But maybe their error message will be descriptive
// enough as-is?

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
        let (loc, ranges) = (keyword.offset(), vec![]);
        let msg = format!(
            "Unexpected relation keyword '{}'. Did you mean 'on'?",
            keyword
        );
        Err(LalrpopError::User {
            error: ParseError::ParseSugar { loc, msg, ranges },
        })
    }
}

// TODO(gj): Create a Parsed<Term> or something that _always_ has source info.
fn term_source_range(term: &Term) -> Range<usize> {
    let (start, end) = term.span().unwrap();
    start..end
}

pub fn validate_parsed_declaration(
    (name, term): (Symbol, Term),
) -> Result<Production, LalrpopError<usize, Token, error::ParseError>> {
    match (name.0.as_ref(), term.value()) {
        ("roles", Value::List(_)) => Ok(Production::Roles(term)),
        ("permissions", Value::List(_)) => Ok(Production::Permissions(term)),
        ("relations", Value::Dictionary(_)) => Ok(Production::Relations(term)),

        ("roles", Value::Dictionary(_)) | ("permissions", Value::Dictionary(_)) => {
            let (loc, ranges) = (term.offset(), vec![term_source_range(&term)]);
            let msg = format!("Expected '{}' declaration to be a list of strings; found a dictionary:\n", name);
            Err(LalrpopError::User { error: ParseError::ParseSugar { loc, msg, ranges } })
        }
        ("relations", Value::List(_)) => Err(LalrpopError::User {
            error: ParseError::ParseSugar {
                loc: term.offset(),
                msg: "Expected 'relations' declaration to be a dictionary; found a list:\n".to_owned(),
                ranges: vec![term_source_range(&term)],
            },
        }),

        (_, Value::List(_)) => Err(LalrpopError::User {
            error: ParseError::ParseSugar {
                loc: term.offset(),
                msg: format!(
                    "Unexpected declaration '{}'. Did you mean for this to be 'roles = [ ... ];' or 'permissions = [ ... ];'?\n", name
                ),
                ranges: vec![term_source_range(&term)],
            },
        }),
        (_, Value::Dictionary(_)) => Err(LalrpopError::User {
            error: ParseError::ParseSugar {
                loc: term.offset(),
                msg: format!(
                    "Unexpected declaration '{}'. Did you mean for this to be 'relations = {{ ... }};'?\n", name
                ),
                ranges: vec![term_source_range(&term)],
            },
        }),
        _ => unreachable!(),
    }
}

pub fn turn_productions_into_namespace(
    keyword: Option<Term>,
    resource: Term,
    productions: Vec<Production>,
) -> Result<Namespace, LalrpopError<usize, Token, error::ParseError>> {
    if let Some(keyword) = keyword {
        let entity_type = match keyword.value().as_symbol().unwrap().0.as_ref() {
            "actor" => EntityType::Actor,
            "resource" => EntityType::Resource,
            _ => {
                let (loc, ranges) = (keyword.offset(), vec![]);
                let msg = format!(
                    "Expected 'actor' or 'resource' but found '{}'.",
                    keyword.to_polar()
                );
                let error = ParseError::ParseSugar { loc, msg, ranges };
                return Err(LalrpopError::User { error });
            }
        };

        let mut roles: Option<Term> = None;
        let mut permissions: Option<Term> = None;
        let mut relations: Option<Term> = None;
        let mut implications = vec![];

        let make_error = |name: &str, previous: &Term, new: &Term| {
            let loc = new.offset();
            let ranges = vec![term_source_range(previous), term_source_range(new)];
            let msg = format!(
                "Multiple '{}' declarations in {} namespace.\n",
                name,
                resource.to_polar()
            );
            ParseError::ParseSugar { loc, msg, ranges }
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
            entity_type,
            resource,
            roles,
            permissions,
            relations,
            implications,
        })
    } else {
        let (loc, ranges) = (resource.offset(), vec![]);
        let msg = "Expected 'actor' or 'resource' but found nothing.".to_owned();
        let error = ParseError::ParseSugar { loc, msg, ranges };
        Err(LalrpopError::User { error })
    }
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
    pub fn as_rule(&self, namespace: &Term, namespaces: &Namespaces) -> PolarResult<Rule> {
        let Self { head, body } = self;
        // Copy SourceInfo from head of implication.
        // TODO(gj): assert these can only be None in tests.
        let src_id = head.get_source_id().unwrap_or(0);
        let (start, end) = head.span().unwrap_or((0, 0));

        let name = namespaces.get_rule_name_for_declaration_in_namespace(head, namespace)?;
        let params = implication_head_to_params(head, namespace);
        let body = implication_body_to_rule_body(body, namespace, namespaces)?;

        Ok(Rule::new_from_parser(
            src_id, start, end, name, params, body,
        ))
    }
}

type Declarations = HashMap<Term, Declaration>;

impl Declaration {
    fn as_relation_type(&self) -> PolarResult<&Term> {
        if let Declaration::Relation(relation) = self {
            Ok(relation)
        } else {
            Err(RuntimeError::TypeError {
                msg: format!("Expected Relation; got: {:?}", self),
                stack_trace: None,
            }
            .into())
        }
    }

    fn as_rule_name(&self) -> Symbol {
        match self {
            Declaration::Role => sym!("has_role"),
            Declaration::Permission => sym!("has_permission"),
            Declaration::Relation(_) => sym!("has_relation"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EntityType {
    Actor,
    Resource,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Namespace {
    pub entity_type: EntityType,
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
    pub implications: HashMap<Term, Vec<Implication>>,
    pub actors: HashSet<Term>,
    pub resources: HashSet<Term>,
}

impl Namespaces {
    pub fn new() -> Self {
        Self {
            declarations: HashMap::new(),
            implications: HashMap::new(),
            actors: HashSet::new(),
            resources: HashSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.declarations.clear();
        self.implications.clear();
        self.actors.clear();
        self.resources.clear();
    }

    fn add(
        &mut self,
        entity_type: EntityType,
        resource: Term,
        declarations: Declarations,
        implications: Vec<Implication>,
    ) {
        self.declarations.insert(resource.clone(), declarations);
        self.implications.insert(resource.clone(), implications);
        match entity_type {
            EntityType::Actor => self.actors.insert(resource),
            EntityType::Resource => self.resources.insert(resource),
        };
    }

    fn exists(&self, resource: &Term) -> bool {
        self.declarations.contains_key(resource)
    }

    /// Look up `declaration` in `namespace`.
    ///
    /// Invariant: `namespace` _must_ exist.
    fn get_declaration_in_namespace(
        &self,
        declaration: &Term,
        namespace: &Term,
    ) -> PolarResult<&Declaration> {
        if let Some(declaration) = self.declarations[namespace].get(declaration) {
            Ok(declaration)
        } else {
            let (loc, ranges) = (declaration.offset(), vec![]);
            let msg = format!("Undeclared term {} referenced in implication in {} namespace. Did you mean to declare it as a role, permission, or relation?", declaration.to_polar(), namespace);
            Err(ParseError::ParseSugar { loc, msg, ranges }.into())
        }
    }

    /// Look up `relation` in `namespace` and return its type.
    fn get_relation_type_in_namespace(
        &self,
        relation: &Term,
        namespace: &Term,
    ) -> PolarResult<&Term> {
        self.get_declaration_in_namespace(relation, namespace)?
            .as_relation_type()
    }

    /// Look up `declaration` in `namespace` and return the appropriate rule name for rewriting.
    fn get_rule_name_for_declaration_in_namespace(
        &self,
        declaration: &Term,
        namespace: &Term,
    ) -> PolarResult<Symbol> {
        Ok(self
            .get_declaration_in_namespace(declaration, namespace)?
            .as_rule_name())
    }

    /// Traverse from `namespace` to a related namespace via `relation`, then look up `declaration`
    /// in the related namespace and return the appropriate rule name for rewriting.
    fn get_rule_name_for_declaration_in_related_namespace(
        &self,
        declaration: &Term,
        relation: &Term,
        namespace: &Term,
    ) -> PolarResult<Symbol> {
        let related_namespace = self.get_relation_type_in_namespace(relation, namespace)?;

        if let Some(declarations) = self.declarations.get(related_namespace) {
            if let Some(declaration) = declarations.get(declaration) {
                Ok(declaration.as_rule_name())
            } else {
                let (loc, ranges) = (declaration.offset(), vec![]);
                let msg = format!("{}: Term {} not declared on related resource {}. Did you mean to declare it as a role, permission, or relation on resource {}?", namespace.to_polar(), declaration.to_polar(), related_namespace.to_polar(), related_namespace.to_polar());
                Err(ParseError::ParseSugar { loc, msg, ranges }.into())
            }
        } else {
            let (loc, ranges) = (related_namespace.offset(), vec![]);
            let msg = format!("{}: Relation {} in implication body `{} on {}` has type {}, but no such namespace exists. Try declaring one: `resource {} {{}}`", namespace.to_polar(), relation.to_polar(), declaration.to_polar(), relation.to_polar(), related_namespace.to_polar(), related_namespace.to_polar());
            Err(ParseError::ParseSugar { loc, msg, ranges }.into())
        }
    }
}

pub fn check_all_relation_types_have_been_registered(kb: &KnowledgeBase) -> Vec<PolarError> {
    let mut errors = vec![];
    for declarations in kb.namespaces.declarations.values() {
        for (declaration, kind) in declarations {
            if let Declaration::Relation(relation_type) = kind {
                errors.extend(relation_type_is_registered(kb, (declaration, relation_type)).err());
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
                let (loc, ranges) = (role.offset(), vec![]);
                let msg = format!(
                    "{}: Duplicate declaration of {} in the roles list.",
                    resource.to_polar(),
                    role.to_polar()
                );
                return Err(ParseError::ParseSugar { loc, msg, ranges }.into());
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
                let (loc, ranges) = (permission.offset(), vec![]);
                return Err(ParseError::ParseSugar { loc, msg, ranges }.into());
            }
        }
    }

    if let Some(relations) = relations {
        for (relation, relation_type) in &relations.value().as_dict()?.fields {
            // Stringify relation so that we can index into the declarations map with a string
            // reference to the relation. E.g., relation `creator: User` gets stored as `"creator"
            // => Relation(User)` so that when we encounter an implication `"admin" if "creator";`
            // we can easily look up what type of declaration `"creator"` is.
            let stringified_relation = relation_type.clone_with_value(value!(relation.0.as_str()));
            let declaration = Declaration::Relation(relation_type.clone());
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
                let (loc, ranges) = (relation_type.offset(), vec![]);
                return Err(ParseError::ParseSugar { loc, msg, ranges }.into());
            }
        }
    }
    Ok(declarations)
}

fn namespace_as_var(namespace: &Term) -> Value {
    let name = &namespace.value().as_symbol().expect("sym").0;
    let mut lowercased = name.to_lowercase();

    // If the namespace's name is already lowercase, append "_instance" to distinguish the variable
    // name from the namespace's name.
    if &lowercased == name {
        lowercased += "_instance";
    }

    value!(sym!(lowercased))
}

/// Turn an implication body into an And-wrapped call (for a local implication) or pair of calls
/// (for a cross-resource implication).
fn implication_body_to_rule_body(
    (implier, relation): &(Term, Option<Term>),
    namespace: &Term,
    namespaces: &Namespaces,
) -> PolarResult<Term> {
    // Create a variable derived from the name of the current namespace. E.g., if we're in the
    // `Repo` namespace, the variable name will be `repo`.
    let namespace_var = implier.clone_with_value(namespace_as_var(namespace));

    // The actor variable will always be named `actor`.
    let actor_var = implier.clone_with_value(value!(sym!("actor")));

    // If there's a relation, e.g., `if <implier> on <relation>`...
    if let Some(relation) = relation {
        // TODO(gj): what if the relation is with the same type? E.g.,
        // `Dir { relations = { parent: Dir }; }`. This might cause Polar to loop.

        // ...then we need to link the rewritten `<implier>` and `<relation>` rules via a shared
        // variable. To be clever, we'll name the variable according to the type of the relation,
        // e.g., if the declared relation is `parent: Org` we'll name the variable `org`.
        let relation_type = namespaces.get_relation_type_in_namespace(relation, namespace)?;
        let relation_type_var = relation.clone_with_value(namespace_as_var(relation_type));

        // For the rewritten `<relation>` call, the rule name will always be `has_relation` and the
        // arguments, in order, will be: the shared variable we just created above, the
        // `<relation>` string, and the namespace variable we created at the top of the function.
        // E.g., `vec![org, "parent", repo]`.
        let relation_call = relation.clone_with_value(value!(Call {
            name: sym!("has_relation"),
            args: vec![relation_type_var.clone(), relation.clone(), namespace_var],
            kwargs: None
        }));

        // To get the rule name for the rewritten `<implier>` call, we need to figure out what
        // type (role, permission, or relation) `<implier>` is declared as _in the namespace
        // related to the current namespace via `<relation>`_. That is, given
        // `resource Repo { roles=["writer"]; relations={parent:Org}; "writer" if "owner" on "parent"; }`,
        // we need to find out whether `"owner"` is declared as a role, permission, or relation in
        // the `Org` namespace. The args for the rewritten `<implier>` call are, in order: the
        // actor variable, the `<implier>` string, and the shared variable we created above for the
        // related type.
        let implier_call = implier.clone_with_value(value!(Call {
            name: namespaces
                .get_rule_name_for_declaration_in_related_namespace(implier, relation, namespace)?,
            args: vec![actor_var, implier.clone(), relation_type_var],
            kwargs: None
        }));

        // Wrap the rewritten `<relation>` and `<implier>` calls in an `And`.
        Ok(implier.clone_with_value(value!(op!(And, relation_call, implier_call))))
    } else {
        // If there's no `<relation>` (e.g., `... if "writer";`), we're dealing with a local
        // implication, and the rewriting process is a bit simpler. To get the appropriate rule
        // name, we look up the declared type (role, permission, or relation) of `<implier>` in the
        // current namespace. The call's args are, in order: the actor variable, the `<implier>`
        // string, and the namespace variable. E.g., `vec![actor, "writer", repo]`.
        let implier_call = implier.clone_with_value(value!(Call {
            name: namespaces.get_rule_name_for_declaration_in_namespace(implier, namespace)?,
            args: vec![actor_var, implier.clone(), namespace_var],
            kwargs: None
        }));

        // Wrap the rewritten `<implier>` call in an `And`.
        Ok(implier.clone_with_value(value!(op!(And, implier_call))))
    }
}

/// Turn an implication head into a trio of params that go in the head of the rewritten rule.
fn implication_head_to_params(head: &Term, namespace: &Term) -> Vec<Parameter> {
    let namespace_name = &namespace.value().as_symbol().expect("sym").0;
    vec![
        Parameter {
            parameter: head.clone_with_value(value!(sym!("actor"))),
            specializer: Some(head.clone_with_value(value!(pattern!(instance!("Actor"))))),
        },
        Parameter {
            parameter: head.clone(),
            specializer: None,
        },
        Parameter {
            parameter: head.clone_with_value(namespace_as_var(namespace)),
            specializer: Some(
                namespace.clone_with_value(value!(pattern!(instance!(namespace_name)))),
            ),
        },
    ]
}

// TODO(gj): better error message, e.g.:
//               duplicate namespace declaration: resource Org { ... } defined on line XX of file YY
//                                                previously defined on line AA of file BB
fn check_for_duplicate_namespaces(namespaces: &Namespaces, namespace: &Term) -> PolarResult<()> {
    if namespaces.exists(namespace) {
        let (loc, ranges) = (namespace.offset(), vec![]);
        let msg = format!("Duplicate declaration of {} namespace.", namespace);
        return Err(ParseError::ParseSugar { loc, msg, ranges }.into());
    }
    Ok(())
}

// TODO(gj): no way to know in the core if `term` was registered as a class or a constant.
fn is_registered_class(kb: &KnowledgeBase, term: &Term) -> PolarResult<bool> {
    Ok(kb.is_constant(term.value().as_symbol()?))
}

fn check_that_namespace_resource_is_registered_as_a_class(
    kb: &KnowledgeBase,
    resource: &Term,
) -> PolarResult<()> {
    if !is_registered_class(kb, resource)? {
        // TODO(gj): better error message
        let msg = format!(
            "In order to be declared as a namespace, {} must be registered as a class.",
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
            let (loc, ranges) = (head.offset(), vec![]);
            let error = ParseError::ParseSugar { loc, msg, ranges };
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
        errors.extend(
            check_that_namespace_resource_is_registered_as_a_class(kb, &self.resource).err(),
        );
        errors.extend(check_for_duplicate_namespaces(&kb.namespaces, &self.resource).err());

        let Namespace {
            entity_type,
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

        kb.namespaces
            .add(entity_type, resource, declarations, implications);

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
        let msg = match p.load_str(policy).unwrap_err() {
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::ParseSugar { msg, .. }),
                ..
            } => msg,
            _ => panic!(),
        };

        assert!(msg.contains(expected));
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
        namespaces.add(
            EntityType::Resource,
            repo_resource,
            repo_declarations.unwrap(),
            vec![],
        );
        namespaces.add(
            EntityType::Resource,
            org_resource,
            org_declarations.unwrap(),
            vec![],
        );
        let implication = Implication {
            head: term!("reader"),
            body: (term!("member"), Some(term!("parent"))),
        };
        let rewritten_role_role = implication
            .as_rule(&term!(sym!("repo")), &namespaces)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"has_role(actor: Actor{}, "reader", repo_instance: repo{}) if has_relation(org_instance, "parent", repo_instance) and has_role(actor, "member", org_instance);"#
        );
    }

    #[test]
    fn test_namespace_local_rewrite_implications() {
        let resource = term!(sym!("Org"));
        let roles = term!(["owner", "member"]);
        let permissions = term!(["invite", "create_repo"]);
        let declarations = index_declarations(Some(roles), Some(permissions), None, &resource);
        let mut namespaces = Namespaces::new();
        namespaces.add(
            EntityType::Resource,
            resource,
            declarations.unwrap(),
            vec![],
        );
        let implication = Implication {
            head: term!("member"),
            body: (term!("owner"), None),
        };
        let rewritten_role_role = implication
            .as_rule(&term!(sym!("Org")), &namespaces)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"has_role(actor: Actor{}, "member", org: Org{}) if has_role(actor, "owner", org);"#
        );

        let implication = Implication {
            head: term!("invite"),
            body: (term!("owner"), None),
        };
        let rewritten_permission_role = implication
            .as_rule(&term!(sym!("Org")), &namespaces)
            .unwrap();
        assert_eq!(
            rewritten_permission_role.to_polar(),
            r#"has_permission(actor: Actor{}, "invite", org: Org{}) if has_role(actor, "owner", org);"#
        );

        let implication = Implication {
            head: term!("create_repo"),
            body: (term!("invite"), None),
        };
        let rewritten_permission_permission = implication
            .as_rule(&term!(sym!("Org")), &namespaces)
            .unwrap();
        assert_eq!(
            rewritten_permission_permission.to_polar(),
            r#"has_permission(actor: Actor{}, "create_repo", org: Org{}) if has_permission(actor, "invite", org);"#
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
        namespaces.add(
            EntityType::Resource,
            repo_resource,
            repo_declarations.unwrap(),
            vec![],
        );
        namespaces.add(
            EntityType::Resource,
            org_resource,
            org_declarations.unwrap(),
            vec![],
        );
        let implication = Implication {
            head: term!("reader"),
            body: (term!("member"), Some(term!("parent"))),
        };
        let rewritten_role_role = implication
            .as_rule(&term!(sym!("Repo")), &namespaces)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            r#"has_role(actor: Actor{}, "reader", repo: Repo{}) if has_relation(org, "parent", repo) and has_role(actor, "member", org);"#
        );
    }

    #[test]
    fn test_namespace_must_be_registered() {
        let p = Polar::new();
        let valid_policy = "resource Org{}";
        expect_error(
            &p,
            valid_policy,
            "In order to be declared as a namespace, Org must be registered as a class.",
        );
        p.register_constant(sym!("Org"), term!("unimportant"));
        assert!(p.load_str(valid_policy).is_ok());
    }

    #[test]
    fn test_namespace_duplicate_namespaces() {
        let p = Polar::new();
        let invalid_policy = "resource Org{}resource Org{}";
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(
            &p,
            invalid_policy,
            "Duplicate declaration of Org namespace.",
        );
    }

    #[test]
    fn test_namespace_with_undeclared_local_implication_head() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(
            &p,
            r#"resource Org{"member" if "owner";}"#,
            r#"Undeclared term "member" referenced in implication in Org namespace. Did you mean to declare it as a role, permission, or relation?"#,
        );
    }

    #[test]
    fn test_namespace_with_undeclared_local_implication_body() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(
            &p,
            r#"resource Org {
                roles=["member"];
                "member" if "owner";
            }"#,
            r#"Undeclared term "owner" referenced in implication in Org namespace. Did you mean to declare it as a role, permission, or relation?"#,
        );
    }

    #[test]
    fn test_namespace_with_undeclared_nonlocal_implication_body() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"));
        p.register_constant(sym!("Org"), term!("unimportant"));

        expect_error(
            &p,
            r#"resource Repo {
                roles = ["writer"];
                relations = { parent: Org };
                "writer" if "owner" on "parent";
            }"#,
            r#"Repo: Relation "parent" in implication body `"owner" on "parent"` has type Org, but no such namespace exists. Try declaring one: `resource Org {}`"#,
        );

        expect_error(
            &p,
            r#"resource Repo {
                roles = ["writer"];
                relations = { parent: Org };
                "writer" if "owner" on "parent";
            }
            resource Org {}"#,
            r#"Repo: Term "owner" not declared on related resource Org. Did you mean to declare it as a role, permission, or relation on resource Org?"#,
        );
    }

    #[test]
    #[ignore = "probably easier after the entity PR goes in"]
    fn test_namespace_resource_relations_can_only_appear_after_on() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"));
        expect_error(
            &p,
            r#"resource Repo {
                roles = ["owner"];
                relations = { parent: Org };
                "parent" if "owner";
            }"#,
            r#"Repo: resource relation "parent" can only appear in an implication following the keyword 'on'."#,
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_namespace_with_circular_implications() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"));
        let policy = r#"resource Repo {
            roles = [ "writer" ];
            "writer" if "writer";
        }"#;
        panic!("{}", p.load_str(policy).unwrap_err());

        // let policy = r#"resource Repo {
        //     roles = [ "writer", "reader" ];
        //     "writer" if "reader";
        //     "reader" if "writer";
        // }"#;
        // panic!("{}", p.load_str(policy).unwrap_err());
        //
        // let policy = r#"resource Repo {
        //     roles = [ "writer", "reader", "admin" ];
        //     "admin" if "reader";
        //     "writer" if "admin";
        //     "reader" if "writer";
        // }"#;
        // panic!("{}", p.load_str(policy).unwrap_err());
    }

    #[test]
    fn test_namespace_with_unregistered_relation_type() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"));
        let policy = r#"resource Repo { relations = { parent: Org }; }"#;
        expect_error(
            &p,
            policy,
            "Type 'Org' in relation 'parent: Org' must be registered as a class.",
        );
        p.register_constant(sym!("Org"), term!("unimportant"));
        p.load_str(policy).unwrap();
    }

    #[test]
    fn test_namespace_with_clashing_declarations() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));

        expect_error(
            &p,
            r#"resource Org{
              roles = ["egg","egg"];
              "egg" if "egg";
            }"#,
            r#"Org: Duplicate declaration of "egg" in the roles list."#,
        );

        expect_error(
            &p,
            r#"resource Org{
              roles = ["egg","tootsie"];
              permissions = ["spring","egg"];

              "egg" if "tootsie";
              "tootsie" if "spring";
            }"#,
            r#"Org: "egg" declared as a permission but it was previously declared as a role."#,
        );

        expect_error(
            &p,
            r#"resource Org{
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
            entity_type: EntityType::Resource,
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
                let mut policy = "resource Repo {\n".to_owned();
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
            r#"resource Org{roles={};}"#,
            r#"Expected 'roles' declaration to be a list of strings; found a dictionary:"#,
        );
        expect_error(
            &p,
            r#"resource Org{relations=[];}"#,
            r#"Expected 'relations' declaration to be a dictionary; found a list:"#,
        );
        expect_error(
            &p,
            r#"resource Org{foo=[];}"#,
            r#"Unexpected declaration 'foo'. Did you mean for this to be 'roles = [ ... ];' or 'permissions = [ ... ];'?"#,
        );
        expect_error(
            &p,
            r#"resource Org{foo={};}"#,
            r#"Unexpected declaration 'foo'. Did you mean for this to be 'relations = { ... };'?"#,
        );
        expect_error(
            &p,
            r#"resource Org{"foo" if "bar" onn "baz";}"#,
            r#"Unexpected relation keyword 'onn'. Did you mean 'on'?"#,
        );
    }

    #[test]
    fn test_namespace_entity_type() {
        let p = Polar::new();

        expect_error(
            &p,
            "Org{}",
            "Expected 'actor' or 'resource' but found nothing.",
        );

        expect_error(
            &p,
            "seahorse Org{}",
            "Expected 'actor' or 'resource' but found 'seahorse'.",
        );
    }

    #[test]
    fn test_namespace_declaration_keywords_are_not_reserved_words() {
        let p = Polar::new();
        p.load_str(
            "on(actor, resource, roles, permissions, relations) if on(actor, resource, roles, permissions, relations);",
        )
        .unwrap();
    }
}
