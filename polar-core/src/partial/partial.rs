use std::collections::HashSet;
//
// use serde::{Deserialize, Serialize};
//
use crate::folder::{fold_term, Folder};
use crate::formatting::ToPolarString;
use crate::terms::{Operation, Operator, Symbol, Term, Value};
use crate::visitor::{walk_operation, Visitor};

impl Operation {
    // Invariant: self is an AND
    pub fn variables(&self) -> HashSet<Symbol> {
        struct VariableVisitor {
            vars: HashSet<Symbol>,
        }

        impl Visitor for VariableVisitor {
            fn visit_variable(&mut self, v: &Symbol) {
                // TODO(gj): check that var is bound to partial or unbound.
                // TODO(gj): update above comment
                self.vars.insert(v.clone());
            }
        }

        let mut visitor = VariableVisitor {
            vars: HashSet::new(),
        };

        walk_operation(&mut visitor, &self);
        visitor.vars
    }

    pub fn ground(&self, var: Symbol, value: Term) -> Self {
        struct Grounder {
            var: Symbol,
            value: Term,
        }

        impl Folder for Grounder {
            fn fold_term(&mut self, t: Term) -> Term {
                if let Value::Variable(v) = t.value() {
                    if v == &self.var {
                        return self.value.clone();
                    }
                }
                fold_term(t, self)
            }
        }

        assert!(
            value.is_ground() || matches!(value.value(), Value::ExternalInstance(_)),
            "Expected ground term for `{}`, got `{}`",
            var,
            value.to_polar()
        );
        Grounder { var, value }.fold_operation(self.clone())
    }

    /// Augment our constraints with those on `other`.
    ///
    /// Invariant: self and other are ANDs
    pub fn merge_constraints(&mut self, other: Self) {
        assert_eq!(self.operator, Operator::And);
        assert_eq!(other.operator, Operator::And);
        self.args.extend(other.args);
    }

    // TODO(gj): simpler way to write this function.
    pub fn inverted_constraints(&self, csp: usize) -> Vec<Operation> {
        let constraints = self.constraints();
        let (old, new) = constraints.split_at(csp);
        let mut combined = old.to_vec();
        combined.push(op!(
            Not,
            term!(value!(Operation {
                operator: Operator::And,
                args: new.iter().cloned().map(|o| term!(value!(o))).collect()
            }))
        ));
        combined
    }

    pub fn constraints(&self) -> Vec<Operation> {
        assert_eq!(self.operator, Operator::And);
        self.args
            .iter()
            .map(|a| a.value().as_expression().unwrap().clone())
            .collect()
    }

    // TODO(gj): can we replace every use of this w/ clone_with_new_constraint for Immutability
    // Purposes?
    pub fn add_constraint(&mut self, o: Operation) {
        assert_eq!(self.operator, Operator::And);
        self.args.push(o.into_term());
    }

    pub fn into_term(self) -> Term {
        Term::new_temporary(Value::Expression(self))
    }

    pub fn clone_with_constraints(&self, constraints: Vec<Operation>) -> Self {
        assert_eq!(self.operator, Operator::And);
        let mut new = self.clone();
        new.args = constraints.into_iter().map(|c| c.into_term()).collect();
        new
    }

