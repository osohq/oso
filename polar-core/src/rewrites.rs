use std::collections::HashMap;

use super::folder::*;
use super::kb::*;
use super::rules::*;
use super::terms::*;

/// Rename each non-constant variable in a term or rule to a fresh variable.
pub struct Renamer<'kb> {
    kb: &'kb KnowledgeBase,
    renames: HashMap<Symbol, Symbol>,
}

impl<'kb> Renamer<'kb> {
    pub fn new(kb: &'kb KnowledgeBase) -> Self {
        Self {
            kb,
            renames: HashMap::new(),
        }
    }
}

impl<'kb> Folder for Renamer<'kb> {
    fn fold_variable(&mut self, v: Symbol) -> Symbol {
        if self.kb.is_constant(&v) {
            v
        } else if let Some(w) = self.renames.get(&v) {
            w.clone()
        } else {
            let w = self.kb.gensym(&v.0);
            self.renames.insert(v, w.clone());
            w
        }
    }

    fn fold_rest_variable(&mut self, r: Symbol) -> Symbol {
        if let Some(s) = self.renames.get(&r) {
            s.clone()
        } else {
            let s = self.kb.gensym(&r.0);
            self.renames.insert(r, s.clone());
            s
        }
    }
}

/// Rewrite expressions, etc.
pub struct Rewriter<'kb> {
    kb: &'kb KnowledgeBase,
    stack: Vec<Vec<Term>>,
}

impl<'kb> Rewriter<'kb> {
    pub fn new(kb: &'kb KnowledgeBase) -> Self {
        Self { kb, stack: vec![] }
    }

    /// Return true if the expression should be rewritten.
    fn needs_rewrite(&self, o: &Operation) -> bool {
        match o.operator {
            Operator::Add
            | Operator::Dot
            | Operator::Div
            | Operator::Mul
            | Operator::Sub
            | Operator::Mod
            | Operator::Rem
                if o.args.len() == 2 =>
            {
                true
            }
            Operator::New if o.args.len() == 1 => true,
            _ => false,
        }
    }
}

fn temp_name(o: &Operator) -> &'static str {
    match o {
        Operator::Add | Operator::Div | Operator::Mul | Operator::Sub => "op",
        Operator::Dot => "value",
        Operator::New => "instance",
        _ => "temp",
    }
}

/// Replace `o(a, b)` with `_c`, where `_c = o(a, b)`.
/// The lookup is hoisted to the nearest enclosing
/// conjunction, creating one if necessary.
impl<'kb> Folder for Rewriter<'kb> {
    /// Rewrite a rule, pushing expressions in the head into the body.
    fn fold_rule(&mut self, Rule { name, body, params }: Rule) -> Rule {
        let mut body = self.fold_term(body);

        self.stack.push(vec![]);
        let params = params.into_iter().map(|p| self.fold_param(p)).collect();
        let rewrites = self.stack.pop().unwrap();
        if !rewrites.is_empty() {
            let terms = unwrap_and(&body);
            body.replace_value(Value::Expression(Operation {
                operator: Operator::And,
                args: terms.into_iter().chain(rewrites).collect(),
            }));
        }
        Rule { name, params, body }
    }

    /// Rewrite an expression as a temp, and push a rewritten
    /// expression that binds the temp.
    fn fold_term(&mut self, t: Term) -> Term {
        match t.value() {
            _ if self.stack.is_empty() => {
                // If there is no containing conjunction, make one.
                self.stack.push(vec![]);
                let mut new = self.fold_term(t);
                let mut rewrites = self.stack.pop().unwrap();
                for rewrite in rewrites.drain(..).rev() {
                    and_wrap(&mut new, rewrite);
                }
                new
            }
            Value::Expression(o) if self.needs_rewrite(o) => {
                // Rewrite sub-expressions, then push a temp onto the args.
                let mut new = fold_operation(o.clone(), self);
                let temp = Value::Variable(self.kb.gensym(temp_name(&o.operator)));
                new.args.push(Term::new_temporary(temp.clone()));

                // Push the rewritten expression into the top stack frame.
                self.stack
                    .last_mut()
                    .unwrap()
                    .push(t.clone_with_value(Value::Expression(new)));

                // Return the temp.
                t.clone_with_value(temp)
            }
            _ => fold_term(t, self),
        }
    }

