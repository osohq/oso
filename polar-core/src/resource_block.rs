use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};

use super::error::{invalid_state, PolarError, PolarResult, ValidationError};
use super::kb::KnowledgeBase;
use super::rules::*;
use super::terms::*;

pub const ACTOR_UNION_NAME: &str = "Actor";
pub const RESOURCE_UNION_NAME: &str = "Resource";

// TODO(gj): round up longhand `has_permission/3` and `has_role/3` rules to incorporate their
// referenced permissions & roles (implied & implier side) into the exhaustiveness checks.

// TODO(gj): round up longhand `has_relation/3` rules to check that every declared `relation` has a
// corresponding `has_relation/3` implementation.

// TODO(gj): disallow same string to be declared as a perm/role and a relation.
// This'll come into play for "owner"-style actor relationships.

// This type is used as a pre-validation bridge between LALRPOP & Rust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Production {
    Declaration((Term, Term)), // (Symbol, List<String> | Dict<Symbol, Symbol>)
    ShorthandRule(Term, (Term, Option<(Term, Term)>)), // (String, (String, Option<(Symbol, String)>))
}

fn validate_relation_keyword(keyword: &Term) -> PolarResult<()> {
    if keyword.as_symbol()?.0 != "on" {
        return Err(ValidationError::ResourceBlock {
            msg: format!(
                "Unexpected relation keyword '{}'. Did you mean 'on'?",
                keyword
            ),
            term: keyword.clone(),
        }
        .into());
    }
    Ok(())
}

#[derive(Debug)]
pub enum ParsedDeclaration {
    Roles(Term),       // List<String>
    Permissions(Term), // List<String>
    Relations(Term),   // Dict<Symbol, Symbol>
}

pub fn validate_parsed_declaration((name, term): (Term, Term)) -> PolarResult<ParsedDeclaration> {
    match (name.as_symbol()?.0.as_ref(), term.value()) {
        ("roles", Value::List(_)) => Ok(ParsedDeclaration::Roles(term)),
        ("permissions", Value::List(_)) => Ok(ParsedDeclaration::Permissions(term)),
        ("relations", Value::Dictionary(_)) => Ok(ParsedDeclaration::Relations(term)),

        ("roles", Value::Dictionary(_)) | ("permissions", Value::Dictionary(_)) => Err(ValidationError::ResourceBlock {
            msg: format!("Expected '{}' declaration to be a list of strings; found a dictionary", name),
            term
        }),
        ("relations", Value::List(_)) => Err(ValidationError::ResourceBlock {
            msg: "Expected 'relations' declaration to be a dictionary; found a list".to_owned(),
            term
        }),

        (_, Value::List(_)) => Err(ValidationError::ResourceBlock {
            msg: format!("Unexpected declaration '{}'. Did you mean for this to be 'roles = [ ... ];' or 'permissions = [ ... ];'?", name),
            term
        }),
        (_, Value::Dictionary(_)) => Err(ValidationError::ResourceBlock {
            msg: format!("Unexpected declaration '{}'. Did you mean for this to be 'relations = {{ ... }};'?", name),
            term
        }),
        _ => unreachable!(),
    }.map_err(Into::into)
}

pub fn block_type_from_keyword(keyword: Option<Term>, resource: &Term) -> PolarResult<BlockType> {
    if let Some(keyword) = keyword {
        match keyword.as_symbol()?.0.as_ref() {
            "actor" => Ok(BlockType::Actor),
            "resource" => Ok(BlockType::Resource),
            other => Err(ValidationError::ResourceBlock {
                msg: format!("Expected 'actor' or 'resource' but found '{}'.", other),
                term: keyword.clone(),
            }
            .into()),
        }
    } else {
        // TODO(gj): add `resource` into this message -- e.g., ("Expected `actor {resource}` or
        // `resource {resource}` ...", resource=resource).
        Err(ValidationError::ResourceBlock {
            msg: "Expected 'actor' or 'resource' but found nothing.".to_owned(),
            term: resource.clone(),
        }
        .into())
    }
}

pub fn resource_block_from_productions(
    keyword: Option<Term>,
    resource: Term,
    productions: Vec<Production>,
) -> (ResourceBlock, Vec<PolarError>) {
    let mut errors = vec![];

    let block_type = match block_type_from_keyword(keyword, &resource) {
        Ok(block_type) => block_type,
        Err(e) => {
            errors.push(e);
            // NOTE(gj): it doesn't matter what we default to here since the (unrecoverable)
            // `ResourceBlock` error pushed on the line above means we aren't going to make it to
            // rule type validation, which is the only place where the `BlockType` distinction
            // matters. I think `Resource` makes marginally more sense than `Actor` since the
            // `BlockType` distinction will go away and there will only be `Resource` blocks once
            // we add better union types and can specify the `Actor` union as a union instead of as
            // `actor Blah {}` "blocks".
            BlockType::Resource
        }
    };

    let mut roles: Option<Term> = None;
    let mut permissions: Option<Term> = None;
    let mut relations: Option<Term> = None;
    let mut shorthand_rules = vec![];

    // TODO(gj): attach 'previous' to error via `related_info` section.
    let make_error = |name: &str, _previous: &Term, new: &Term| {
        ValidationError::ResourceBlock {
            msg: format!(
                "Multiple '{}' declarations in '{}' resource block.",
                name, resource
            ),
            term: new.clone(),
        }
        .into()
    };

    for production in productions {
        match production {
            Production::Declaration(declaration) => {
                match validate_parsed_declaration(declaration) {
                    Ok(ParsedDeclaration::Roles(new)) => {
                        // TODO(gj): combine roles _and_ push error so that we can use the declared
                        // roles in validating shorthand rules even in the face of resource block
                        // errors?
                        if let Some(previous) = roles {
                            errors.push(make_error("roles", &previous, &new));
                        }
                        roles = Some(new);
                    }
                    Ok(ParsedDeclaration::Permissions(new)) => {
                        if let Some(previous) = permissions {
                            errors.push(make_error("permissions", &previous, &new));
                        }
                        permissions = Some(new);
                    }
                    Ok(ParsedDeclaration::Relations(new)) => {
                        if let Some(previous) = relations {
                            errors.push(make_error("relations", &previous, &new));
                        }
                        relations = Some(new);
                    }
                    Err(e) => errors.push(e),
                }
            }
            Production::ShorthandRule(head, body) => {
                // TODO(gj): Warn the user on duplicate rule definitions.
                shorthand_rules.push(ShorthandRule { head, body });
            }
        }
    }

    (
        ResourceBlock {
            block_type,
            resource,
            roles,
            permissions,
            relations,
            shorthand_rules,
        },
        errors,
    )
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Declaration {
    Role,
    Permission,
    /// `Term` is a `Symbol` that is the (registered) type of the relation. E.g., `Org` in `parent: Org`.
    Relation(Term),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ShorthandRule {
    /// `Term` is a `String`. E.g., `"member"` in `"member" if "owner";`.
    pub head: Term,
    /// The first `Term` is the 'implier' `String`, e.g., `"owner"` in `"member" if "owner";`. The
    /// `Option` is the optional 'relation' `Symbol` and `String`, e.g., `on "parent"` in `"member"
    /// if "owner" on "parent";`.
    pub body: (Term, Option<(Term, Term)>),
}

impl ShorthandRule {
    pub fn as_rule(&self, resource_name: &Term, blocks: &ResourceBlocks) -> PolarResult<Rule> {
        let Self { head, body } = self;
        Ok(Rule {
            name: blocks.get_rule_name_for_declaration_in_resource_block(head, resource_name)?,
            params: shorthand_rule_head_to_params(head, resource_name)?,
            body: shorthand_rule_body_to_rule_body(body, resource_name, blocks)?,
            // Copy SourceInfo from head of shorthand rule.
            source_info: head.source_info().clone(),
            required: false,
        })
    }
}

type Declarations = HashMap<Term, Declaration>;

impl Declaration {
    fn as_rule_name(&self) -> Symbol {
        sym!(&format!("has_{}", self))
    }
}

// TODO(gj): this will go away when we have true unions in the future.
/// Resource blocks can either be declared as actors or resources.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockType {
    Actor,
    Resource,
}

/// Successfully-parsed but not-yet-fully-validated-or-persisted resource block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceBlock {
    pub block_type: BlockType,
    pub resource: Term,
    pub roles: Option<Term>,
    pub permissions: Option<Term>,
    pub relations: Option<Term>,
    pub shorthand_rules: Vec<ShorthandRule>,
}

