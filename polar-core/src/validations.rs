use std::collections::{HashMap, HashSet};

use super::diagnostic::Diagnostic;
use super::error::ValidationError;
use super::formatting::source_lines;
use super::kb::*;
use super::rules::*;
use super::sources::Source;
use super::terms::*;
use super::visitor::{walk_call, walk_rule, walk_term, Visitor};

fn common_misspellings(t: &str) -> Option<String> {
    let misspelled_type = match t {
        "integer" => "Integer",
        "int" => "Integer",
        "i32" => "Integer",
        "i64" => "Integer",
        "u32" => "Integer",
        "u64" => "Integer",
        "usize" => "Integer",
        "size_t" => "Integer",
        "float" => "Float",
        "f32" => "Float",
        "f64" => "Float",
        "double" => "Float",
        "char" => "String",
        "str" => "String",
        "string" => "String",
        "list" => "List",
        "array" => "List",
        "Array" => "List",
        "dict" => "Dictionary",
        "Dict" => "Dictionary",
        "dictionary" => "Dictionary",
        "hash" => "Dictionary",
        "Hash" => "Dictionary",
        "map" => "Dictionary",
        "Map" => "Dictionary",
        "HashMap" => "Dictionary",
        "hashmap" => "Dictionary",
        "hash_map" => "Dictionary",
        _ => return None,
    };
    Some(misspelled_type.to_owned())
}

/// Record singleton variables and unknown specializers in a rule.
struct SingletonVisitor<'kb> {
    kb: &'kb KnowledgeBase,
    singletons: HashMap<Symbol, Option<Term>>,
}

fn diagnostic_from_singleton(term: Term, source: Option<Source>) -> Diagnostic {
    if let Value::Pattern(Pattern::Instance(InstanceLiteral { tag, .. })) = term.value() {
        let mut msg = format!("Unknown specializer {}", tag);
        if let Some(t) = common_misspellings(&tag.0) {
            msg.push_str(&format!(", did you mean {}?", t));
        }
        if let Some(ref source) = source {
            msg.push('\n');
            msg.push_str(&source_lines(source, term.offset(), 0));
        }
        Diagnostic::Warning(msg)
    } else {
        Diagnostic::Error(ValidationError::SingletonVariable { term }.into())
    }
}

impl<'kb> SingletonVisitor<'kb> {
    fn new(kb: &'kb KnowledgeBase) -> Self {
        Self {
            kb,
            singletons: HashMap::new(),
        }
    }

    fn warnings(self) -> Vec<Diagnostic> {
        let mut singletons = self.singletons.into_values().flatten().collect::<Vec<_>>();
        singletons.sort_by_key(Term::offset);
        singletons
            .into_iter()
            .map(|term| {
                let source = self.kb.get_term_source(&term);
                diagnostic_from_singleton(term, source)
            })
            .collect()
    }
}

impl<'kb> Visitor for SingletonVisitor<'kb> {
    fn visit_term(&mut self, t: &Term) {
        match t.value() {
            Value::Variable(v)
            | Value::RestVariable(v)
            | Value::Pattern(Pattern::Instance(InstanceLiteral { tag: v, .. }))
                if !v.is_temporary_var()
                    && !v.is_namespaced_var()
                    && !self.kb.is_constant(v)
                    && !self.kb.is_union(t) =>
            {
                self.singletons
                    .entry(v.clone())
                    .and_modify(|o| *o = None)
                    .or_insert_with(|| Some(t.clone()));
            }
            _ => (),
        }
        walk_term(self, t);
    }
}

pub fn check_singletons(rule: &Rule, kb: &KnowledgeBase) -> Vec<Diagnostic> {
    let mut visitor = SingletonVisitor::new(kb);
    walk_rule(&mut visitor, rule);
    visitor.warnings()
}

struct AndOrPrecendenceCheck<'kb> {
    kb: &'kb KnowledgeBase,
    unparenthesized_expr: Vec<(Source, Term)>,
}

impl<'kb> AndOrPrecendenceCheck<'kb> {
    fn new(kb: &'kb KnowledgeBase) -> Self {
        Self {
            kb,
            unparenthesized_expr: Default::default(),
        }
    }