    fn fold_operation(&mut self, o: Operation) -> Operation {
        match o.operator {
            Operator::And | Operator::Or | Operator::Not => Operation {
                operator: fold_operator(o.operator, self),
                args: o
                    .args
                    .into_iter()
                    .map(|arg| {
                        self.stack.push(vec![]);
                        let mut arg = self.fold_term(arg);
                        let mut rewrites = self.stack.pop().unwrap();
                        for rewrite in rewrites.drain(..).rev() {
                            and_wrap(&mut arg, rewrite);
                        }
                        arg
                    })
                    .collect(),
            },
            _ => fold_operation(o, self),
        }
    }

    fn fold_rest_variable(&mut self, v: Symbol) -> Symbol {
        if v.0 == "_" {
            self.kb.gensym("_")
        } else {
            v
        }
    }

    fn fold_variable(&mut self, v: Symbol) -> Symbol {
        if v.0 == "_" {
            self.kb.gensym("_")
        } else {
            v
        }
    }
}

/// Replace the left value with And(right, left).
fn and_wrap(left: &mut Term, right: Term) {
    let new_value = Value::Expression(Operation {
        operator: Operator::And,
        args: vec![right, left.clone()],
    });
    left.replace_value(new_value);
}

/// Return a cloned list of arguments from And(*args).
pub fn unwrap_and(term: &Term) -> TermList {
    match term.value() {
        Value::Expression(Operation {
            operator: Operator::And,
            args,
        }) => args.clone(),
        _ => panic!("expected And, found {}", term.to_polar()),
    }
}

/// Rewrite a term.
pub fn rewrite_term(term: Term, kb: &mut KnowledgeBase) -> Term {
    let mut fld = Rewriter::new(kb);
    fld.fold_term(term)
}