#[derive(Clone, Default)]
pub struct ResourceBlocks {
    /// Map from resource (`Symbol`) to the declarations in that resource's block.
    declarations: HashMap<Term, Declarations>,
    /// Map from resource (`Symbol`) to the shorthand rules declared in that resource's block.
    pub shorthand_rules: HashMap<Term, Vec<ShorthandRule>>,
    /// Set of all resource block types declared as actors. Internally treated like a union type
    /// where all declared types are members of the union.
    pub actors: HashSet<Term>,
    /// Set of all resource block types declared as resources. Internally treated like a union type
    /// where all declared types are members of the union.
    pub resources: HashSet<Term>,
}

impl ResourceBlocks {
    pub fn new() -> Self {
        Self {
            declarations: HashMap::new(),
            shorthand_rules: HashMap::new(),
            actors: HashSet::new(),
            resources: HashSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.declarations.clear();
        self.shorthand_rules.clear();
        self.actors.clear();
        self.resources.clear();
    }

    fn add(
        &mut self,
        block_type: BlockType,
        resource: Term,
        declarations: Declarations,
        shorthand_rules: Vec<ShorthandRule>,
    ) -> Vec<PolarError> {
        let mut errors = vec![];

        // Merge existing declarations if we are reopening a resource block, otherwise add new
        if let Some(existing) = self.declarations.get_mut(&resource) {
            for (key, new) in declarations {
                if let Some(existing) = existing.insert(key.clone(), new.clone()) {
                    if existing != new {
                        errors.push(
                            ValidationError::DuplicateResourceBlockDeclaration {
                                resource: resource.clone(),
                                declaration: key.clone(),
                                existing,
                                new: new.clone(),
                            }
                            .into(),
                        );
                    }
                }
            }
        } else {
            // or insert a new set of declarations for the resource type
            self.declarations.insert(resource.clone(), declarations);
        }

        // Merge existing shorthand rules if we are reopening a resource block, otherwise add new
        self.shorthand_rules
            .entry(resource.clone())
            .and_modify(|existing| existing.extend(shorthand_rules.clone()))
            .or_insert(shorthand_rules);

        match block_type {
            BlockType::Actor => {
                self.actors.insert(resource.clone());
                self.resources.insert(resource)
            }
            BlockType::Resource => self.resources.insert(resource),
        };

        errors
    }

    /// Look up `declaration` in `resource` block.
    ///
    /// Invariant: `resource` _must_ exist.
    fn get_declaration_in_resource_block(
        &self,
        declaration: &Term,
        resource_name: &Term,
    ) -> PolarResult<&Declaration> {
        let maybe_declarations = self.declarations.get(resource_name);
        let maybe_declaration = maybe_declarations.and_then(|ds| ds.get(declaration));
        if let Some(declaration) = maybe_declaration {
            Ok(declaration)
        } else {
            Err(ValidationError::ResourceBlock {
                msg: format!("Undeclared term {} referenced in rule in '{}' resource block. Did you mean to declare it as a role, permission, or relation?", declaration, resource_name),
                term: declaration.clone(),
            }.into())
        }
    }

    /// Look up `relation` in `resource` block and return its type.
    pub fn get_relation_type_in_resource_block(
        &self,
        relation: &Term,
        resource: &Term,
    ) -> PolarResult<&Term> {
        let declaration = self.get_declaration_in_resource_block(relation, resource)?;
        if let Declaration::Relation(related_type) = declaration {
            Ok(related_type)
        } else {
            invalid_state(format!("Expected Relation; got: {:?}", declaration))
        }
    }

    /// Look up `declaration` in `resource` block and return the appropriate rule name for
    /// rewriting.
    fn get_rule_name_for_declaration_in_resource_block(
        &self,
        declaration: &Term,
        resource_name: &Term,
    ) -> PolarResult<Symbol> {
        Ok(self
            .get_declaration_in_resource_block(declaration, resource_name)?
            .as_rule_name())
    }

    /// Traverse from `resource` block to a related resource block via `relation`, then look up
    /// `declaration` in the related block and return the appropriate rule name for rewriting.
    fn get_rule_name_for_declaration_in_related_resource_block(
        &self,
        declaration: &Term,
        relation: &Term,
        resource: &Term,
    ) -> PolarResult<Symbol> {
        let related_block = self.get_relation_type_in_resource_block(relation, resource)?;

        if let Some(declarations) = self.declarations.get(related_block) {
            if let Some(declaration) = declarations.get(declaration) {
                Ok(declaration.as_rule_name())
            } else {
                Err(ValidationError::ResourceBlock {
                    msg: format!("{}: Term {} not declared in related resource block '{}'. Did you mean to declare it as a role, permission, or relation in the '{}' resource block?", resource, declaration, related_block, related_block),
                    term: declaration.clone(),
                }.into())
            }
        } else {
            Err(ValidationError::ResourceBlock {
                msg: format!("{}: Relation {} in rule body `{} on {}` has type '{}', but no such resource block exists. Try declaring one: `resource {} {{}}`", resource, relation, declaration, relation, related_block, related_block),
                term: related_block.clone(),
            }.into())
        }
    }

    pub fn declarations(&self) -> &HashMap<Term, Declarations> {
        &self.declarations
    }

    pub fn has_roles(&self) -> bool {
        let mut declarations = self.declarations().values().flat_map(HashMap::values);
        declarations.any(|d| matches!(d, Declaration::Role))
    }

    pub fn relation_tuples(&self) -> Vec<(&Term, &Term, &Term)> {
        let mut tuples = vec![];
        for (object, declarations) in self.declarations() {
            for (name, declaration) in declarations {
                if let Declaration::Relation(subject) = declaration {
                    tuples.push((subject, name, object));
                }
            }
        }
        tuples
    }
}

// TODO(gj): build up errors but keep on truckin'.
fn index_declarations(
    roles: Option<Term>,
    permissions: Option<Term>,
    relations: Option<Term>,
    resource: &Term,
) -> PolarResult<HashMap<Term, Declaration>> {
    let mut declarations = HashMap::new();

    if let Some(roles) = roles {
        for role in roles.as_list()? {
            if let Some(existing) = declarations.insert(role.clone(), Declaration::Role) {
                return Err(ValidationError::DuplicateResourceBlockDeclaration {
                    resource: resource.clone(),
                    declaration: role.clone(),
                    existing,
                    new: Declaration::Role,
                }
                .into());
            }
        }
    }

    if let Some(permissions) = permissions {
        for permission in permissions.as_list()? {
            if let Some(existing) = declarations.insert(permission.clone(), Declaration::Permission)
            {
                return Err(ValidationError::DuplicateResourceBlockDeclaration {
                    resource: resource.clone(),
                    declaration: permission.clone(),
                    existing,
                    new: Declaration::Permission,
                }
                .into());
            }
        }
    }

    if let Some(relations) = relations {
        for (relation, relation_type) in &relations.as_dict()?.fields {
            // Stringify relation so that we can index into the declarations map with a string
            // reference to the relation. E.g., relation `creator: User` gets stored as
            // `"creator" => Relation(User)` so that when we encounter a shorthand rule
            // `"admin" if "creator";` we can easily look up what type of declaration `"creator"`
            // is.
            let stringified_relation = relation_type.clone_with_value(value!(relation.0.as_str()));
            let declaration = Declaration::Relation(relation_type.clone());

            if let Some(existing) =
                declarations.insert(stringified_relation.clone(), declaration.clone())
            {
                return Err(ValidationError::DuplicateResourceBlockDeclaration {
                    resource: resource.clone(),
                    declaration: stringified_relation,
                    existing,
                    new: declaration,
                }
                .into());
            }
        }
    }
    Ok(declarations)
}

fn resource_name_as_var(resource_name: &Term, related: bool) -> PolarResult<Value> {
    let name = &resource_name.as_symbol()?.0;
    let mut lowercased = name.to_lowercase();

    // If the resource's name is already lowercase, append "_instance" to distinguish the variable
    // name from the resource's name. In most cases, the resource name will not be lowercase (e.g.,
    // `Organization` or `RepositorySettings`).
    if &lowercased == name {
        lowercased += "_instance";
    }

    // If the `related` flag is specified, prepend "related_" to distinguish the variable name for
    // the *related* resource from the variable name for the relatee resource. Without this logic,
    // resources in a recursive relationship will have the same variable name.
    if related {
        lowercased.insert_str(0, "related_");
    }

    Ok(value!(sym!(lowercased)))
}

/// Turn a shorthand rule body into an `And`-wrapped call (for a local rule) or pair of calls (for
/// a cross-resource rule).
fn shorthand_rule_body_to_rule_body(
    (implier, relation): &(Term, Option<(Term, Term)>),
    resource_name: &Term,
    blocks: &ResourceBlocks,
) -> PolarResult<Term> {
    // Create a variable derived from the current block's resource name. E.g., if we're in the
    // `Repo` resource block, the variable name will be `repo`.
    let resource_var = implier.clone_with_value(resource_name_as_var(resource_name, false)?);

    // The actor variable will always be named `actor`.
    let actor_var = implier.clone_with_value(value!(sym!("actor")));

    // If there's a relation, e.g., `if <implier> <keyword> <relation>`...
    if let Some((keyword, relation)) = relation {
        // ...then we need to validate the keyword...
        validate_relation_keyword(keyword)?;

        // ...and then link the rewritten `<implier>` and `<relation>` rules via a shared variable.
        // To be clever, we'll name the variable according to the type of the relation, e.g., if
        // the declared relation is `parent: Org` we'll name the variable `org`.
        let relation_type = blocks.get_relation_type_in_resource_block(relation, resource_name)?;
        let relation_type_var =
            relation.clone_with_value(resource_name_as_var(relation_type, true)?);

        // For the rewritten `<relation>` call, the rule name will always be `has_relation` and the
        // arguments, in order, will be: the shared variable we just created above, the
        // `<relation>` string, and the resource variable we created at the top of the function.
        // E.g., `vec![org, "parent", repo]`.
        let relation_call = relation.clone_with_value(value!(Call {
            name: sym!("has_relation"),
            args: vec![relation_type_var.clone(), relation.clone(), resource_var],
            kwargs: None
        }));

        // To get the rule name for the rewritten `<implier>` call, we need to figure out what type
        // (role, permission, or relation) `<implier>` is declared as _in the resource block
        // related to the current resource block via `<relation>`_. That is, given
        // `resource Repo { roles=["writer"]; relations={parent:Org}; "writer" if "owner" on "parent"; }`,
        // we need to find out whether `"owner"` is declared as a role, permission, or relation in
        // the `Org` resource block. The args for the rewritten `<implier>` call are, in order: the
        // actor variable, the `<implier>` string, and the shared variable we created above for the
        // related type.
        let implier_call = implier.clone_with_value(value!(Call {
            name: blocks.get_rule_name_for_declaration_in_related_resource_block(
                implier,
                relation,
                resource_name
            )?,
            args: vec![actor_var, implier.clone(), relation_type_var],
            kwargs: None
        }));

        // Wrap the rewritten `<relation>` and `<implier>` calls in an `And`.
        Ok(implier.clone_with_value(value!(op!(And, relation_call, implier_call))))
    } else {
        // If there's no `<relation>` (e.g., `... if "writer";`), we're dealing with a local rule,
        // and the rewriting process is a bit simpler. To get the appropriate rule name, we look up
        // the declared type (role, permission, or relation) of `<implier>` in the current resource
        // block. The call's args are, in order: the actor variable, the `<implier>` string, and
        // the resource variable. E.g., `vec![actor, "writer", repo]`.
        let implier_call = implier.clone_with_value(value!(Call {
            name: blocks.get_rule_name_for_declaration_in_resource_block(implier, resource_name)?,
            args: vec![actor_var, implier.clone(), resource_var],
            kwargs: None
        }));

        // Wrap the rewritten `<implier>` call in an `And`.
        Ok(implier.clone_with_value(value!(op!(And, implier_call))))
    }
}

/// Turn a shorthand rule head into a trio of params that go in the head of the rewritten rule.
fn shorthand_rule_head_to_params(head: &Term, resource: &Term) -> PolarResult<Vec<Parameter>> {
    let resource_name = &resource.as_symbol()?.0;
    let params = vec![
        Parameter {
            parameter: head.clone_with_value(value!(sym!("actor"))),
            specializer: Some(head.clone_with_value(value!(pattern!(instance!(ACTOR_UNION_NAME))))),
        },
        Parameter {
            parameter: head.clone(),
            specializer: None,
        },
        Parameter {
            parameter: head.clone_with_value(resource_name_as_var(resource, false)?),
            specializer: Some(
                resource.clone_with_value(value!(pattern!(instance!(resource_name)))),
            ),
        },
    ];
    Ok(params)
}

impl ResourceBlock {
    pub fn add_to_kb(self, kb: &mut KnowledgeBase) -> Vec<PolarError> {
        let mut errors = vec![];
        // Check that resource block's resource has been registered as a class.
        errors.extend(kb.get_registered_class(&self.resource).err());

        let ResourceBlock {
            block_type,
            resource,
            roles,
            permissions,
            relations,
            shorthand_rules,
        } = self;

        match index_declarations(roles, permissions, relations, &resource) {
            Ok(declarations) => {
                errors.extend(kb.resource_blocks.add(
                    block_type,
                    resource,
                    declarations,
                    shorthand_rules,
                ));
            }
            Err(e) => errors.push(e),
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use permutohedron::Heap;

    use super::*;
    use crate::error::{
        ErrorKind::{Runtime, Validation},
        RuntimeError, ValidationError,
    };
    use crate::events::QueryEvent;
    use crate::parser::{parse_lines, Line};
    use crate::polar::Polar;
    use crate::sources::Source;

    #[track_caller]
    fn expect_error(p: &Polar, policy: &str, expected: &str) {
        let error = p.load_str(policy).unwrap_err().unwrap_validation();
        let msg = match error {
            ValidationError::ResourceBlock { msg, .. } => msg,
            ValidationError::UnregisteredClass { .. }
            | ValidationError::DuplicateResourceBlockDeclaration { .. } => error.to_string(),
            _ => panic!("Unexpected error: {}", error),
        };
        assert!(msg.contains(expected));
    }

    #[test]
    fn test_resource_block_rewrite_shorthand_rules_with_lowercase_resource_specializer() {
        let repo_resource = term!(sym!("repo"));
        let repo_roles = term!(["reader"]);
        let repo_relations = term!(btreemap! { sym!("parent") => term!(sym!("org")) });
        let repo_declarations =
            index_declarations(Some(repo_roles), None, Some(repo_relations), &repo_resource);

        let org_resource = term!(sym!("org"));
        let org_roles = term!(["member"]);
        let org_declarations = index_declarations(Some(org_roles), None, None, &org_resource);

        let mut blocks = ResourceBlocks::new();
        blocks.add(
            BlockType::Resource,
            repo_resource,
            repo_declarations.unwrap(),
            vec![],
        );
        blocks.add(
            BlockType::Resource,
            org_resource,
            org_declarations.unwrap(),
            vec![],
        );
        let shorthand_rule = ShorthandRule {
            head: term!("reader"),
            body: (term!("member"), Some((term!(sym!("on")), term!("parent")))),
        };
        let rewritten_role_role = shorthand_rule
            .as_rule(&term!(sym!("repo")), &blocks)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_string(),
            format!("has_role(actor: {}{{}}, \"reader\", repo_instance: repo{{}}) if has_relation(related_org_instance, \"parent\", repo_instance) and has_role(actor, \"member\", related_org_instance);", ACTOR_UNION_NAME),
        );
    }

    #[test]
    fn test_resource_block_rewrite_shorthand_rules_with_recursive_relation() {
        let resource = term!(sym!("Dir"));
        let permissions = term!(["read"]);
        let relations = term!(btreemap! { sym!("parent") => resource.clone() });
        let declarations = index_declarations(None, Some(permissions), Some(relations), &resource);

        let mut blocks = ResourceBlocks::new();
        blocks.add(
            BlockType::Resource,
            resource.clone(),
            declarations.unwrap(),
            vec![],
        );

        let shorthand_rule = ShorthandRule {
            head: term!("read"),
            body: (term!("read"), Some((term!(sym!("on")), term!("parent")))),
        };
        let rewritten_role_role = shorthand_rule.as_rule(&resource, &blocks).unwrap();

        assert_eq!(
            rewritten_role_role.to_string(),
            format!("has_permission(actor: {}{{}}, \"read\", dir: Dir{{}}) if has_relation(related_dir, \"parent\", dir) and has_permission(actor, \"read\", related_dir);", ACTOR_UNION_NAME),
        );
    }

    #[test]
    fn test_resource_block_local_rewrite_shorthand_rules() {
        let resource = term!(sym!("Org"));
        let roles = term!(["owner", "member"]);
        let permissions = term!(["invite", "create_repo"]);
        let declarations = index_declarations(Some(roles), Some(permissions), None, &resource);
        let mut blocks = ResourceBlocks::new();
        blocks.add(BlockType::Resource, resource, declarations.unwrap(), vec![]);
        let shorthand_rule = ShorthandRule {
            head: term!("member"),
            body: (term!("owner"), None),
        };
        let rewritten_role_role = shorthand_rule
            .as_rule(&term!(sym!("Org")), &blocks)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_string(),
            format!("has_role(actor: {}{{}}, \"member\", org: Org{{}}) if has_role(actor, \"owner\", org);", ACTOR_UNION_NAME),
        );

        let shorthand_rule = ShorthandRule {
            head: term!("invite"),
            body: (term!("owner"), None),
        };
        let rewritten_permission_role = shorthand_rule
            .as_rule(&term!(sym!("Org")), &blocks)
            .unwrap();
        assert_eq!(
            rewritten_permission_role.to_string(),
            format!("has_permission(actor: {}{{}}, \"invite\", org: Org{{}}) if has_role(actor, \"owner\", org);", ACTOR_UNION_NAME),
        );

        let shorthand_rule = ShorthandRule {
            head: term!("create_repo"),
            body: (term!("invite"), None),
        };
        let rewritten_permission_permission = shorthand_rule
            .as_rule(&term!(sym!("Org")), &blocks)
            .unwrap();
        assert_eq!(
            rewritten_permission_permission.to_string(),
            format!("has_permission(actor: {}{{}}, \"create_repo\", org: Org{{}}) if has_permission(actor, \"invite\", org);", ACTOR_UNION_NAME),
        );
    }

