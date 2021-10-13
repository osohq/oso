use super::error::*;
use super::formatting::source_lines;
use super::kb::*;
use super::rules::*;
use super::sources::Source;
use super::terms::*;
use super::visitor::{walk_call, walk_rule, walk_term, Visitor};

use std::collections::HashSet;
use std::collections::{hash_map::Entry, HashMap};
use std::iter::FromIterator;

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

fn warn_str(sym: &Symbol, term: &Term, source: &Option<Source>) -> PolarResult<String> {
    if let Value::Pattern(..) = term.value() {
        let mut msg = format!("Unknown specializer {}", sym);
        if let Some(t) = common_misspellings(&sym.0) {
            msg.push_str(&format!(", did you mean {}?", t));
        }
        Ok(msg)
    } else {
        let perr = error::ParseError::SingletonVariable {
            loc: term.offset(),
            name: sym.0.clone(),
        };
        let err = error::PolarError {
            kind: error::ErrorKind::Parse(perr),
            context: None,
        };

        let src = if let Some(ref s) = source {
            Some(s)
        } else {
            None
        };
        Err(err.set_context(src, Some(term)))
    }
}

impl<'kb> SingletonVisitor<'kb> {
    fn new(kb: &'kb KnowledgeBase) -> Self {
        Self {
            kb,
            singletons: HashMap::new(),
        }
    }

    fn warnings(&mut self) -> PolarResult<Vec<String>> {
        let mut singletons = self
            .singletons
            .drain()
            .filter_map(|(sym, singleton)| singleton.map(|term| (sym.clone(), term)))
            .collect::<Vec<(Symbol, Term)>>();
        singletons.sort_by_key(|(_sym, term)| term.offset());
        singletons
            .iter()
            .map(|(sym, term)| {
                let src = term
                    .get_source_id()
                    .and_then(|id| self.kb.sources.get_source(id));
                let mut msg = warn_str(sym, term, &src)?;
                if let Some(ref source) = src {
                    msg.push('\n');
                    msg.push_str(&source_lines(source, term.offset(), 0));
                }
                Ok(msg)
            })
            .collect::<PolarResult<Vec<String>>>()
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
                match self.singletons.entry(v.clone()) {
                    Entry::Occupied(mut o) => {
                        o.insert(None);
                    }
                    Entry::Vacant(v) => {
                        v.insert(Some(t.clone()));
                    }
                }
            }
            _ => (),
        }
        walk_term(self, t);
    }
}