/// Rewrite a rule.
pub fn rewrite_rule(rule: Rule, kb: &mut KnowledgeBase) -> Rule {
    let mut fld = Rewriter::new(kb);
    fld.fold_rule(rule)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatting::ToPolarString;

    // Re-defined here for convenience
    fn parse_query(src: &str) -> Term {
        crate::parser::parse_query(0, src).unwrap()
    }

    fn parse_rules(src: &str) -> Vec<Rule> {
        crate::parser::parse_rules(0, src).unwrap()
    }

    #[test]
    fn rewrite_anonymous_vars() {
        let mut kb = KnowledgeBase::new();
        let query = parse_query("[1, 2, 3] = [_, _, _]");
        assert_eq!(
            rewrite_term(query, &mut kb).to_polar(),
            "[1, 2, 3] = [_1, _2, _3]"
        );
    }

    #[test]
    fn rewrite_rules() {
        let mut kb = KnowledgeBase::new();
        let rules = parse_rules("f(a.b);");
        let rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a.b);");

        // First rewrite
        let rule = rewrite_rule(rule, &mut kb);
        assert_eq!(rule.to_polar(), "f(_value_1) if a.b = _value_1;");

        // Check we can parse the rules back again
        let again = parse_rules(&rule.to_polar());
        let again_rule = again[0].clone();
        assert_eq!(again_rule.to_polar(), rule.to_polar());

        // Chained lookups
        let rules = parse_rules("f(a.b.c);");
        let rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a.b.c);");
        let rule = rewrite_rule(rule, &mut kb);
        assert_eq!(
            rule.to_polar(),
            "f(_value_3) if a.b = _value_2 and _value_2.c = _value_3;"
        );
    }

    #[test]
    fn rewrite_nested_lookups() {
        let mut kb = KnowledgeBase::new();

        // Lookups with args
        let rules = parse_rules("f(a, c) if a.b(c);");
        let rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a, c) if a.b(c);");
        let rule = rewrite_rule(rule, &mut kb);
        assert_eq!(
            rule.to_polar(),
            "f(a, c) if a.b(c) = _value_1 and _value_1;"
        );

        // Nested lookups
        let rules = parse_rules("f(a, c, e) if a.b(c.d(e.f()));");
        let rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a, c, e) if a.b(c.d(e.f()));");
        let rule = rewrite_rule(rule, &mut kb);
        assert_eq!(
            rule.to_polar(),
            "f(a, c, e) if e.f() = _value_2 and c.d(_value_2) = _value_3 and a.b(_value_3) = _value_4 and _value_4;"
        );
    }

    #[test]
    fn rewrite_terms() {
        let mut kb = KnowledgeBase::new();
        let term = parse_query("x and a.b");
        assert_eq!(term.to_polar(), "x and a.b");
        assert_eq!(
            rewrite_term(term, &mut kb).to_polar(),
            "x and a.b = _value_1 and _value_1"
        );

        let query = parse_query("f(a.b().c)");
        assert_eq!(query.to_polar(), "f(a.b().c)");
        assert_eq!(
            rewrite_term(query, &mut kb).to_polar(),
            "a.b() = _value_2 and _value_2.c = _value_3 and f(_value_3)"
        );

        let term = parse_query("a.b = 1");
        assert_eq!(
            rewrite_term(term, &mut kb).to_polar(),
            "a.b = _value_4 and _value_4 = 1"
        );
        let term = parse_query("{x: 1}.x = 1");
        assert_eq!(term.to_polar(), "{x: 1}.x = 1");
        assert_eq!(
            rewrite_term(term, &mut kb).to_polar(),
            "{x: 1}.x = _value_5 and _value_5 = 1"
        );
    }

    #[test]
    fn rewrite_expressions() {
        let mut kb = KnowledgeBase::new();

        let term = parse_query("0 - 0 = 0");
        assert_eq!(term.to_polar(), "0 - 0 = 0");
        assert_eq!(
            rewrite_term(term, &mut kb).to_polar(),
            "0 - 0 = _op_1 and _op_1 = 0"
        );

        let rules = parse_rules("sum(a, b, a + b);");
        let rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "sum(a, b, a + b);");
        let rule = rewrite_rule(rule, &mut kb);
        assert_eq!(rule.to_polar(), "sum(a, b, _op_2) if a + b = _op_2;");

        let rules = parse_rules("fib(n, a+b) if fib(n-1, a) and fib(n-2, b);");
        let rule = rules[0].clone();
        let rule = rewrite_rule(rule, &mut kb);
        assert_eq!(rule.to_polar(), "fib(n, _op_5) if n - 1 = _op_3 and fib(_op_3, a) and n - 2 = _op_4 and fib(_op_4, b) and a + b = _op_5;");
    }

    #[test]
    fn rewrite_nested_literal() {
        let mut kb = KnowledgeBase::new();
        let term = parse_query("new Foo(x: bar.y)");
        assert_eq!(term.to_polar(), "new Foo(x: bar.y)");
        assert_eq!(
            rewrite_term(term, &mut kb).to_polar(),
            "bar.y = _value_1 and new (Foo(x: _value_1), _instance_2) and _instance_2"
        );

        let term = parse_query("f(new Foo(x: bar.y))");
        assert_eq!(term.to_polar(), "f(new Foo(x: bar.y))");
        assert_eq!(
            rewrite_term(term, &mut kb).to_polar(),
            "bar.y = _value_3 and new (Foo(x: _value_3), _instance_4) and f(_instance_4)"
        );
    }

    #[test]
    fn rewrite_class_constructor() {
        let mut kb = KnowledgeBase::new();
        let term = parse_query("new Foo(a: 1, b: 2)");
        assert_eq!(term.to_polar(), "new Foo(a: 1, b: 2)");

        // @ means external constructor
        assert_eq!(
            rewrite_term(term, &mut kb).to_polar(),
            "new (Foo(a: 1, b: 2), _instance_1) and _instance_1"
        );
    }

    #[test]
    fn rewrite_nested_class_constructor() {
        let mut kb = KnowledgeBase::new();
        let term = parse_query("new Foo(a: 1, b: new Foo(a: 2, b: 3))");
        assert_eq!(term.to_polar(), "new Foo(a: 1, b: new Foo(a: 2, b: 3))");

        assert_eq!(
            rewrite_term(term, &mut kb).to_polar(),
            "new (Foo(a: 2, b: 3), _instance_1) and \
             new (Foo(a: 1, b: _instance_1), _instance_2) and _instance_2"
        );
    }

    #[test]
    fn rewrite_rules_constructor() {
        let mut kb = KnowledgeBase::new();
        let mut rules = parse_rules("rule_test(new Foo(a: 1, b: 2));");
        let rule = rules.pop().unwrap();
        assert_eq!(rule.to_polar(), "rule_test(new Foo(a: 1, b: 2));");
        assert!(rules.is_empty());

        let rule = rewrite_rule(rule, &mut kb);
        assert_eq!(
            rule.to_polar(),
            "rule_test(_instance_1) if new (Foo(a: 1, b: 2), _instance_1);"
        )
    }

    #[test]
    fn rewrite_not_with_lookup() {
        let mut kb = KnowledgeBase::new();
        let term = parse_query("not foo.x = 1");
        assert_eq!(term.to_polar(), "not foo.x = 1");

        pretty_assertions::assert_eq!(
            rewrite_term(term, &mut kb).to_polar(),
            "not (foo.x = _value_1 and _value_1 = 1)"
        )
    }
}