    #[test]
    fn test_resource_block_nonlocal_rewrite_shorthand_rules() {
        let repo_resource = term!(sym!("Repo"));
        let repo_roles = term!(["reader"]);
        let repo_relations = term!(btreemap! { sym!("parent") => term!(sym!("Org")) });
        let repo_declarations =
            index_declarations(Some(repo_roles), None, Some(repo_relations), &repo_resource);
        let org_resource = term!(sym!("Org"));
        let org_roles = term!(["member"]);
        let org_declarations = index_declarations(Some(org_roles), None, None, &org_resource);
        let mut blocks = ResourceBlocks::new();
        blocks.add(
            BlockType::Resource,
            repo_resource,
            repo_declarations.unwrap(),
            vec![],
        );
        blocks.add(
            BlockType::Resource,
            org_resource,
            org_declarations.unwrap(),
            vec![],
        );
        let shorthand_rule = ShorthandRule {
            head: term!("reader"),
            body: (term!("member"), Some((term!(sym!("on")), term!("parent")))),
        };
        let rewritten_role_role = shorthand_rule
            .as_rule(&term!(sym!("Repo")), &blocks)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_string(),
            format!("has_role(actor: {}{{}}, \"reader\", repo: Repo{{}}) if has_relation(related_org, \"parent\", repo) and has_role(actor, \"member\", related_org);", ACTOR_UNION_NAME),
        );
    }

