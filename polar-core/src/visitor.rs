//! Inspiration:
//! - https://github.com/rust-unofficial/patterns/blob/607fcb00c4ecb9c6317e4e101e16dc15717758bd/patterns/visitor.md
//! - https://docs.rs/rustc-ap-syntax/645.0.0/src/rustc_ap_syntax/visit.rs.html
//!
//! Paraphrasing the above, this is an AST walker. Each overridden visit method has full control
//! over what happens with its node: it can do its own traversal of the node's children, call
//! `visitor::walk_*` to apply the default traversal algorithm, or prevent deeper traversal by
//! doing nothing.

use crate::rules::*;
use crate::terms::*;

/// Paraphrasing from https://docs.rs/rustc-ap-syntax/71.0.0/src/syntax/fold.rs.html:
///
/// Any additions to this trait should happen in form of a call to a public `walk_*` function that
/// only calls out to the visitor again, not other `walk_*` functions. This is a necessary API
/// workaround to the problem of not being able to call out to the super default method in an
/// overridden default method.
///
/// Paraphrasing from https://docs.rs/rustc-ap-syntax/645.0.0/src/rustc_ap_syntax/visit.rs.html:
///
/// Each method of the Visitor trait is a hook to be potentially overridden. Each method's default
/// implementation recursively visits the substructure of the input via the corresponding `walk_*`
/// method; e.g., the `visit_rule` method by default calls `visitor::walk_rule`.
pub trait Visitor: Sized {
    // Atoms. These may be overridden as needed.
    fn visit_number(&mut self, _n: &Numeric) {}
    fn visit_string(&mut self, _s: &str) {}
    fn visit_boolean(&mut self, _b: &bool) {}
    fn visit_instance_id(&mut self, _i: &u64) {}
    fn visit_symbol(&mut self, _s: &Symbol) {}
    fn visit_variable(&mut self, _v: &Symbol) {}
    fn visit_rest_variable(&mut self, _r: &Symbol) {}
    fn visit_operator(&mut self, _o: &Operator) {}

    // Compounds. If you override these, you must walk the children manually.
    fn visit_generic_rule(&mut self, rule: &GenericRule) {
        walk_generic_rule(self, rule);
    }
    fn visit_rule(&mut self, r: &Rule) {
        walk_rule(self, r)
    }
    fn visit_term(&mut self, t: &Term) {
        walk_term(self, t)
    }
    fn visit_field(&mut self, k: &Symbol, v: &Term) {
        walk_field(self, k, v)
    }
    fn visit_external_instance(&mut self, e: &ExternalInstance) {
        walk_external_instance(self, e)
    }
    fn visit_instance_literal(&mut self, i: &InstanceLiteral) {
        walk_instance_literal(self, i)
    }
    fn visit_dictionary(&mut self, d: &Dictionary) {
        walk_dictionary(self, d)
    }
    fn visit_pattern(&mut self, p: &Pattern) {
        walk_pattern(self, p)
    }
    fn visit_call(&mut self, c: &Call) {
        walk_call(self, c)
    }
    #[allow(clippy::ptr_arg)]
    fn visit_list(&mut self, l: &TermList) {
        walk_list(self, l)
    }
    fn visit_operation(&mut self, o: &Operation) {
        walk_operation(self, o)
    }
    fn visit_param(&mut self, p: &Parameter) {
        walk_param(self, p)
    }
}

macro_rules! walk_elements {
    ($visitor: expr, $method: ident, $list: expr) => {
        for element in $list {
            $visitor.$method(element)
        }
    };
}

macro_rules! walk_fields {
    ($visitor: expr, $dict: expr) => {
        for (k, v) in $dict {
            $visitor.visit_field(k, v)
        }
    };
}

pub fn walk_generic_rule<V: Visitor>(visitor: &mut V, rule: &GenericRule) {
    for rule in rule.rules.values() {
        visitor.visit_rule(rule);
    }
}

pub fn walk_rule<V: Visitor>(visitor: &mut V, rule: &Rule) {
    visitor.visit_symbol(&rule.name);
    walk_elements!(visitor, visit_param, &rule.params);
    visitor.visit_term(&rule.body);
}

pub fn walk_term<V: Visitor>(visitor: &mut V, term: &Term) {
    match term.value() {
        Value::Number(n) => visitor.visit_number(n),
        Value::String(s) => visitor.visit_string(s),
        Value::Boolean(b) => visitor.visit_boolean(b),
        Value::ExternalInstance(e) => visitor.visit_external_instance(e),
        Value::Dictionary(d) => visitor.visit_dictionary(d),
        Value::Pattern(p) => visitor.visit_pattern(p),
        Value::Call(c) => visitor.visit_call(c),
        Value::List(l) => visitor.visit_list(l),
        Value::Variable(v) => visitor.visit_variable(v),
        Value::RestVariable(r) => visitor.visit_rest_variable(r),
        Value::Expression(o) => visitor.visit_operation(o),
    }
}

pub fn walk_field<V: Visitor>(visitor: &mut V, key: &Symbol, value: &Term) {
    visitor.visit_symbol(key);
    visitor.visit_term(value);
}

pub fn walk_external_instance<V: Visitor>(visitor: &mut V, instance: &ExternalInstance) {
    visitor.visit_instance_id(&instance.instance_id);
}

pub fn walk_instance_literal<V: Visitor>(visitor: &mut V, instance: &InstanceLiteral) {
    visitor.visit_symbol(&instance.tag);
    visitor.visit_dictionary(&instance.fields);
}