    pub fn clone_with_new_constraint(&self, constraint: Term) -> Self {
        assert_eq!(self.operator, Operator::And);
        let mut new = self.clone();
        new.args.push(constraint);
        new
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::error::{ErrorKind, PolarError, RuntimeError};
    use crate::events::QueryEvent;
    use crate::formatting::ToPolarString;
    use crate::kb::Bindings;
    use crate::polar::{Polar, Query};
    use crate::terms::Call;

    macro_rules! assert_partial_expression {
        ($bindings:expr, $sym:expr, $right:expr) => {
            assert_eq!(
                $bindings
                    .get(&sym!($sym))
                    .expect(&format!("{} is unbound", $sym))
                    .value()
                    .as_expression()
                    .unwrap()
                    .to_polar(),
                $right
            )
        };
    }

    macro_rules! assert_partial_expressions {
        ($bindings:expr, $($sym:expr => $value:expr),*) => {
            {
                let bindings = $bindings;
                $(assert_partial_expression!(bindings, $sym, $value);)*
            }
        };
    }

    macro_rules! assert_query_done {
        ($query:expr) => {
            assert!(matches!($query.next_event()?, QueryEvent::Done { .. }));
        };
    }

    macro_rules! assert_query_none {
        ($query:expr) => {
            assert!(matches!($query.next_event()?, QueryEvent::None));
        };
    }

    fn next_binding(query: &mut Query) -> Result<Bindings, PolarError> {
        let event = query.next_event()?;
        if let QueryEvent::Result { bindings, .. } = event {
            Ok(bindings)
        } else {
            panic!("not bindings, {:?}", &event);
        }
    }

    type TestResult = Result<(), PolarError>;

    #[test]
    fn basic_test() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x = 1;
               f(x) if x = 2;
               f(x) if x.a = 3 or x.b = 4;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(2));
        assert_partial_expression!(next_binding(&mut q)?, "x", "3 = _this.a");
        assert_partial_expression!(next_binding(&mut q)?, "x", "4 = _this.b");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_and() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y, z) if x = y and x = z;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), 1, 1])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), 1, 2])), false);
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_two_rule() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x, y, z) if x < y and x < z and g(x);
               g(x) if x < 3;
               g(x) if x < 4 or x < 5;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), 1, 2])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this < 1 and _this < 2 and _this < 3");
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this < 1 and _this < 2 and _this < 4");
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this < 1 and _this < 2 and _this < 5");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_isa() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x: Post) if x.foo = 1;
               f(x: User) if x.bar = 1;

               f(x: Post) if g(x.y);
               g(x: User) if x.z = 1;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this matches Post{} and 1 = _this.foo");
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this matches User{} and 1 = _this.bar");
        assert_partial_expression!(
            next_binding(&mut q)?,
            "x",
            "_this matches Post{} and _this.y matches User{} and 1 = _this.y.z"
        );
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_isa_with_fields() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x: Post{id: 1});
               f(x: Post{id: 1}) if x matches {id: 2};
               f(x: Post{id: 1}) if x matches Post{id: 2};
               f(x: Post{id: 1}) if x matches User{id: 2}; # Will fail.
               f(x: Post{id: 1}) if x matches {id: 2, bar: 2};
               f(x: Post{id: 1, bar: 1}) if x matches User{id: 2}; # Will fail.
               f(x: Post{id: 1, bar: 3}) if x matches Post{id: 2} and x.y = 1;
               f(x: {id: 1, bar: 1}) if x matches {id: 2};
               f(x: {id: 1}) if x matches {id: 2, bar: 2};
               f(x: {id: 1});
               f(x: {id: 1}) if x matches {id: 2};
               f(x: {id: 1}) if x matches Post{id: 2};
               f(x: 1);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let mut next_binding = || loop {
            match q.next_event().unwrap() {
                QueryEvent::Result { bindings, .. } => return bindings,
                QueryEvent::ExternalIsSubclass {
                    call_id,
                    left_class_tag,
                    right_class_tag,
                } => {
                    q.question_result(call_id, left_class_tag.0.starts_with(&right_class_tag.0))
                        .unwrap();
                }
                _ => panic!("not bindings"),
            }
        };
        assert_partial_expression!(next_binding(), "x", "_this matches Post{} and _this.id = 1");
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this matches Post{} and _this.id = 1 and _this.id = 2"
        );
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this matches Post{} and _this.id = 1 and _this.id = 2"
        );
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this matches Post{} and _this.id = 1 and _this.id = 2 and _this.bar = 2"
        );
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this matches Post{} and _this.id = 1 and _this.bar = 3 and _this.id = 2 and 1 = _this.y"
        );
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this.id = 1 and _this.bar = 1 and _this.id = 2"
        );
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this.id = 1 and _this.id = 2 and _this.bar = 2"
        );
        assert_partial_expression!(next_binding(), "x", "_this.id = 1");
        assert_partial_expression!(next_binding(), "x", "_this.id = 1 and _this.id = 2");
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this.id = 1 and _this matches Post{} and _this.id = 2"
        );
        assert_eq!(next_binding()[&sym!("x")], term!(1));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_isa_two_rule() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x: Post) if x.foo = 0 and g(x);
               f(x: User) if x.bar = 1 and g(x);
               g(x: Post) if x.post = 1;
               g(x: PostSubclass) if x.post_subclass = 1;
               g(x: User) if x.user = 1;
               g(x: UserSubclass) if x.user_subclass = 1;

               f(x: Foo) if h(x.y);
               h(x: Bar) if x.z = 1;
               h(x: Baz) if x.z = 1;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let mut next_binding = || loop {
            match q.next_event().unwrap() {
                QueryEvent::Result { bindings, .. } => return bindings,
                QueryEvent::ExternalIsSubclass {
                    call_id,
                    left_class_tag,
                    right_class_tag,
                } => {
                    q.question_result(call_id, left_class_tag.0.starts_with(&right_class_tag.0))
                        .unwrap();
                }
                _ => panic!("not bindings"),
            }
        };
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this matches Post{} and 0 = _this.foo and 1 = _this.post"
        );
        assert_partial_expression!(
            next_binding(),
"x",
            "_this matches Post{} and 0 = _this.foo and _this matches PostSubclass{} and 1 = _this.post_subclass"
        );
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this matches User{} and 1 = _this.bar and 1 = _this.user"
        );
        assert_partial_expression!(
            next_binding(),
"x",
            "_this matches User{} and 1 = _this.bar and _this matches UserSubclass{} and 1 = _this.user_subclass"
        );
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this matches Foo{} and _this.y matches Bar{} and 1 = _this.y.z"
        );
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this matches Foo{} and _this.y matches Baz{} and 1 = _this.y.z"
        );
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_isa_subclass_superclass() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x: PostSubclass) if g(x);
               g(x: Post);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("a")])), false);
        let mut next_binding = || loop {
            match q.next_event().unwrap() {
                QueryEvent::Result { bindings, .. } => return bindings,
                QueryEvent::ExternalIsSubclass {
                    call_id,
                    left_class_tag,
                    right_class_tag,
                } => {
                    q.question_result(call_id, left_class_tag.0.starts_with(&right_class_tag.0))
                        .unwrap();
                }
                _ => panic!("not bindings"),
            }
        };
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this matches PostSubclass{} and _this matches Post{}"
        );
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_comparison() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"positive(x) if x > 0;
               positive(x) if x > 0 and x < 0;
               zero(x) if x == 0;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("positive", [sym!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this > 0");
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this > 0 and _this < 0");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("zero", [sym!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this == 0");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_comparison_dot() -> TestResult {
        let p = Polar::new();
        p.load_str("a_positive(x) if x.a > 0 and 0 < x.a;")?;
        let mut q = p.new_query_from_term(term!(call!("a_positive", [sym!("x")])), false);
        // TODO(gj): Canonicalize comparisons.
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this.a > 0 and 0 < _this.a");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_nested_dot_ops() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x.y.z > 0;
               g(x) if x.y = 0 and x.y > 1 and x.y.z > 1;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this.y.z > 0");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "x",
            "0 = _this.y and _this.y > 1 and _this.y.z > 1"
        );
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_multiple_partials() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y) if x = 1 and y = 2;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(2));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_in_arithmetic_op() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if x = x + 0;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(error, PolarError {
            kind: ErrorKind::Runtime(RuntimeError::Unsupported { .. }), ..}));
        Ok(())
    }

    #[test]
    fn test_method_call_on_partial() -> TestResult {
        let p = Polar::new();
        p.load_str("g(x) if x.foo();")?;
        let mut q = p.new_query_from_term(term!(call!("g", [sym!("a")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(error, PolarError {
            kind: ErrorKind::Runtime(RuntimeError::Unsupported { .. }), ..}));
        Ok(())
    }

    #[test]
    fn test_partial_unification_1() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x, y) if x = y;
               f(x, y) if x = y and 1 = x;
               f(x, y) if 2 = y and x = y and x = 1;

               g(x, y) if x = 1 and y = 2;
               g(x, y) if x = 1 and y = 2 and x = y;"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(sym!("y")));
        assert_eq!(next[&sym!("y")], term!(sym!("x")));
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(2));
        assert_query_done!(q);

        // Register `y` as a partial.
        p.register_constant(sym!("y"), term!(value!(op!(And))));
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "", "y" => "x = _this");
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(2));
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_partial_unification_2() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x, y) if x = y;
               f(x, y) if x = y and 1 = x;
               f(x, y) if 2 = y and x = y and x = 1;

               g(x, y) if x = 1 and y = 2;
               g(x, y) if x = 1 and y = 2 and x = y;"#,
        )?;

        // Register `x` as a partial.
        p.register_constant(sym!("x"), term!(value!(op!(And))));
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "_this = y", "y" => "");
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(2));
        assert_query_done!(q);

        // Register `y` as a partial.
        p.register_constant(sym!("y"), term!(value!(op!(And))));
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "", "y" => "");
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(2));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_comparing_partials() -> TestResult {
        let p = Polar::new();

        p.load_str("f(x, y) if x > y;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "_this > y", "y" => "x > _this");

        p.load_str("g(x, y) if y = 1 and x > y;")?;
        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this > 1");
        assert_eq!(next[&sym!("y")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), value!(1)])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this > 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), value!(2)])), false);
        assert_query_done!(q);

        p.load_str("h(x, y) if x > y and y = 1;")?;
        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "_this > 1", "y" => "1 = _this and x > 1");
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_dot_lookup_with_unbound_as_field() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if {a: 1, b: 2}.(x) > 0;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!("a"));
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!("b"));
        Ok(())
    }

    #[test]
    fn test_dot_lookup_with_partial_as_field() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y) if {a: y, b: y}.(x) > 0;")?;
        p.register_constant(sym!("x"), term!(value!(op!(And))));
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!("a"));
        assert_partial_expression!(next, "y", "_this > 0");
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!("b"));
        assert_partial_expression!(next, "y", "_this > 0");
        Ok(())
    }

    #[test]
    fn test_partial_inverter() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if not x = 1;
               g(x) if not x > 1;
               h(x) if not (x = 1 and x = 2);
               i(x) if not (x = 1 or x = 2);
               j(x) if not (not x = 1);
               k(x) if not (not (not x = 1));"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 != _this");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this <= 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(sym!("x")));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 != _this and 2 != _this");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("j", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("k", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 != _this");
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_negate_conjunctions() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if not (y = 1 and x.foo = y);
               g(x) if not (x.foo = y and 1 = y);
               h(x) if not (y = 1 and x.foo.bar = y);
               i(x) if not (y = x.foo.bar and 1 = y);"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this.foo != 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this.foo != 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 != _this.foo.bar");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 != _this.foo.bar");
        assert_query_done!(q);

        Ok(())
    }

    #[ignore]
    #[test]
    fn partially_negated_constraints() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x = 3 and not (x = 1 and (not x = 2));
               g(x) if not (x = 1 and (not x = 2));
               h(x) if x = 1 and not (x = 2 or x = 3);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this = 3 and _this != 1 or _this = 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this != 1 or _this = 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this = 1 and _this != 2 and _this != 3");
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn partial_with_unbound_variables() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if not (x.foo = y);
               g(x) if not (x.foo.bar = y);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_query_none!(q);
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_query_none!(q);
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_negate_disjunctions() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if not (x.foo = 1 or 2 = x.foo);
               g(x) if not (1 = x or x = 2);
               h(x) if not (x.foo.bar = 1 or 2 = x.foo.bar);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "1 != _this.foo and 2 != _this.foo");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 != _this and 2 != _this");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "1 != _this.foo.bar and 2 != _this.foo.bar");
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_trivial_partials() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x);
               g(x) if false;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(sym!("x")));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_in_partial_lhs() -> TestResult {
        let p = Polar::new();
        p.load_str("lhs(x) if x in [1, 2];")?;
        // Partials on the LHS of `in` accumulate constraints disjunctively.
        let mut q = p.new_query_from_term(term!(call!("lhs", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(2));
        assert_query_done!(q);
        Ok(())
    }

    #[ignore]
    #[test]
    fn test_negated_in_partial_lhs() -> TestResult {
        let p = Polar::new();
        p.load_str("not_lhs(x) if not x in [1, 2];")?;
        // Inverting an `in` produces a conjunction of the inverted disjunctive constraints.
        let mut q = p.new_query_from_term(term!(call!("not_lhs", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this != 1 and _this != 2");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_contains_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"contains(x, y) if x in y;
               contains_dot(x, y) if x in y.foo;
               contains_dot_dot(x, y) if x in y.foo.bar and y.foo = 2;"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("contains", [1, sym!("y")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 in _this");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("contains_dot", [1, sym!("y")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 in _this.foo");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("contains_dot_dot", [1, sym!("y")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "y",
            "1 in _this.foo.bar and 2 = _this.foo"
        );
        assert_query_done!(q);

        Ok(())
    }

    #[ignore]
    #[test]
    fn test_in_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if y in x.values;
               g(x, y) if y in x.values;
               h(x) if y in x.values and (y.bar = 1 and y.baz = 2) or y.bar = 3;"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        // TODO (dhatch): This doesn't work now, but ultimately this should have
        // no constraints since nothing is added to `y`.
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "x" => "_y_12 in _this.values"
        );
        assert_query_done!(q);

        // Not sure about this one, where there's an output binding.  There are still
        // no constraints on b.
        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "y in _this.values");
        assert_partial_expression!(next, "y", "_this in x.values");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "x" => "_y_27 in _this.values and 1 = _y_27.bar and 2 = _y_27.baz"
        );
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "x" => "_y_27 in _this.values and 3 = _y_27.bar"
        );
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_that_cut_with_partial_errors() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if cut;")?;
        p.register_constant(sym!("x"), op!(And).into_term());
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(error, PolarError {
            kind: ErrorKind::Runtime(RuntimeError::Unsupported { .. }), ..}));
        Ok(())
    }

    #[test]
    #[ignore = "cut not yet implemented with partials"]
    fn test_cut_with_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x = 1;
               f(x) if x = 2 and cut;
               f(x) if x = 3;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("a")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(1));
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(2));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    #[ignore = "cut not yet implemented with partials"]
    fn test_conditional_cut_with_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x = 1 or x = 2 and cut and x = 2;
               g(1) if cut;
               g(2);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 1 and _this = 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("a")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(1));
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(2));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    #[ignore = "cut not yet implemented with partials"]
    fn test_method_sorting_with_cut_and_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x, y) if cut and x = 1;
               f(x, y: 2) if x = 2;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("a"), value!(2)])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(2));
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(1));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_assignment_to_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x := 1;
               g(x) if x = 1 and y := x;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn nonlogical_inversions() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if not print(x);")?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("a")])), false);
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_nested_dot_in() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y) if x in y.a.b.c;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [1, sym!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "1 in _this.a.b.c");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_nested_dot_lookup() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x, y) if x = y.a.b.c;
               f(x, y) if x > y.a.b.c and x < y.a.b and y.a.b.c > x;
               f(x, y) if x = y.a;
               f(x, y) if x = y.a.b;
               f(x, y) if x = y.a.b.c.d;
               f(x, y) if x = y.a.b.c.d.e;
               f(x, y) if x = y.a.b.c.d.e.f;"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [1, sym!("y")])), false);

        assert_partial_expression!(next_binding(&mut q)?, "y", "1 = _this.a.b.c");
        assert_partial_expression!(
            next_binding(&mut q)?,
            "y",
            "1 > _this.a.b.c and 1 < _this.a.b and _this.a.b.c > 1"
        );
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 = _this.a");
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 = _this.a.b");
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 = _this.a.b.c.d");
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 = _this.a.b.c.d.e");
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 = _this.a.b.c.d.e.f");
        assert_query_done!(q);
        Ok(())
    }

    #[ignore]
    #[test]
    fn test_partial_negated_isa() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if (not x matches Foo{} or not g(x)) and x = 1;
               g(_: Bar);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "x",
            "not _this matches Foo{} and _this = 1"
        );
        assert_partial_expression!(
            next_binding(&mut q)?,
            "x",
            "not _this matches Bar{} and _this = 1"
        );
        assert_query_done!(q);
        Ok(())
    }
}