    #[test]
    fn test_resource_block_resource_must_be_registered() {
        let p = Polar::new();
        let valid_policy = "resource Org{}";
        expect_error(&p, valid_policy, "Unregistered class: Org");
        p.register_constant(sym!("Org"), term!("unimportant"))
            .unwrap();
        assert!(p.load_str(valid_policy).is_ok());
    }

    #[test]
    fn test_resource_block_declarations_spread_over_multiple_resource_blocks() {
        let p = Polar::new();

        let repo_instance = ExternalInstance {
            instance_id: 1,
            constructor: None,
            repr: None,
            class_repr: None,
            class_id: None,
        };
        let repo_term = term!(Value::ExternalInstance(repo_instance.clone()));
        let repo_name = sym!("Repo");
        p.register_constant(repo_name.clone(), repo_term).unwrap();
        p.register_mro(repo_name, vec![repo_instance.instance_id])
            .unwrap();

        // Use declarations from one resource block in shorthand rules in another.
        let valid_policy = r#"
            resource Repo {
              relations = { readable_parent: Repo };
              roles = ["reader"];
              permissions = ["read"];

              "write" if "writer";
              "write" if "write" on "writable_parent";
            }

            resource Repo {
              relations = { writable_parent: Repo };
              roles = ["writer"];
              permissions = ["write"];

              "read" if "reader";
              "read" if "read" on "readable_parent";
            }

            has_role(_: Actor, _: String, _: Resource);
            has_relation(subject: Repo, "writable_parent", object: Repo) if
                object.parent_id = subject.id;
            has_relation(subject: Repo, "readable_parent", object: Repo) if
                object.parent_id = subject.id;
        "#;

        p.load_str(valid_policy).unwrap();
        // Create explicit scope to allow the RWLock obtained from kb.read() to
        // be dropped explicitly and independently of the function scope.
        {
            let blocks = &p.kb.read().unwrap().resource_blocks;
            let declarations = blocks.declarations.get(&term!(sym!("Repo"))).unwrap();
            assert_eq!(declarations.len(), 6);
            let shorthand_rules = blocks.shorthand_rules.get(&term!(sym!("Repo"))).unwrap();
            assert_eq!(shorthand_rules.len(), 4);
        }
        p.clear_rules();

        // Raise a validation error if shorthand rules reference declarations
        // not found in any matching block
        let invalid_policy = r#"
            resource Repo {
                permissions = ["write"];
            }

            resource Repo {
                roles = ["reader"];
                "read" if "reader";
            }

            has_role(actor: Actor, _role: String, repo: Repo) if
                repo in actor.repos;
        "#;
        expect_error(
            &p,
            invalid_policy,
            r#"Undeclared term "read" referenced in rule in 'Repo' resource block"#,
        );

        // Multiple blocks can declare distinct roles/permissions/relations.
        let valid_policy = r#"
            resource Repo {
              relations = { readable_parent: Repo };
              roles = ["reader"];
              permissions = ["read"];

              "read" if "reader";
              "read" if "read" on "readable_parent";
            }

            resource Repo {
              relations = { writable_parent: Repo };
              roles = ["writer"];
              permissions = ["write"];

              "write" if "writer";
              "write" if "write" on "writable_parent";
            }

            has_role(_: Actor, _: String, _: Resource);
            has_relation(subject: Repo, "writable_parent", object: Repo) if
                object.parent_id = subject.id;
            has_relation(subject: Repo, "readable_parent", object: Repo) if
                object.parent_id = subject.id;
        "#;

        p.load_str(valid_policy).unwrap();
        // Create explicit scope to allow the RWLock obtained from kb.read() to
        // be dropped explicitly and independently of the function scope.
        {
            let blocks = &p.kb.read().unwrap().resource_blocks;
            let declarations = blocks.declarations.get(&term!(sym!("Repo"))).unwrap();
            assert_eq!(declarations.len(), 6);
            let shorthand_rules = blocks.shorthand_rules.get(&term!(sym!("Repo"))).unwrap();
            assert_eq!(shorthand_rules.len(), 4);
        }
        p.clear_rules();

        // Duplicate declarations are fine if they're used in multiple blocks.
        let valid_policy = r#"
            resource Repo {
              relations = { readable_parent: Repo, writable_parent: Repo };
              roles = ["reader", "writer"];
              permissions = ["read", "write"];

              "read" if "reader";
              "write" if "writer";

              "read" if "read" on "readable_parent";
              "write" if "write" on "writable_parent";
            }

            resource Repo {
              relations = { readable_parent: Repo, writable_parent: Repo };
              roles = ["reader", "writer"];
              permissions = ["read", "write"];

              "reader" if "writer";

              "read" if "reader" on "readable_parent";
              "write" if "writer" on "writable_parent";
            }

            has_role(_: Actor, _: String, _: Resource);
            has_relation(subject: Repo, "writable_parent", object: Repo) if
                object.parent_id = subject.id;
            has_relation(subject: Repo, "readable_parent", object: Repo) if
                object.parent_id = subject.id;
        "#;
        p.load_str(valid_policy).unwrap();
        {
            let blocks = &p.kb.read().unwrap().resource_blocks;
            let declarations = blocks.declarations.get(&term!(sym!("Repo"))).unwrap();
            assert_eq!(declarations.len(), 6);
            let shorthand_rules = blocks.shorthand_rules.get(&term!(sym!("Repo"))).unwrap();
            assert_eq!(shorthand_rules.len(), 7);
        }
    }