pub fn walk_dictionary<V: Visitor>(visitor: &mut V, dict: &Dictionary) {
    walk_fields!(visitor, &dict.fields);
}

pub fn walk_pattern<V: Visitor>(visitor: &mut V, pattern: &Pattern) {
    match pattern {
        Pattern::Dictionary(dict) => visitor.visit_dictionary(dict),
        Pattern::Instance(instance) => visitor.visit_instance_literal(instance),
    }
}

pub fn walk_call<V: Visitor>(visitor: &mut V, call: &Call) {
    visitor.visit_symbol(&call.name);
    walk_elements!(visitor, visit_term, &call.args);
    if let Some(kwargs) = call.kwargs.as_ref() {
        walk_fields!(visitor, kwargs);
    }
}

#[allow(clippy::ptr_arg)]
pub fn walk_list<V: Visitor>(visitor: &mut V, list: &TermList) {
    walk_elements!(visitor, visit_term, list);
}

pub fn walk_operation<V: Visitor>(visitor: &mut V, expr: &Operation) {
    visitor.visit_operator(&expr.operator);
    walk_elements!(visitor, visit_term, &expr.args);
}

pub fn walk_param<V: Visitor>(visitor: &mut V, param: &Parameter) {
    visitor.visit_term(&param.parameter);
    if let Some(ref specializer) = param.specializer {
        visitor.visit_term(specializer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestVisitor {
        visited: Vec<Value>,
    }

    impl TestVisitor {
        fn new() -> Self {
            Self { visited: vec![] }
        }
        fn push(&mut self, value: Value) {
            self.visited.push(value);
        }
    }

    impl Visitor for TestVisitor {
        fn visit_number(&mut self, n: &Numeric) {
            self.push(Value::Number(*n));
        }
        fn visit_string(&mut self, s: &str) {
            self.push(Value::String(s.to_string()));
        }
        fn visit_boolean(&mut self, b: &bool) {
            self.push(Value::Boolean(*b));
        }
        fn visit_instance_id(&mut self, i: &u64) {
            self.push(Value::Number(Numeric::Integer(*i as i64)));
        }
        fn visit_symbol(&mut self, s: &Symbol) {
            self.push(Value::Variable(s.clone()));
        }
        fn visit_variable(&mut self, v: &Symbol) {
            self.push(Value::Variable(v.clone()));
        }
        fn visit_rest_variable(&mut self, r: &Symbol) {
            self.push(Value::RestVariable(r.clone()));
        }
        fn visit_operator(&mut self, o: &Operator) {
            self.push(Value::Expression(Operation {
                operator: *o,
                args: vec![],
            }));
        }
    }

    #[test]
    fn test_walk_term_atomics() {
        let number = value!(1);
        let string = value!("Hi there!");
        let boolean = value!(true);
        let variable = value!(sym!("x"));
        let rest_var = Value::RestVariable(sym!("rest"));
        let list = Value::List(vec![
            term!(number.clone()),
            term!(string.clone()),
            term!(boolean.clone()),
            term!(variable.clone()),
            term!(rest_var.clone()),
        ]);
        let term = term!(list);
        let mut v = TestVisitor::new();
        v.visit_term(&term);
        assert_eq!(v.visited, vec![number, string, boolean, variable, rest_var]);
    }

    #[test]
    fn test_walk_term_compounds() {
        let external_instance = term!(Value::ExternalInstance(ExternalInstance {
            instance_id: 1,
            constructor: None,
            repr: None,
            class_repr: None,
            class_id: None,
        }));
        let instance_pattern = term!(value!(Pattern::Instance(InstanceLiteral {
            tag: sym!("d"),
            fields: Dictionary {
                fields: btreemap! {
                    sym!("e") => term!(call!("f", [2])),
                    sym!("g") => term!(op!(Add, term!(3), term!(4))),
                }
            }
        })));
        let dict_pattern = term!(Value::Pattern(Pattern::Dictionary(Dictionary {
            fields: btreemap! {
                sym!("i") => term!("j"),
                sym!("k") => term!("l"),
            },
        })));
        let term = term!(btreemap! {
            sym!("a") => term!(btreemap!{
                sym!("b") => external_instance,
                sym!("c") => instance_pattern,
            }),
            sym!("h") => dict_pattern,
        });
        let mut v = TestVisitor::new();
        v.visit_term(&term);
        assert_eq!(
            v.visited,
            vec![
                value!(sym!("a")),
                value!(sym!("b")),
                value!(1),
                value!(sym!("c")),
                value!(sym!("d")),
                value!(sym!("e")),
                value!(sym!("f")),
                value!(2),
                value!(sym!("g")),
                value!(op!(Add)),
                value!(3),
                value!(4),
                value!(sym!("h")),
                value!(sym!("i")),
                value!("j"),
                value!(sym!("k")),
                value!("l"),
            ]
        );
    }

    #[test]
    fn test_walk_rule() {
        let rule = rule!("a", ["b"; instance!("c"), value!("d")] => call!("e", [value!("f")]));
        let mut v = TestVisitor::new();
        v.visit_rule(&rule);
        assert_eq!(
            v.visited,
            vec![
                value!(sym!("a")),
                value!(sym!("b")),
                value!(sym!("c")),
                value!("d"),
                value!(op!(And)),
                value!(sym!("e")),
                value!("f"),
            ]
        );
    }
}
