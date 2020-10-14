use super::partial::Constraints;
use super::rules::*;
use super::terms::*;

pub trait Visitor<'term>: Sized {
    // Atoms. These may be overridden as needed.
    fn visit_number(&mut self, _n: &'term Numeric) {}
    fn visit_string(&mut self, _s: &'term str) {}
    fn visit_boolean(&mut self, _b: &'term bool) {}
    fn visit_id(&mut self, _i: &'term u64) {}
    fn visit_name(&mut self, _n: &'term Symbol) {}
    fn visit_variable(&mut self, _v: &'term Symbol) {}
    fn visit_rest_variable(&mut self, _r: &'term Symbol) {}
    fn visit_operator(&mut self, _o: &'term Operator) {}

    // Compounds. If you override these, you must walk the children manually.
    fn visit_rule(&mut self, r: &'term Rule) {
        walk_rule(self, r)
    }
    fn visit_term(&mut self, t: &'term Term) {
        walk_term(self, t)
    }
    fn visit_field(&mut self, k: &'term Symbol, v: &'term Term) {
        walk_field(self, k, v)
    }
    fn visit_external_instance(&mut self, e: &'term ExternalInstance) {
        walk_external_instance(self, e)
    }
    fn visit_instance_literal(&mut self, i: &'term InstanceLiteral) {
        walk_instance_literal(self, i)
    }
    fn visit_dictionary(&mut self, d: &'term Dictionary) {
        walk_dictionary(self, d)
    }
    fn visit_pattern(&mut self, p: &'term Pattern) {
        walk_pattern(self, p)
    }
    fn visit_call(&mut self, c: &'term Call) {
        walk_call(self, c)
    }
    #[allow(clippy::ptr_arg)]
    fn visit_list(&mut self, l: &'term TermList) {
        walk_list(self, l)
    }
    fn visit_operation(&mut self, o: &'term Operation) {
        walk_operation(self, o)
    }
    fn visit_param(&mut self, p: &'term Parameter) {
        walk_param(self, p)
    }
    fn visit_partial(&mut self, c: &'term Constraints) {
        walk_partial(self, c)
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
    ($visitor: expr, $method: ident, $dict: expr) => {
        for (k, v) in $dict {
            $visitor.$method(k, v)
        }
    };
}

pub fn walk_rule<'a, V: Visitor<'a>>(visitor: &mut V, rule: &'a Rule) {
    visitor.visit_name(&rule.name);
    walk_elements!(visitor, visit_param, &rule.params);
    visitor.visit_term(&rule.body);
}

pub fn walk_term<'a, V: Visitor<'a>>(visitor: &mut V, term: &'a Term) {
    match term.value() {
        Value::Number(n) => visitor.visit_number(n),
        Value::String(s) => visitor.visit_string(s),
        Value::Boolean(b) => visitor.visit_boolean(b),
        Value::ExternalInstance(e) => visitor.visit_external_instance(e),
        Value::InstanceLiteral(i) => visitor.visit_instance_literal(i),
        Value::Dictionary(d) => visitor.visit_dictionary(d),
        Value::Pattern(p) => visitor.visit_pattern(p),
        Value::Call(c) => visitor.visit_call(c),
        Value::List(l) => visitor.visit_list(l),
        Value::Variable(v) => visitor.visit_variable(v),
        Value::RestVariable(r) => visitor.visit_rest_variable(r),
        Value::Expression(o) => visitor.visit_operation(o),
        Value::Partial(p) => visitor.visit_partial(p),
    }
}

pub fn walk_field<'a, V: Visitor<'a>>(visitor: &mut V, key: &'a Symbol, value: &'a Term) {
    visitor.visit_name(key);
    visitor.visit_term(value);
}

pub fn walk_external_instance<'a, V: Visitor<'a>>(visitor: &mut V, instance: &'a ExternalInstance) {
    visitor.visit_id(&instance.instance_id);
}

pub fn walk_instance_literal<'a, V: Visitor<'a>>(visitor: &mut V, instance: &'a InstanceLiteral) {
    visitor.visit_name(&instance.tag);
    walk_fields!(visitor, visit_field, &instance.fields.fields);
}

pub fn walk_dictionary<'a, V: Visitor<'a>>(visitor: &mut V, dict: &'a Dictionary) {
    walk_fields!(visitor, visit_field, &dict.fields);
}

pub fn walk_pattern<'a, V: Visitor<'a>>(visitor: &mut V, pattern: &'a Pattern) {
    match pattern {
        Pattern::Dictionary(dict) => walk_fields!(visitor, visit_field, &dict.fields),
        Pattern::Instance(instance) => visitor.visit_instance_literal(&instance),
    }
}

pub fn walk_call<'a, V: Visitor<'a>>(visitor: &mut V, call: &'a Call) {
    visitor.visit_name(&call.name);
    walk_elements!(visitor, visit_term, &call.args);
}

#[allow(clippy::ptr_arg)]
pub fn walk_list<'a, V: Visitor<'a>>(visitor: &mut V, list: &'a TermList) {
    walk_elements!(visitor, visit_term, list);
}

pub fn walk_operation<'a, V: Visitor<'a>>(visitor: &mut V, expr: &'a Operation) {
    visitor.visit_operator(&expr.operator);
    walk_elements!(visitor, visit_term, &expr.args);
}

pub fn walk_param<'a, V: Visitor<'a>>(visitor: &mut V, param: &'a Parameter) {
    visitor.visit_term(&param.parameter);
    if let Some(ref specializer) = param.specializer {
        visitor.visit_term(specializer);
    }
}

pub fn walk_partial<'a, V: Visitor<'a>>(visitor: &mut V, partial: &'a Constraints) {
    visitor.visit_name(&partial.variable);
    walk_elements!(visitor, visit_operation, &partial.operations);
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

    impl<'term> Visitor<'term> for TestVisitor {
        fn visit_number(&mut self, n: &'term Numeric) {
            self.push(Value::Number(*n));
        }
        fn visit_string(&mut self, s: &'term str) {
            self.push(Value::String(s.to_string()));
        }
        fn visit_boolean(&mut self, b: &'term bool) {
            self.push(Value::Boolean(*b));
        }
        fn visit_id(&mut self, i: &'term u64) {
            self.push(Value::Number(Numeric::Integer(*i as i64)));
        }
        fn visit_name(&mut self, n: &'term Symbol) {
            self.push(Value::Variable(n.clone()));
        }
        fn visit_variable(&mut self, v: &'term Symbol) {
            self.push(Value::Variable(v.clone()));
        }
        fn visit_rest_variable(&mut self, r: &'term Symbol) {
            self.push(Value::RestVariable(r.clone()));
        }
        fn visit_operator(&mut self, o: &'term Operator) {
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

    // TODO(gj): Add test for walking a partial.
}