    #[test]
    fn test_resource_block_declarations_overwriting() {
        let p = Polar::new();

        let repo_instance = ExternalInstance {
            instance_id: 1,
            constructor: None,
            repr: None,
            class_repr: None,
            class_id: None,
        };
        let repo_term = term!(Value::ExternalInstance(repo_instance.clone()));
        let repo_name = sym!("Repo");
        p.register_constant(repo_name.clone(), repo_term).unwrap();
        p.register_mro(repo_name, vec![repo_instance.instance_id])
            .unwrap();

        // validate overwriting declarations of the same type is ok
        let valid_policy = r#"
            resource Repo {
                roles = ["reader"];
            }

            resource Repo {
                roles = ["reader"];
            }

            has_role(actor: Actor, _role: String, repo: Repo) if
               repo in actor.repos;
        "#;
        assert!(p.load_str(valid_policy).is_ok());
        p.clear_rules();

        // validate overwriting of different types throws error
        let invalid_policy = r#"
            resource Repo {
                roles = ["reader"];
            }

            resource Repo {
                permissions = ["reader"];
            }

            has_role(actor: Actor, _role: String, repo: Repo) if
               repo in actor.repos;
        "#;

        expect_error(
            &p,
            invalid_policy,
            r#"Cannot overwrite existing role declaration "reader" in resource Repo with permission"#,
        );

        // validate overwriting of different types throws error
        let invalid_policy = r#"
            resource Repo {
                roles = ["reader"];
            }

            resource Repo {
                relations = { reader: Repo };
            }

            has_role(actor: Actor, _role: String, repo: Repo) if
                repo in actor.repos;
            has_relation(subject: Repo, "reader", object: Repo) if
                object.parent_id = subject.id;
        "#;

        expect_error(
            &p,
            invalid_policy,
            r#"Cannot overwrite existing role declaration "reader" in resource Repo with relation"#,
        );

        // validating overwriting relations of different types throws error
        let invalid_policy = r#"
            actor User {}
            resource Repo {
                relations = { reader: Repo };
            }

            resource Repo {
                relations = { reader: User };
            }

            has_role(actor: Actor, _role: String, repo: Repo) if
                repo in actor.repos;
            has_relation(subject: Repo, "reader", object: Repo) if
                object.parent_id = subject.id;
        "#;
        let user_instance = ExternalInstance {
            instance_id: 2,
            constructor: None,
            repr: None,
            class_repr: None,
            class_id: None,
        };
        let user_term = term!(Value::ExternalInstance(user_instance.clone()));
        let user_name = sym!("User");
        p.register_constant(user_name.clone(), user_term).unwrap();
        p.register_mro(user_name, vec![user_instance.instance_id])
            .unwrap();
        expect_error(
            &p,
            invalid_policy,
            r#"Cannot overwrite existing relation declaration "reader" in resource Repo with relation"#,
        );
    }

