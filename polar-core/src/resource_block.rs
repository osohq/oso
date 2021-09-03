use std::collections::{HashMap, HashSet};
use std::ops::Range;

use lalrpop_util::ParseError as LalrpopError;

use super::error::{ParseError, PolarError, PolarResult, RuntimeError};
use super::kb::KnowledgeBase;
use super::lexer::Token;
use super::rules::*;
use super::terms::*;

pub const ACTOR_UNION_NAME: &str = "Actor";
pub const RESOURCE_UNION_NAME: &str = "Resource";

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
    Roles(Term),                               // List<String>
    Permissions(Term),                         // List<String>
    Relations(Term),                           // Dict<Symbol, Symbol>
    ShorthandRule(Term, (Term, Option<Term>)), // (String, (String, Option<String>))
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
            error: ParseError::ResourceBlock { loc, msg, ranges },
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
            Err(LalrpopError::User { error: ParseError::ResourceBlock { loc, msg, ranges } })
        }
        ("relations", Value::List(_)) => Err(LalrpopError::User {
            error: ParseError::ResourceBlock {
                loc: term.offset(),
                msg: "Expected 'relations' declaration to be a dictionary; found a list:\n".to_owned(),
                ranges: vec![term_source_range(&term)],
            },
        }),

        (_, Value::List(_)) => Err(LalrpopError::User {
            error: ParseError::ResourceBlock {
                loc: term.offset(),
                msg: format!(
                    "Unexpected declaration '{}'. Did you mean for this to be 'roles = [ ... ];' or 'permissions = [ ... ];'?\n", name
                ),
                ranges: vec![term_source_range(&term)],
            },
        }),
        (_, Value::Dictionary(_)) => Err(LalrpopError::User {
            error: ParseError::ResourceBlock {
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

pub fn turn_productions_into_resource_block(
    keyword: Option<Term>,
    resource: Term,
    productions: Vec<Production>,
) -> Result<ResourceBlock, LalrpopError<usize, Token, error::ParseError>> {
    if let Some(keyword) = keyword {
        let block_type = match keyword.value().as_symbol().unwrap().0.as_ref() {
            "actor" => BlockType::Actor,
            "resource" => BlockType::Resource,
            _ => {
                let (loc, ranges) = (keyword.offset(), vec![]);
                let msg = format!(
                    "Expected 'actor' or 'resource' but found '{}'.",
                    keyword.to_polar()
                );
                let error = ParseError::ResourceBlock { loc, msg, ranges };
                return Err(LalrpopError::User { error });
            }
        };

        let mut roles: Option<Term> = None;
        let mut permissions: Option<Term> = None;
        let mut relations: Option<Term> = None;
        let mut shorthand_rules = vec![];

        let make_error = |name: &str, previous: &Term, new: &Term| {
            let loc = new.offset();
            let ranges = vec![term_source_range(previous), term_source_range(new)];
            let msg = format!(
                "Multiple '{}' declarations in '{}' resource block.\n",
                name,
                resource.to_polar()
            );
            ParseError::ResourceBlock { loc, msg, ranges }
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
                Production::ShorthandRule(head, body) => {
                    // TODO(gj): Warn the user on duplicate rule definitions.
                    shorthand_rules.push(ShorthandRule { head, body });
                }
            }
        }

        Ok(ResourceBlock {
            block_type,
            resource,
            roles,
            permissions,
            relations,
            shorthand_rules,
        })
    } else {
        let (loc, ranges) = (resource.offset(), vec![]);
        let msg = "Expected 'actor' or 'resource' but found nothing.".to_owned();
        let error = ParseError::ResourceBlock { loc, msg, ranges };
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
pub struct ShorthandRule {
    /// `Term` is a `String`. E.g., `"member"` in `"member" if "owner";`.
    pub head: Term,
    /// Both terms are strings. The former is the 'implier' and the latter is the 'relation', e.g.,
    /// `"owner"` and `"parent"`, respectively, in `"writer" if "owner" on "parent";`.
    pub body: (Term, Option<Term>),
}

impl ShorthandRule {
    pub fn as_rule(&self, resource_block: &Term, blocks: &ResourceBlocks) -> PolarResult<Rule> {
        let Self { head, body } = self;
        // Copy SourceInfo from head of shorthand rule.
        // TODO(gj): assert these can only be None in tests.
        let src_id = head.get_source_id().unwrap_or(0);
        let (start, end) = head.span().unwrap_or((0, 0));

        let name = blocks.get_rule_name_for_declaration_in_resource_block(head, resource_block)?;
        let params = shorthand_rule_head_to_params(head, resource_block);
        let body = shorthand_rule_body_to_rule_body(body, resource_block, blocks)?;

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

// TODO(gj): this will go away when we have true unions in the future.
/// Resource blocks can either be declared as actors or resources.
#[derive(Clone, Debug, PartialEq)]
pub enum BlockType {
    Actor,
    Resource,
}

/// Successfully-parsed but not-yet-fully-validated-or-persisted resource block.
#[derive(Clone, Debug, PartialEq)]
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
    ) {
        self.declarations.insert(resource.clone(), declarations);
        self.shorthand_rules
            .insert(resource.clone(), shorthand_rules);
        match block_type {
            BlockType::Actor => self.actors.insert(resource),
            BlockType::Resource => self.resources.insert(resource),
        };
    }

    fn exists(&self, resource: &Term) -> bool {
        self.declarations.contains_key(resource)
    }

    /// Look up `declaration` in `resource` block.
    ///
    /// Invariant: `resource` _must_ exist.
    fn get_declaration_in_resource_block(
        &self,
        declaration: &Term,
        resource: &Term,
    ) -> PolarResult<&Declaration> {
        if let Some(declaration) = self.declarations[resource].get(declaration) {
            Ok(declaration)
        } else {
            let (loc, ranges) = (declaration.offset(), vec![]);
            let msg = format!("Undeclared term {} referenced in rule in the '{}' resource block. Did you mean to declare it as a role, permission, or relation?", declaration.to_polar(), resource);
            Err(ParseError::ResourceBlock { loc, msg, ranges }.into())
        }
    }

    /// Look up `relation` in `resource` block and return its type.
    fn get_relation_type_in_resource_block(
        &self,
        relation: &Term,
        resource: &Term,
    ) -> PolarResult<&Term> {
        self.get_declaration_in_resource_block(relation, resource)?
            .as_relation_type()
    }

    /// Look up `declaration` in `resource` block and return the appropriate rule name for
    /// rewriting.
    fn get_rule_name_for_declaration_in_resource_block(
        &self,
        declaration: &Term,
        resource: &Term,
    ) -> PolarResult<Symbol> {
        Ok(self
            .get_declaration_in_resource_block(declaration, resource)?
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
                let (loc, ranges) = (declaration.offset(), vec![]);
                let msg = format!("{}: Term {} not declared in related resource block '{}'. Did you mean to declare it as a role, permission, or relation in the '{}' resource block?", resource.to_polar(), declaration.to_polar(), related_block.to_polar(), related_block.to_polar());
                Err(ParseError::ResourceBlock { loc, msg, ranges }.into())
            }
        } else {
            let (loc, ranges) = (related_block.offset(), vec![]);
            let msg = format!("{}: Relation {} in rule body `{} on {}` has type '{}', but no such resource block exists. Try declaring one: `resource {} {{}}`", resource.to_polar(), relation.to_polar(), declaration.to_polar(), relation.to_polar(), related_block.to_polar(), related_block.to_polar());
            Err(ParseError::ResourceBlock { loc, msg, ranges }.into())
        }
    }
}

pub fn check_all_relation_types_have_been_registered(kb: &KnowledgeBase) -> Vec<PolarError> {
    let mut errors = vec![];
    for declarations in kb.resource_blocks.declarations.values() {
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
                return Err(ParseError::ResourceBlock { loc, msg, ranges }.into());
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
                return Err(ParseError::ResourceBlock { loc, msg, ranges }.into());
            }
        }
    }

    if let Some(relations) = relations {
        for (relation, relation_type) in &relations.value().as_dict()?.fields {
            // Stringify relation so that we can index into the declarations map with a string
            // reference to the relation. E.g., relation `creator: User` gets stored as
            // `"creator" => Relation(User)` so that when we encounter a shorthand rule
            // `"admin" if "creator";` we can easily look up what type of declaration `"creator"`
            // is.
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
                return Err(ParseError::ResourceBlock { loc, msg, ranges }.into());
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

/// Turn a shorthand rule body into an `And`-wrapped call (for a local rule) or pair of calls (for
/// a cross-resource rule).
fn shorthand_rule_body_to_rule_body(
    (implier, relation): &(Term, Option<Term>),
    resource: &Term,
    blocks: &ResourceBlocks,
) -> PolarResult<Term> {
    // Create a variable derived from the current block's resource name. E.g., if we're in the
    // `Repo` resource block, the variable name will be `repo`.
    let resource_var = implier.clone_with_value(resource_as_var(resource));

    // The actor variable will always be named `actor`.
    let actor_var = implier.clone_with_value(value!(sym!("actor")));

    // If there's a relation, e.g., `if <implier> on <relation>`...
    if let Some(relation) = relation {
        // TODO(gj): what if the relation is with the same type? E.g.,
        // `Dir { relations = { parent: Dir }; }`. This might cause Polar to loop.

        // ...then we need to link the rewritten `<implier>` and `<relation>` rules via a shared
        // variable. To be clever, we'll name the variable according to the type of the relation,
        // e.g., if the declared relation is `parent: Org` we'll name the variable `org`.
        let relation_type = blocks.get_relation_type_in_resource_block(relation, resource)?;
        let relation_type_var = relation.clone_with_value(resource_as_var(relation_type));

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
                implier, relation, resource
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
            name: blocks.get_rule_name_for_declaration_in_resource_block(implier, resource)?,
            args: vec![actor_var, implier.clone(), resource_var],
            kwargs: None
        }));

        // Wrap the rewritten `<implier>` call in an `And`.
        Ok(implier.clone_with_value(value!(op!(And, implier_call))))
    }
}

/// Turn a shorthand rule head into a trio of params that go in the head of the rewritten rule.
fn shorthand_rule_head_to_params(head: &Term, resource: &Term) -> Vec<Parameter> {
    let resource_name = &resource.value().as_symbol().expect("sym").0;
    vec![
        Parameter {
            parameter: head.clone_with_value(value!(sym!("actor"))),
            specializer: Some(head.clone_with_value(value!(pattern!(instance!(ACTOR_UNION_NAME))))),
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

// TODO(gj): better error message, e.g.:
//               duplicate resource block declared: resource Org { ... } defined on line XX of file YY
//                                                  previously defined on line AA of file BB
fn check_for_duplicate_resource_blocks(
    blocks: &ResourceBlocks,
    resource: &Term,
) -> PolarResult<()> {
    if blocks.exists(resource) {
        let (loc, ranges) = (resource.offset(), vec![]);
        let msg = format!("Duplicate declaration of '{}' resource block.", resource);
        return Err(ParseError::ResourceBlock { loc, msg, ranges }.into());
    }
    Ok(())
}

// TODO(gj): no way to know in the core if `term` was registered as a class or a constant.
fn is_registered_class(kb: &KnowledgeBase, term: &Term) -> PolarResult<bool> {
    Ok(kb.is_constant(term.value().as_symbol()?))
}

fn check_that_block_type_is_not_already_registered(
    kb: &KnowledgeBase,
    block_type: &BlockType,
    resource: &Term,
) -> PolarResult<()> {
    let union_name = match block_type {
        BlockType::Actor => ACTOR_UNION_NAME,
        BlockType::Resource => RESOURCE_UNION_NAME,
    };
    let already_registered = is_registered_class(kb, &term!(sym!(union_name)))?;
    if already_registered {
        let msg = format!("Cannot declare '{} {} {{ ... }}'; '{}' already registered as a constant. To resolve this conflict, please register '{}' under a different name.", block_type.to_polar(), resource.to_polar(), union_name, union_name);
        let (loc, ranges) = (resource.offset(), vec![]);
        return Err(ParseError::ResourceBlock { loc, msg, ranges }.into());
    }
    Ok(())
}

fn check_that_block_resource_is_registered(kb: &KnowledgeBase, resource: &Term) -> PolarResult<()> {
    if !is_registered_class(kb, resource)? {
        // TODO(gj): better error message
        let msg = format!(
            "Invalid resource block '{}' -- '{}' must be a registered class.",
            resource.to_polar(),
            resource.to_polar(),
        );
        let (loc, ranges) = (resource.offset(), vec![]);
        // TODO(gj): UnregisteredClassError in the core.
        return Err(ParseError::ResourceBlock { loc, msg, ranges }.into());
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
        return Err(ParseError::ResourceBlock { loc, msg, ranges }.into());
    }
    Ok(())
}

fn check_that_shorthand_rule_heads_are_declared_locally(
    shorthand_rules: &[ShorthandRule],
    declarations: &Declarations,
    resource: &Term,
) -> Vec<PolarError> {
    let mut errors = vec![];
    for ShorthandRule { head, .. } in shorthand_rules {
        if !declarations.contains_key(head) {
            let msg = format!(
                "Undeclared term {} referenced in rule in '{}' resource block. \
                Did you mean to declare it as a role, permission, or relation?",
                head.to_polar(),
                resource
            );
            let (loc, ranges) = (head.offset(), vec![]);
            let error = ParseError::ResourceBlock { loc, msg, ranges };
            errors.push(error.into());
        }
    }
    errors
}

impl ResourceBlock {
    pub fn add_to_kb(self, kb: &mut KnowledgeBase) -> PolarResult<()> {
        let mut errors = vec![];
        errors.extend(
            check_that_block_type_is_not_already_registered(kb, &self.block_type, &self.resource)
                .err(),
        );
        errors.extend(check_that_block_resource_is_registered(kb, &self.resource).err());
        errors
            .extend(check_for_duplicate_resource_blocks(&kb.resource_blocks, &self.resource).err());

        let ResourceBlock {
            block_type,
            resource,
            roles,
            permissions,
            relations,
            shorthand_rules,
        } = self;

        let declarations = index_declarations(roles, permissions, relations, &resource)?;

        errors.append(&mut check_that_shorthand_rule_heads_are_declared_locally(
            &shorthand_rules,
            &declarations,
            &resource,
        ));

        // TODO(gj): Emit all errors instead of just the first.
        if !errors.is_empty() {
            return Err(errors[0].clone());
        }

        kb.resource_blocks
            .add(block_type, resource, declarations, shorthand_rules);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use permute::permute;

    use std::collections::HashSet;

    use super::*;
    use crate::events::QueryEvent;
    use crate::parser::{parse_lines, Line};
    use crate::polar::Polar;

    #[track_caller]
    fn expect_error(p: &Polar, policy: &str, expected: &str) {
        let msg = match p.load_str(policy).unwrap_err() {
            error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::ResourceBlock { msg, .. }),
                ..
            } => msg,
            _ => panic!(),
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
            body: (term!("member"), Some(term!("parent"))),
        };
        let rewritten_role_role = shorthand_rule
            .as_rule(&term!(sym!("repo")), &blocks)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            format!("has_role(actor: {}{{}}, \"reader\", repo_instance: repo{{}}) if has_relation(org_instance, \"parent\", repo_instance) and has_role(actor, \"member\", org_instance);", ACTOR_UNION_NAME),
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
            rewritten_role_role.to_polar(),
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
            rewritten_permission_role.to_polar(),
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
            rewritten_permission_permission.to_polar(),
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
            body: (term!("member"), Some(term!("parent"))),
        };
        let rewritten_role_role = shorthand_rule
            .as_rule(&term!(sym!("Repo")), &blocks)
            .unwrap();
        assert_eq!(
            rewritten_role_role.to_polar(),
            format!("has_role(actor: {}{{}}, \"reader\", repo: Repo{{}}) if has_relation(org, \"parent\", repo) and has_role(actor, \"member\", org);", ACTOR_UNION_NAME),
        );
    }

    #[test]
    fn test_resource_block_resource_must_be_registered() {
        let p = Polar::new();
        let valid_policy = "resource Org{}";
        expect_error(
            &p,
            valid_policy,
            "Invalid resource block 'Org' -- 'Org' must be a registered class.",
        );
        p.register_constant(sym!("Org"), term!("unimportant"));
        assert!(p.load_str(valid_policy).is_ok());
    }

    #[test]
    fn test_resource_block_duplicates() {
        let p = Polar::new();
        let invalid_policy = "resource Org{}resource Org{}";
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(
            &p,
            invalid_policy,
            "Duplicate declaration of 'Org' resource block.",
        );
    }

    #[test]
    fn test_resource_block_with_undeclared_local_shorthand_rule_head() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(
            &p,
            r#"resource Org{"member" if "owner";}"#,
            r#"Undeclared term "member" referenced in rule in 'Org' resource block. Did you mean to declare it as a role, permission, or relation?"#,
        );
    }

    #[test]
    fn test_resource_block_with_undeclared_local_shorthand_rule_body() {
        let p = Polar::new();
        p.register_constant(sym!("Org"), term!("unimportant"));
        expect_error(
            &p,
            r#"resource Org {
                roles=["member"];
                "member" if "owner";
            }"#,
            r#"Undeclared term "owner" referenced in rule in the 'Org' resource block. Did you mean to declare it as a role, permission, or relation?"#,
        );
    }

    #[test]
    fn test_resource_block_with_undeclared_nonlocal_shorthand_rule_body() {
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
        p.register_constant(sym!("Repo"), term!("unimportant"));
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
    #[ignore = "not yet implemented"]
    fn test_resource_block_with_circular_shorthand_rules() {
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
    fn test_resource_block_with_unregistered_relation_type() {
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
    fn test_resource_block_with_clashing_declarations() {
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
    fn test_resource_block_parsing_permutations() {
        use std::iter::FromIterator;

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
                    body: (term!("member"), Some(term!("parent"))),
                },
            ],
        };

        // Helpers

        let equal = |line: &Line, expected: &ResourceBlock| match line {
            Line::ResourceBlock(parsed) => {
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

        let test_case = |parts: Vec<&str>, expected: &ResourceBlock| {
            for permutation in permute(parts).into_iter() {
                let mut policy = "resource Repo {\n".to_owned();
                policy += &permutation.join("\n");
                policy += "}";
                assert!(equal(&parse_lines(0, &policy).unwrap()[0], expected));
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
            Err(error::PolarError {
                kind: error::ErrorKind::Parse(error::ParseError::ResourceBlock { msg, .. }),
                ..
            }) => msg,
            Err(e) => panic!("{}", e),
            _ => panic!("succeeded when I should've failed"),
        };
        assert_eq!(msg, "hi");
    }

    #[test]
    fn test_union_type_matches() {
        // When no unions exist, `Actor` is a regular variable.
        let polar = Polar::new();
        polar.register_constant(sym!(ACTOR_UNION_NAME), term!(1));
        let query = polar.new_query(
            &format!("{} matches {}", ACTOR_UNION_NAME, ACTOR_UNION_NAME),
            false,
        );
        let next_event = query.unwrap().next_event().unwrap();
        assert!(matches!(next_event, QueryEvent::ExternalIsa { .. }));

        // When unions exist, `Actor matches Actor` because a union matches itself.
        let polar = Polar::new();
        polar.register_constant(sym!("User"), term!("unimportant"));
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
        polar.register_constant(sym!("User"), term!("unimportant"));
        polar.register_constant(sym!("Repo"), term!("unimportant"));
        polar.load_str("actor User {} resource Repo {}").unwrap();
        let query = polar.new_query(
            &format!("not {} matches {}", ACTOR_UNION_NAME, RESOURCE_UNION_NAME),
            false,
        );
        let next_event = query.unwrap().next_event().unwrap();
        assert!(matches!(next_event, QueryEvent::Result { .. }));
    }

    #[test]
    fn test_union_type_cannot_be_registered() {
        let polar = Polar::new();
        polar.register_constant(sym!(ACTOR_UNION_NAME), term!("unimportant"));
        expect_error(&polar, "actor User {}", "Cannot declare 'actor User { ... }'; 'Actor' already registered as a constant. To resolve this conflict, please register 'Actor' under a different name.")
    }

    #[test]
    fn test_validate_rules_with_union_type_specializers() {
        let mut kb = KnowledgeBase::new();
        kb.constant(
            sym!("Fruit"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 1,
                constructor: None,
                repr: None
            })),
        );
        kb.constant(
            sym!("Citrus"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 2,
                constructor: None,
                repr: None
            })),
        );
        kb.constant(
            sym!("Orange"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 3,
                constructor: None,
                repr: None
            })),
        );
        kb.add_mro(sym!("Fruit"), vec![1]).unwrap();
        // Citrus is a subclass of Fruit
        kb.add_mro(sym!("Citrus"), vec![2, 1]).unwrap();
        // Orange is a subclass of Citrus
        kb.add_mro(sym!("Orange"), vec![3, 2, 1]).unwrap();

        kb.constant(
            sym!("User"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 4,
                constructor: None,
                repr: None
            })),
        );
        kb.add_mro(sym!("User"), vec![4]).unwrap();

        // Add member to 'Resource' union.
        kb.resource_blocks.resources.insert(term!(sym!("Citrus")));
        // Add member to 'Actor' union.
        kb.resource_blocks.actors.insert(term!(sym!("User")));

        // Union matches union.
        kb.add_rule_prototype(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        assert!(kb.validate_rules().is_ok());

        // TODO(gj): revisit when we have unions beyond Actor & Resource. Union A matches
        // union B if union A is a member of union B.
        //
        // Union A does not match union B.
        kb.clear_rules();
        kb.add_rule_prototype(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!(ACTOR_UNION_NAME))]));
        assert!(matches!(
            kb.validate_rules().unwrap_err(),
            PolarError {
                kind: error::ErrorKind::Validation(error::ValidationError::InvalidRule { .. }),
                ..
            }
        ));

        // Member of union matches union.
        kb.clear_rules();
        kb.add_rule_prototype(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Citrus"))]));
        assert!(kb.validate_rules().is_ok());

        // TODO(gj): revisit when we have unions beyond Actor & Resource. Member of union A matches
        // union B if union A is a member of union B.
        //
        // Member of union A does not match union B.
        kb.clear_rules();
        kb.add_rule_prototype(rule!("f", ["x"; instance!(sym!(ACTOR_UNION_NAME))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Citrus"))]));
        assert!(matches!(
            kb.validate_rules().unwrap_err(),
            PolarError {
                kind: error::ErrorKind::Validation(error::ValidationError::InvalidRule { .. }),
                ..
            }
        ));

        // Subclass of member of union matches union.
        kb.clear_rules();
        kb.add_rule_prototype(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Orange"))]));
        assert!(kb.validate_rules().is_ok());

        // Superclass of member of union does not match union.
        kb.clear_rules();
        kb.add_rule_prototype(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Fruit"))]));
        assert!(matches!(
            kb.validate_rules().unwrap_err(),
            PolarError {
                kind: error::ErrorKind::Validation(error::ValidationError::InvalidRule { .. }),
                ..
            }
        ));

        // TODO(gj): revisit when we have unions beyond Actor & Resource. Not currently possible to
        // have an instance of a member of a union as a specializer until we have true unions where
        // we could define, e.g., `type MyUnion = Integer;`
        //
        // Instance of member of union matches union.
        // kb.clear_rules();
        // kb.add_rule_prototype(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        // kb.add_rule(rule!("f", ["x"; 1]));
        // assert!(kb.validate_rules().is_ok());

        // TODO(gj): revisit when we have unions beyond Actor & Resource. Not currently possible to
        // have an instance of a member of a union as a specializer until we have true unions where
        // we could define, e.g., `type MyUnion = Integer;`
        //
        // Instance of subclass of member of union matches union.
        // kb.clear_rules();
        // kb.add_rule_prototype(rule!("f", ["x"; instance!(sym!(RESOURCE_UNION_NAME))]));
        // kb.add_rule(rule!("f", ["x"; 1]));
        // assert!(kb.validate_rules().is_ok());
    }

    // TODO(gj): add test for union pattern with fields. Behavior will probably be the same as for
    // fieldless union pattern where we create a choicepoint of matches against every union member
    // with the same set of fields.
}