    fn warnings(&mut self) -> Vec<Diagnostic> {
        self.unparenthesized_expr
            .iter()
            .map(|(source, or_term)| {
                let mut msg = "Expression without parentheses could be ambiguous. \n\
                    Prior to 0.20, `x and y or z` would parse as `x and (y or z)`. \n\
                    As of 0.20, it parses as `(x and y) or z`, matching other languages. \n\
                \n\n"
                    .to_string();
                msg.push_str(&source_lines(source, or_term.offset(), 0));
                Diagnostic::Warning(msg)
            })
            .collect()
    }
}

impl<'kb> Visitor for AndOrPrecendenceCheck<'kb> {
    fn visit_operation(&mut self, o: &Operation) {
        if (o.operator == Operator::And || o.operator == Operator::Or) && o.args.len() > 1 {
            for term in o.args.iter().filter(|t| {
                // find all inner expressions that are AND/OR terms where the outer
                // term is OR/AND respectively
                matches!(t.value(),
                    Value::Expression(op) if
                        (op.operator == Operator::Or || op.operator == Operator::And)
                        && op.operator != o.operator
                )
            }) {
                let span = term.span().unwrap();
                let source = self.kb.get_term_source(term).unwrap();

                // check if source _before_ the term contains an opening
                // parenthesis
                if !source.src[..span.0].trim().ends_with('(') {
                    self.unparenthesized_expr.push((source, term.clone()));
                }
            }
        }
        crate::visitor::walk_operation(self, o)
    }
}

pub fn check_ambiguous_precedence(rule: &Rule, kb: &KnowledgeBase) -> Vec<Diagnostic> {
    let mut visitor = AndOrPrecendenceCheck::new(kb);
    walk_rule(&mut visitor, rule);
    visitor.warnings()
}

pub fn check_no_allow_rule(kb: &KnowledgeBase) -> Option<Diagnostic> {
    let has_allow = kb.get_rules().contains_key(&sym!("allow"));
    let has_allow_field = kb.get_rules().contains_key(&sym!("allow_field"));
    let has_allow_request = kb.get_rules().contains_key(&sym!("allow_request"));
    if has_allow || has_allow_field || has_allow_request {
        None
    } else {
        Some(Diagnostic::Warning(
            "Your policy does not contain an allow rule, which usually means \
that no actions are allowed. Did you mean to add an allow rule to \
the top of your policy?

  allow(actor, action, resource) if ...

You can also suppress this warning by adding an allow_field or allow_request \
rule. For more information about allow rules, see:

  https://docs.osohq.com/reference/polar/builtin_rule_types.html#allow"
                .to_string(),
        ))
    }
}

struct ResourceBlocksMissingHasPermissionVisitor {
    calls_has_permission: bool,
}

impl Visitor for ResourceBlocksMissingHasPermissionVisitor {
    fn visit_call(&mut self, call: &Call) {
        if call.name.0 == "has_permission" {
            self.calls_has_permission = true;
        }
        walk_call(self, call)
    }
}

impl ResourceBlocksMissingHasPermissionVisitor {
    fn new() -> Self {
        Self {
            calls_has_permission: false,
        }
    }

    fn warnings(&mut self) -> Option<Diagnostic> {
        if !self.calls_has_permission {
            return Some(Diagnostic::Warning("Warning: your policy uses resource blocks but does not call the \
has_permission rule. This means that permissions you define in a \
resource block will not have any effect. Did you mean to include a \
call to has_permission in a top-level allow rule?

  allow(actor, action, resource) if
      has_permission(actor, action, resource);

For more information about resource blocks, see https://docs.osohq.com/any/reference/polar/polar-syntax.html#actor-and-resource-blocks".to_string()
            ));
        }
        None
    }
}

pub fn check_resource_blocks_missing_has_permission(kb: &KnowledgeBase) -> Option<Diagnostic> {
    if kb.resource_blocks.resources.is_empty() {
        return None;
    }

    let mut visitor = ResourceBlocksMissingHasPermissionVisitor::new();
    for rule in kb.get_rules().values() {
        visitor.visit_generic_rule(rule);
    }
    visitor.warnings()
}

struct UndefinedRuleCallVisitor<'kb> {
    call_terms: Vec<Term>,
    defined_rules: HashSet<&'kb Symbol>,
}

impl<'kb> UndefinedRuleCallVisitor<'kb> {
    fn new(defined_rules: HashSet<&'kb Symbol>) -> Self {
        Self {
            defined_rules,
            call_terms: Vec::new(),
        }
    }