    #[test]
    fn test_resource_block_with_undeclared_local_shorthand_rule_head() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"))
            .unwrap();
        expect_error(
            &p,
            r#"resource Org{"member" if "owner";}"#,
            r#"Undeclared term "member" referenced in rule in 'Org' resource block. Did you mean to declare it as a role, permission, or relation?"#,
        );
    }

    #[test]
    fn test_resource_block_with_undeclared_local_shorthand_rule_body() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"))
            .unwrap();
        expect_error(
            &p,
            r#"resource Org {
                roles=["member"];
                "member" if "owner";
            }"#,
            r#"Undeclared term "owner" referenced in rule in 'Org' resource block. Did you mean to declare it as a role, permission, or relation?"#,
        );
    }

    #[test]
    fn test_resource_block_with_undeclared_nonlocal_shorthand_rule_body() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"))
            .unwrap();
        p.register_constant(sym!("Org"), term!("unimportant"))
            .unwrap();

        expect_error(
            &p,
            r#"resource Repo {
                roles = ["writer"];
                relations = { parent: Org };
                "writer" if "owner" on "parent";
            }"#,
            r#"Repo: Relation "parent" in rule body `"owner" on "parent"` has type 'Org', but no such resource block exists. Try declaring one: `resource Org {}`"#,
        );

        expect_error(
            &p,
            r#"resource Repo {
                roles = ["writer"];
                relations = { parent: Org };
                "writer" if "owner" on "parent";
            }
            resource Org {}"#,
            r#"Repo: Term "owner" not declared in related resource block 'Org'. Did you mean to declare it as a role, permission, or relation in the 'Org' resource block?"#,
        );
    }

    #[test]
    #[ignore = "probably easier after the entity PR goes in"]
    fn test_resource_block_resource_relations_can_only_appear_after_on() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"))
            .unwrap();
        expect_error(
            &p,
            r#"resource Repo {
                roles = ["owner"];
                relations = { parent: Org };
                "parent" if "owner";
            }"#,
            r#"Repo: resource relation "parent" can only appear in a rule body following the keyword 'on'."#,
        );
    }

    #[test]
    #[ignore = "outside scope of current task; leaving test here to be implemented later"]
    fn test_resource_block_resource_relations_are_the_only_thing_that_can_appear_after_on() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"))
            .unwrap();
        expect_error(
            &p,
            r#"resource Repo {
                roles = ["writer"];
                "writer" if "owner" on "parent";
            }"#,
            r#"Repo: term "parent" must be declared as a relation: `relations = { parent: <SomeResource> };`"#,
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_resource_block_with_circular_shorthand_rules() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"))
            .unwrap();
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
    fn test_resource_block_with_unregistered_relation_type() {
        let p = Polar::new();
        p.register_constant(sym!("Repo"), term!("unimportant"))
            .unwrap();
        let policy = r#"resource Repo { relations = { parent: Org }; }"#;
        expect_error(&p, policy, "Unregistered class: Org");
        p.register_constant(sym!("Org"), term!("unimportant"))
            .unwrap();
        p.load_str(policy).unwrap();
    }

    #[test]
    fn test_resource_block_with_clashing_declarations() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"))
            .unwrap();

        expect_error(
            &p,
            r#"resource Org{
              roles = ["egg","egg"];
              "egg" if "egg";
            }"#,
            r#"Cannot overwrite existing role declaration "egg" in resource Org with role"#,
        );

        expect_error(
            &p,
            r#"resource Org{
              roles = ["egg","tootsie"];
              permissions = ["spring","egg"];

              "egg" if "tootsie";
              "tootsie" if "spring";
            }"#,
            r#"Cannot overwrite existing role declaration "egg" in resource Org with permission"#,
        );

        expect_error(
            &p,
            r#"resource Org{
              permissions = [ "egg" ];
              relations = { egg: Roll };
            }"#,
            r#"Cannot overwrite existing permission declaration "egg" in resource Org with relation"#,
        );
    }

    #[test]
    fn test_resource_block_parsing_permutations() {
        // Policy pieces
        let roles = r#"roles = ["writer", "reader"];"#;
        let permissions = r#"permissions = ["push", "pull"];"#;
        let relations = r#"relations = { creator: User, parent: Org };"#;
        let shorthand_rules = vec![
            r#""pull" if "reader";"#,
            r#""push" if "writer";"#,
            r#""writer" if "creator";"#,
            r#""reader" if "member" on "parent";"#,
        ];

        // Maximal block
        let block = ResourceBlock {
            block_type: BlockType::Resource,
            resource: term!(sym!("Repo")),
            roles: Some(term!(["writer", "reader"])),
            permissions: Some(term!(["push", "pull"])),
            relations: Some(term!(btreemap! {
                sym!("creator") => term!(sym!("User")),
                sym!("parent") => term!(sym!("Org")),
            })),
            shorthand_rules: vec![
                // TODO(gj): shorthand_rule! macro
                ShorthandRule {
                    head: term!("pull"),
                    body: (term!("reader"), None),
                },
                ShorthandRule {
                    head: term!("push"),
                    body: (term!("writer"), None),
                },
                ShorthandRule {
                    head: term!("writer"),
                    body: (term!("creator"), None),
                },
                ShorthandRule {
                    head: term!("reader"),
                    body: (term!("member"), Some((term!(sym!("on")), term!("parent")))),
                },
            ],
        };

        // Helpers

        let equal = |line: &Line, expected: &ResourceBlock| match line {
            Line::ResourceBlock {
                keyword,
                resource,
                productions,
            } => {
                let (parsed, _) = resource_block_from_productions(
                    keyword.clone(),
                    resource.clone(),
                    productions.clone(),
                );
                let parsed_shorthand_rules: HashSet<&ShorthandRule> =
                    HashSet::from_iter(&parsed.shorthand_rules);
                let expected_shorthand_rules = HashSet::from_iter(&expected.shorthand_rules);
                parsed.resource == expected.resource
                    && parsed.roles == expected.roles
                    && parsed.permissions == expected.permissions
                    && parsed.relations == expected.relations
                    && parsed_shorthand_rules == expected_shorthand_rules
            }
            _ => false,
        };

        let test_case = |mut parts: Vec<&str>, expected: &ResourceBlock| {
            for permutation in Heap::new(&mut parts) {
                let mut policy = "resource Repo {\n".to_owned();
                policy += &permutation.join("\n");
                policy += "}";
                assert!(equal(
                    &parse_lines(Source::new(policy)).unwrap()[0],
                    expected
                ));
            }
        };

        // Test each case with and without shorthand rules.
        let test_cases = |parts: Vec<&str>, expected: &ResourceBlock| {
            let mut parts_with_shorthand_rules = parts.clone();
            parts_with_shorthand_rules.append(&mut shorthand_rules.clone());
            test_case(parts_with_shorthand_rules, expected);

            let expected_without_shorthand_rules = ResourceBlock {
                shorthand_rules: vec![],
                ..expected.clone()
            };
            test_case(parts, &expected_without_shorthand_rules);
        };

        // Cases

        // Roles, Permissions, Relations
        test_cases(vec![roles, permissions, relations], &block);

        // Roles, Permissions, _________
        let expected = ResourceBlock {
            relations: None,
            ..block.clone()
        };
        test_cases(vec![roles, permissions], &expected);

        // Roles, ___________, Relations
        let expected = ResourceBlock {
            permissions: None,
            ..block.clone()
        };
        test_cases(vec![roles, relations], &expected);

        // _____, Permissions, Relations
        let expected = ResourceBlock {
            roles: None,
            ..block.clone()
        };
        test_cases(vec![permissions, relations], &expected);

        // Roles, ___________, _________
        let expected = ResourceBlock {
            permissions: None,
            relations: None,
            ..block.clone()
        };
        test_cases(vec![roles], &expected);

        // _____, Permissions, _________
        let expected = ResourceBlock {
            roles: None,
            relations: None,
            ..block.clone()
        };
        test_cases(vec![permissions], &expected);

        // _____, ___________, Relations
        let expected = ResourceBlock {
            roles: None,
            permissions: None,
            ..block.clone()
        };
        test_cases(vec![relations], &expected);

        // _____, ___________, _________
        let expected = ResourceBlock {
            roles: None,
            permissions: None,
            relations: None,
            ..block
        };
        test_cases(vec![], &expected);
    }

    #[test]
    fn test_resource_block_declaration_keywords() {
        let p = Polar::new();
        expect_error(
            &p,
            r#"resource Org{roles={};}"#,
            r#"Expected 'roles' declaration to be a list of strings; found a dictionary"#,
        );
        expect_error(
            &p,
            r#"resource Org{relations=[];}"#,
            r#"Expected 'relations' declaration to be a dictionary; found a list"#,
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
    }

    #[test]
    fn test_resource_block_relation_keywords() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"))
            .unwrap();
        expect_error(
            &p,
            r#"resource Org {
              roles = ["foo", "bar"];
              relations = { baz: Org };
              "foo" if "bar" onn "baz";
            }"#,
            "Unexpected relation keyword 'onn'. Did you mean 'on'?",
        );
    }

    #[test]
    fn test_resource_block_types() {
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
    fn test_resource_block_declaration_keywords_are_not_reserved_words() {
        let p = Polar::new();
        p.load_str(
            "on(actor, resource, roles, permissions, relations) if on(actor, resource, roles, permissions, relations);",
        )
        .unwrap();
    }

    // TODO(gj): test union types in all of the positions where classes can appear, such as in
    // `new` expressions.

    #[test]
    #[ignore = "unimplemented"]
    fn test_resource_block_union_types_are_not_constructable() {
        let p = Polar::new();
        let q = p.new_query(&format!("new {}()", ACTOR_UNION_NAME), false);
        let msg = match q {
            Err(PolarError(Validation(ValidationError::ResourceBlock { msg, .. }))) => msg,
            Err(e) => panic!("{}", e),
            _ => panic!("succeeded when I should've failed"),
        };
        assert_eq!(msg, "hi");
    }

    #[test]
    fn test_union_type_matches() {
        // When unions exist, `Actor matches Actor` because a union matches itself.
        let polar = Polar::new();
        polar
            .register_constant(sym!("User"), term!("unimportant"))
            .unwrap();
        polar.load_str("actor User {}").unwrap();
        let query = polar.new_query(
            &format!("{} matches {}", ACTOR_UNION_NAME, ACTOR_UNION_NAME),
            false,
        );
        let next_event = query.unwrap().next_event().unwrap();
        assert!(matches!(next_event, QueryEvent::Result { .. }));

        // When unions exist, `not Actor matches Resource` because a union doesn't match a
        // different union.
        let polar = Polar::new();
        polar
            .register_constant(sym!("User"), term!("unimportant"))
            .unwrap();
        polar
            .register_constant(sym!("Repo"), term!("unimportant"))
            .unwrap();
        polar.load_str("actor User {} resource Repo {}").unwrap();
        let query = polar.new_query(
            &format!("not {} matches {}", ACTOR_UNION_NAME, RESOURCE_UNION_NAME),
            false,
        );
        let next_event = query.unwrap().next_event().unwrap();
        assert!(matches!(next_event, QueryEvent::Result { .. }));
    }

    #[test]
    fn test_union_type_names_are_reserved() {
        let polar = Polar::new();
        let err = polar
            .register_constant(sym!(ACTOR_UNION_NAME), term!("unimportant"))
            .expect_err("Expected register_constant to throw error.");
        assert!(matches!(
            err.0,
            Runtime(RuntimeError::InvalidRegistration { .. })
        ));
    }

    #[test]
    fn test_validate_rules_with_union_type_specializers() {
        let mut kb = KnowledgeBase::new();
        kb.register_constant(
            sym!("Fruit"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 1,
                constructor: None,
                repr: None,
                class_repr: None,
                class_id: None,
            })),
        )
        .unwrap();
        kb.register_constant(
            sym!("Citrus"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 2,
                constructor: None,
                repr: None,
                class_repr: None,
                class_id: None,
            })),
        )
        .unwrap();
        kb.register_constant(
            sym!("Orange"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 3,
                constructor: None,
                repr: None,
                class_repr: None,
                class_id: None,
            })),
        )
        .unwrap();
        kb.add_mro(sym!("Fruit"), vec![1]).unwrap();
        // Citrus is a subclass of Fruit
        kb.add_mro(sym!("Citrus"), vec![2, 1]).unwrap();
        // Orange is a subclass of Citrus
        kb.add_mro(sym!("Orange"), vec![3, 2, 1]).unwrap();

        kb.register_constant(
            sym!("User"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 4,
                constructor: None,
                repr: None,
                class_repr: None,
                class_id: None,
            })),
        )
        .unwrap();
        kb.add_mro(sym!("User"), vec![4]).unwrap();

        // Add member to 'Resource' union.
        kb.resource_blocks.resources.insert(term!(sym!("Citrus")));
        // Add member to 'Actor' union.
        kb.resource_blocks.actors.insert(term!(sym!("User")));

        // Union matches union.
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        assert!(kb.validate_rules().is_empty());

        kb.clear_rules();
        kb.resource_blocks.resources.insert(term!(sym!("Citrus")));
        kb.resource_blocks.actors.insert(term!(sym!("User")));

        // TODO(gj): revisit when we have unions beyond Actor & Resource. Union A matches
        // union B if union A is a member of union B.
        //
        // Union A does not match union B.
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!(ACTOR_UNION_NAME))]));
        let diagnostic = kb.validate_rules().into_iter().next().unwrap();
        let error = diagnostic.unwrap_error().unwrap_validation();
        assert!(matches!(error, ValidationError::InvalidRule { .. }));

        kb.clear_rules();
        kb.resource_blocks.resources.insert(term!(sym!("Citrus")));
        kb.resource_blocks.actors.insert(term!(sym!("User")));

        // Member of union matches union.
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Citrus"))]));
        assert!(kb.validate_rules().is_empty());

        kb.clear_rules();
        kb.resource_blocks.resources.insert(term!(sym!("Citrus")));
        kb.resource_blocks.actors.insert(term!(sym!("User")));

        // TODO(gj): revisit when we have unions beyond Actor & Resource. Member of union A matches
        // union B if union A is a member of union B.
        //
        // Member of union A does not match union B.
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!(ACTOR_UNION_NAME))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Citrus"))]));
        let diagnostic = kb.validate_rules().into_iter().next().unwrap();
        let error = diagnostic.unwrap_error().unwrap_validation();
        assert!(matches!(error, ValidationError::InvalidRule { .. }));

        kb.clear_rules();
        kb.resource_blocks.resources.insert(term!(sym!("Citrus")));
        kb.resource_blocks.actors.insert(term!(sym!("User")));

        // Subclass of member of union matches union.
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Orange"))]));
        assert!(kb.validate_rules().is_empty());

        kb.clear_rules();
        kb.resource_blocks.resources.insert(term!(sym!("Citrus")));
        kb.resource_blocks.actors.insert(term!(sym!("User")));

        // Superclass of member of union does not match union.
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Fruit"))]));
        let diagnostic = kb.validate_rules().into_iter().next().unwrap();
        let error = diagnostic.unwrap_error().unwrap_validation();
        assert!(matches!(error, ValidationError::InvalidRule { .. }));

        // kb.clear_rules();
        // kb.resource_blocks.resources.insert(term!(sym!("Citrus")));
        // kb.resource_blocks.actors.insert(term!(sym!("User")));
        //
        // TODO(gj): revisit when we have unions beyond Actor & Resource. Not currently possible to
        // have an instance of a member of a union as a specializer until we have true unions where
        // we could define, e.g., `type MyUnion = Integer;`
        //
        // Instance of member of union matches union.
        // kb.add_rule_type(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        // kb.add_rule(rule!("f", ["x"; 1]));
        // assert!(kb.validate_rules().is_ok());

        // kb.clear_rules();
        // kb.resource_blocks.resources.insert(term!(sym!("Citrus")));
        // kb.resource_blocks.actors.insert(term!(sym!("User")));
        //
        // TODO(gj): revisit when we have unions beyond Actor & Resource. Not currently possible to
        // have an instance of a member of a union as a specializer until we have true unions where
        // we could define, e.g., `type MyUnion = Integer;`
        //
        // Instance of subclass of member of union matches union.
        // kb.add_rule_type(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        // kb.add_rule(rule!("f", ["x"; 1]));
        // assert!(kb.validate_rules().is_ok());
    }

    // TODO(gj): add test for union pattern with fields. Behavior will probably be the same as for
    // fieldless union pattern where we create a choicepoint of matches against every union member
    // with the same set of fields.

    // Test creation of resource-specific rule type (for `has_relation`) and general rule type (for
    // `has_role`):
    //   - has_relation between (Organization, "parent", Repository)
    //   - has_role created because at least one resource block has roles declared
    #[test]
    fn test_create_resource_specific_rule_types() -> PolarResult<()> {
        let policy = r#"
            resource Organization {
                roles = ["member"];
            }

            resource Repository {
                roles = ["reader"];
                relations = {parent: Organization};
                "reader" if "member" on "parent";
            }

            has_relation(organization: Organization, "parent", repository: Repository) if
                repository.org_id = organization.id;

            has_role(user: Actor, _role: String, organization: Organization) if
                organization.id in user.org_ids;
        "#;

        let polar = Polar::new();

        let repo_instance = ExternalInstance {
            instance_id: 1,
            constructor: None,
            repr: None,
            class_repr: None,
            class_id: None,
        };
        let repo_term = term!(Value::ExternalInstance(repo_instance.clone()));
        let repo_name = sym!("Repository");
        polar.register_constant(repo_name.clone(), repo_term)?;
        polar.register_mro(repo_name.clone(), vec![repo_instance.instance_id])?;

        let org_instance = ExternalInstance {
            instance_id: 2,
            constructor: None,
            repr: None,
            class_repr: None,
            class_id: None,
        };
        let org_term = term!(Value::ExternalInstance(org_instance.clone()));
        let org_name = sym!("Organization");
        polar.register_constant(org_name.clone(), org_term)?;
        polar.register_mro(org_name.clone(), vec![org_instance.instance_id])?;

        polar.load_str(policy)?;

        let kb = polar.kb.read().unwrap();

        let has_role_rule_types = kb.get_rule_types(&sym!("has_role")).unwrap();
        // has_role(actor: Actor, role: String, resource: Resource)
        let expected = rule!("has_role", ["actor"; instance!(ACTOR_UNION_NAME), "role"; instance!("String"), "resource"; instance!(RESOURCE_UNION_NAME)]);
        assert_eq!(1, has_role_rule_types.len());
        assert_eq!(has_role_rule_types[0], expected);

        let has_relation_rule_types = kb.get_rule_types(&sym!("has_relation")).unwrap();
        // has_relation(organization: Organization, "parent", repository: Repository)
        let expected = rule!("has_relation", ["subject"; instance!(org_name), "parent", "object"; instance!(repo_name)]);
        assert_eq!(1, has_relation_rule_types.len());
        assert_eq!(has_relation_rule_types[0], expected,);

        Ok(())
    }

    // Test creation of rule types for actor roles
    //   - has_role created because at least one resource block has roles declared
    #[test]
    fn test_create_resource_specific_rule_types_actor_roles() -> PolarResult<()> {
        let policy = r#"
            actor Team {
                roles = ["member", "owner"];

                "member" if "owner";
            }
        "#;

        let polar = Polar::new();

        let team_instance = ExternalInstance {
            instance_id: 1,
            constructor: None,
            repr: None,
            class_repr: None,
            class_id: None,
        };
        let team_term = term!(Value::ExternalInstance(team_instance.clone()));
        let team_name = sym!("Team");
        polar.register_constant(team_name.clone(), team_term)?;
        polar.register_mro(team_name, vec![team_instance.instance_id])?;

        polar.load_str(policy)?;

        let kb = polar.kb.read().unwrap();

        let has_role_rule_types = kb.get_rule_types(&sym!("has_role")).unwrap();
        // has_role(actor: Actor, role: String, resource: Resource)
        let expected = rule!("has_role", ["actor"; instance!(ACTOR_UNION_NAME), "role"; instance!("String"), "resource"; instance!(RESOURCE_UNION_NAME)]);
        assert_eq!(1, has_role_rule_types.len());
        assert_eq!(has_role_rule_types[0], expected);

        Ok(())
    }
}
