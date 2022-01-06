use std::collections::{HashMap, HashSet};

use super::diagnostic::Diagnostic;
use super::error::{PolarError, ValidationError};
use super::kb::*;
use super::rules::*;
use super::terms::*;
use super::visitor::{walk_call, walk_rule, walk_term, Visitor};
use super::warning::ValidationWarning;

/// Record singleton variables and unknown specializers in a rule.
struct SingletonVisitor<'kb> {
    kb: &'kb KnowledgeBase,
    singletons: HashMap<Symbol, Option<Term>>,
}

impl<'kb> SingletonVisitor<'kb> {
    fn new(kb: &'kb KnowledgeBase) -> Self {
        Self {
            kb,
            singletons: HashMap::new(),
        }
    }

    fn warnings(self) -> Vec<Diagnostic> {
        let mut singletons = self
            .singletons
            .into_iter()
            .flat_map(|(sym, term)| term.map(|t| (sym, t)))
            .collect::<Vec<_>>();
        singletons.sort_by_key(|(_, term)| term.parsed_context().map_or(0, |context| context.left));
        singletons
            .into_iter()
            .map(|(sym, term)| {
                if let Value::Pattern(_) = term.value() {
                    Diagnostic::Warning(ValidationWarning::UnknownSpecializer { term, sym }.into())
                } else {
                    Diagnostic::Error(ValidationError::SingletonVariable { term }.into())
                }
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

struct AndOrPrecendenceCheck {
    unparenthesized_expr: Vec<Term>,
}

impl AndOrPrecendenceCheck {
    fn new() -> Self {
        Self {
            unparenthesized_expr: Default::default(),
        }
    }

    fn warnings(self) -> Vec<Diagnostic> {
        self.unparenthesized_expr
            .into_iter()
            .map(|term| Diagnostic::Warning(ValidationWarning::AmbiguousPrecedence { term }.into()))
            .collect()
    }
}

impl Visitor for AndOrPrecendenceCheck {
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
                // TODO(gj): are these `.unwrap()`s chill?
                let context = term.parsed_context().unwrap();

                // TODO(gj): is this unchecked indexing operation chill?
                //
                // check if source _before_ the term contains an opening
                // parenthesis
                if !context.source.src[..context.left].trim().ends_with('(') {
                    self.unparenthesized_expr.push(term.clone());
                }
            }
        }
        crate::visitor::walk_operation(self, o)
    }
}

pub fn check_ambiguous_precedence(rule: &Rule) -> Vec<Diagnostic> {
    let mut visitor = AndOrPrecendenceCheck::new();
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
            ValidationWarning::MissingAllowRule.into(),
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

    fn warnings(&mut self) -> Option<ValidationWarning> {
        if !self.calls_has_permission {
            return Some(ValidationWarning::MissingHasPermissionRule);
        }
        None
    }
}

pub fn check_resource_blocks_missing_has_permission(
    kb: &KnowledgeBase,
) -> Option<ValidationWarning> {
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
                term.value()
                    .as_call()
                    .map_or(false, |call| !self.defined_rules.contains(&call.name))
            })
            .map(|term| PolarError::from(ValidationError::UndefinedRuleCall { term }).into())
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