pub fn check_singletons(rule: &Rule, kb: &KnowledgeBase) -> PolarResult<Vec<String>> {
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

    fn warnings(&mut self) -> Vec<String> {
        self.unparenthesized_expr
            .iter()
            .map(|(source, or_term)| {
                let mut msg = "Expression without parentheses could be ambiguous. \n\
                    Prior to 0.20, `x and y or z` would parse as `x and (y or z)`. \n\
                    As of 0.20, it parses as `(x and y) or z`, matching other languages. \n\
                \n\n"
                    .to_string();
                msg.push_str(&source_lines(source, or_term.offset(), 0));
                msg
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
                let source = term
                    .get_source_id()
                    .and_then(|src_id| self.kb.sources.get_source(src_id))
                    .unwrap();

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

pub fn check_ambiguous_precedence(rule: &Rule, kb: &KnowledgeBase) -> Vec<String> {
    let mut visitor = AndOrPrecendenceCheck::new(kb);
    walk_rule(&mut visitor, rule);
    visitor.warnings()
}

pub fn check_no_allow_rule(kb: &KnowledgeBase) -> Vec<String> {
    let has_allow = kb.get_rules().contains_key(&sym!("allow"));
    let has_allow_field = kb.get_rules().contains_key(&sym!("allow_field"));
    let has_allow_request = kb.get_rules().contains_key(&sym!("allow_request"));
    if has_allow || has_allow_field || has_allow_request {
        vec![]
    } else {
        vec![
            "Your policy does not contain an allow rule, which usually means \
that no actions are allowed. Did you mean to add an allow rule to \
the top of your policy?

  allow(actor, action, resource) if ...

You can also suppress this warning by adding an allow_field or allow_request \
rule. For more information about allow rules, see:

  https://docs.osohq.com/reference/polar/builtin_rule_types.html#allow"
                .to_string(),
        ]
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

    fn warnings(&mut self) -> Vec<String> {
        if !self.calls_has_permission {
            return vec!["Warning: your policy uses resource blocks but does not call the \
has_permission rule. This means that permissions you define in a \
resource block will not have any effect. Did you mean to include a \
call to has_permission in a top-level allow rule?

  allow(actor, action, resource) if
      has_permission(actor, action, resource);

For more information about resource blocks, see https://docs.osohq.com/any/reference/polar/polar-syntax.html#actor-and-resource-blocks".to_string(),

            ];
        }
        vec![]
    }
}

pub fn check_resource_blocks_missing_has_permission(kb: &KnowledgeBase) -> Vec<String> {
    if kb.resource_blocks.resources.is_empty() {
        return vec![];
    }

    let mut visitor = ResourceBlocksMissingHasPermissionVisitor::new();
    for rule in kb.get_rules().values() {
        visitor.visit_generic_rule(rule);
    }
    visitor.warnings()
}

struct UndefinedRuleVisitor<'kb> {
    kb: &'kb KnowledgeBase,
    call_terms: Vec<Term>,
    defined_rules: HashSet<&'kb Symbol>,
}

impl<'kb> UndefinedRuleVisitor<'kb> {
    fn new(kb: &'kb KnowledgeBase, defined_rules: HashSet<&'kb Symbol>) -> Self {
        Self {
            kb,
            defined_rules,
            call_terms: Vec::new(),
        }
    }

    fn warnings(&mut self) -> Vec<PolarError> {
        let mut warnings = vec![];
        for term in &self.call_terms {
            let call = term.value().as_call().unwrap();
            if !self.defined_rules.contains(&call.name) {
                warnings.push(self.kb.set_error_context(
                    term,
                    error::ValidationError::UndefinedRule {
                        rule_name: call.name.0.clone(),
                    },
                ));
            }
        }
        warnings
    }
}

impl<'kb> Visitor for UndefinedRuleVisitor<'kb> {
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

pub fn check_undefined_rule_calls(kb: &KnowledgeBase) -> Vec<PolarError> {
    let mut visitor = UndefinedRuleVisitor::new(kb, HashSet::from_iter(kb.get_rules().keys()));
    for rule in kb.get_rules().values() {
        visitor.visit_generic_rule(rule);
    }

    visitor.warnings()
}

#[cfg(test)]
mod tests {
    use crate::kb::KnowledgeBase;
    use crate::rules::*;
    use crate::terms::*;
    use crate::warnings::{
        check_no_allow_rule, check_resource_blocks_missing_has_permission,
        check_undefined_rule_calls,
    };

    #[test]
    fn test_check_no_allow_rule_no_allow() {
        let mut kb = KnowledgeBase::new();
        kb.add_rule(rule!("f", [sym!("x")]));
        kb.add_rule(rule!("g", [sym!("x")]));
        assert_eq!(check_no_allow_rule(&kb).len(), 1);
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
        assert_eq!(check_no_allow_rule(&kb).len(), 0);
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
        assert_eq!(check_no_allow_rule(&kb).len(), 0);
    }

    #[test]
    fn test_check_no_allow_rule_with_allow_request() {
        let mut kb = KnowledgeBase::new();
        kb.add_rule(rule!("f", [sym!("x")]));
        kb.add_rule(rule!("allow_request", [sym!("actor"), sym!("request")]));
        kb.add_rule(rule!("g", [sym!("x")]));
        assert_eq!(check_no_allow_rule(&kb).len(), 0);
    }

    #[test]
    fn test_resource_missing_has_permission_warning() {
        let mut kb = KnowledgeBase::new();
        kb.resource_blocks
            .resources
            .insert(term!(sym!("Organization")));
        assert_eq!(check_resource_blocks_missing_has_permission(&kb).len(), 1);
    }

    #[test]
    fn test_resource_missing_has_permission_clean() {
        let mut kb = KnowledgeBase::new();
        kb.resource_blocks
            .resources
            .insert(term!(sym!("Organization")));
        kb.add_rule(rule!("f", [sym!("x")] => call!("has_permission", [sym!("y")])));
        let warnings = check_resource_blocks_missing_has_permission(&kb);

        assert_eq!(warnings.len(), 0);
    }

    #[test]
    fn test_undefined_rule_warning() {
        let mut kb = KnowledgeBase::new();
        kb.add_rule(rule!("f", [sym!("x")] => call!("no_such_rule", [sym!("y")])));
        let warnings = check_undefined_rule_calls(&kb);
        assert_eq!(warnings.len(), 1);

        assert!(format!("{}", warnings.first().unwrap())
            .contains(r#"Call to undefined rule "no_such_rule""#));
    }

    #[test]
    fn test_undefined_rule_warning_clean() {
        let mut kb = KnowledgeBase::new();
        kb.add_rule(rule!("f", [sym!("x")] => call!("defined_rule", [sym!("y")])));
        kb.add_rule(rule!("defined_rule", [sym!("x")]));
        let warnings = check_undefined_rule_calls(&kb);
        assert_eq!(warnings.len(), 0);
    }
}