    fn errors(self) -> Vec<Diagnostic> {
        self.call_terms
            .into_iter()
            .filter(|term| {
                let call = term.value().as_call().unwrap();
                !self.defined_rules.contains(&call.name)
            })
            .map(|term| Diagnostic::Error(ValidationError::UndefinedRuleCall { term }.into()))
            .collect()
    }
}

impl<'kb> Visitor for UndefinedRuleCallVisitor<'kb> {
    fn visit_term(&mut self, term: &Term) {
        match term.value() {
            Value::Expression(op) => {
                if op.operator == Operator::Dot || op.operator == Operator::New {
                    return;
                }
            }
            Value::Call(_) => self.call_terms.push(term.clone()),
            _ => {}
        }
        walk_term(self, term)
    }
}

pub fn check_undefined_rule_calls(kb: &KnowledgeBase) -> Vec<Diagnostic> {
    let mut visitor = UndefinedRuleCallVisitor::new(kb.get_rules().keys().collect());
    for rule in kb.get_rules().values() {
        visitor.visit_generic_rule(rule);
    }
    visitor.errors()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kb::KnowledgeBase;

    #[test]
    fn test_check_no_allow_rule_no_allow() {
        let mut kb = KnowledgeBase::new();
        kb.add_rule(rule!("f", [sym!("x")]));
        kb.add_rule(rule!("g", [sym!("x")]));
        assert!(check_no_allow_rule(&kb).is_some());
    }

    #[test]
    fn test_check_no_allow_rule_with_allow() {
        let mut kb = KnowledgeBase::new();
        kb.add_rule(rule!("f", [sym!("x")]));
        kb.add_rule(rule!(
            "allow",
            [sym!("actor"), sym!("action"), sym!("resource")]
        ));
        kb.add_rule(rule!("g", [sym!("x")]));
        assert!(check_no_allow_rule(&kb).is_none());
    }

    #[test]
    fn test_check_no_allow_rule_with_allow_field() {
        let mut kb = KnowledgeBase::new();
        kb.add_rule(rule!("f", [sym!("x")]));
        kb.add_rule(rule!(
            "allow_field",
            [
                sym!("actor"),
                sym!("action"),
                sym!("resource"),
                sym!("field")
            ]
        ));
        kb.add_rule(rule!("g", [sym!("x")]));
        assert!(check_no_allow_rule(&kb).is_none());
    }

    #[test]
    fn test_check_no_allow_rule_with_allow_request() {
        let mut kb = KnowledgeBase::new();
        kb.add_rule(rule!("f", [sym!("x")]));
        kb.add_rule(rule!("allow_request", [sym!("actor"), sym!("request")]));
        kb.add_rule(rule!("g", [sym!("x")]));
        assert!(check_no_allow_rule(&kb).is_none());
    }

    #[test]
    fn test_check_resource_blocks_missing_has_permission_warning() {
        let mut kb = KnowledgeBase::new();
        kb.resource_blocks
            .resources
            .insert(term!(sym!("Organization")));
        assert!(check_resource_blocks_missing_has_permission(&kb).is_some());
    }

    #[test]
    fn test_check_resource_blocks_missing_has_permission_clean() {
        let mut kb = KnowledgeBase::new();
        kb.resource_blocks
            .resources
            .insert(term!(sym!("Organization")));
        kb.add_rule(rule!("f", [sym!("x")] => call!("has_permission", [sym!("y")])));
        assert!(check_resource_blocks_missing_has_permission(&kb).is_none());
    }

    #[test]
    fn test_undefined_rule_error() {
        let mut kb = KnowledgeBase::new();
        kb.add_rule(rule!("f", [sym!("x")] => call!("no_such_rule", [sym!("y")])));
        let errors = check_undefined_rule_calls(&kb);
        assert_eq!(errors.len(), 1);
        assert!(format!("{}", errors.first().unwrap())
            .contains("Call to undefined rule: no_such_rule(y)"));
    }

    #[test]
    fn test_undefined_rule_error_clean() {
        let mut kb = KnowledgeBase::new();
        kb.add_rule(rule!("f", [sym!("x")] => call!("defined_rule", [sym!("y")])));
        kb.add_rule(rule!("defined_rule", [sym!("x")]));
        assert!(check_undefined_rule_calls(&kb).is_empty());
    }
}
