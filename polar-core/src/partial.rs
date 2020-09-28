use crate::terms::{Operation, Operator, Symbol, Term, Value, Pattern};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Constraints {
    operations: Vec<Operation>,
    // TODO move to the top level value type to correspond better with Value::Variable.
    variable: Symbol,
}

impl Constraints {
    pub fn new(variable: Symbol) -> Self {
        Constraints {
            operations: vec![],
            variable,
        }
    }

    pub fn unify(&mut self, other: Term) -> bool {
        let op = op!(Unify, self.variable_term(), other);
        if !self.is_compatible(op) {
            return false;
        }

        self.operations.push(op);

        return true;
    }

    pub fn isa(&mut self, other: Term) -> bool {
        let isa_op = op!(Isa, self.variable_term(), other);
        if !self.is_compatible(|op| {
            if op.operator == Operation::Isa {
                let right = args.pop().unwrap();
                let left = args.pop().unwrap();

                if let Value::Pattern(Pattern::Instance(instance)) = right {
                    let check_tag = instance.tag;

                }
            }
        }) {
            return false;
        }

        self.operations.push(isa_op);

        return true;
    }

    pub fn is_compatible<F>(&self, check: F) -> bool
    where F: Fn(&Operation) -> bool
    {
        self.operations.iter().all(check)
    }

    /// Add lookup of `field` assigned to `value` on `self.
    ///
    /// Returns: A partial expression for `value`.
    pub fn lookup(&mut self, field: Term, value: Term) -> Term {
        // Note this is a 2-arg lookup (Dot) not 3-arg. (Pre rewrite).
        assert!(matches!(field.value(), Value::String(_)));

        self.operations.push(op!(
            Unify,
            value.clone(),
            term!(op!(Dot, self.variable_term(), field))
        ));

        let name = value.value().clone().symbol().unwrap();
        Term::new_temporary(Value::Partial(Constraints::new(name)))
    }

    /// Return a regular expression consisting of the expression represented by this partial.
    pub fn as_term(self) -> Term {
        Term::new_temporary(Value::Partial(self))
    }

    // HACK for formatting.
    pub fn as_expression(self) -> Term {
        Term::new_temporary(Value::Expression(Operation {
            operator: Operator::And,
            args: self
                .operations
                .into_iter()
                .map(|op| Term::new_temporary(Value::Expression(op)))
                .collect(),
        }))
    }

    pub fn clone_with_name(&self, name: Symbol) -> Self {
        let mut new = self.clone();
        new.variable = name;
        new
    }

    pub fn name(&self) -> &Symbol {
        &self.variable
    }

    fn variable_term(&self) -> Term {
        Term::new_temporary(Value::Variable(sym!("_this")))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::events::QueryEvent;
    use crate::formatting::ToPolarString;
    use crate::polar::Polar;
    use crate::terms::Call;

    macro_rules! assert_partial_expression {
        ($bindings:expr, $sym:expr, $right:expr) => {
            assert_eq!(
                $bindings
                    .get(&sym!($sym))
                    .unwrap()
                    .value()
                    .clone()
                    .partial()?
                    .as_expression()
                    .to_polar(),
                $right
            )
        };
    }

    #[test]
    fn basic_test() -> Result<(), crate::error::PolarError> {
        let polar = Polar::new();
        polar.load_str(r#"f(x) if x = 1;"#).unwrap();
        polar.load_str(r#"f(x) if x = 2;"#).unwrap();
        polar.load_str(r#"f(x) if x.a = 3 or x.b = 4;"#).unwrap();

        let mut query =
            polar.new_query_from_term(term!(call!("f", [Constraints::new(sym!("a"))])), false);

        let mut next_binding = || {
            if let QueryEvent::Result { bindings, .. } = query.next_event().unwrap() {
                bindings
            } else {
                panic!("not bindings");
            }
        };

        // Super hacked up...
        //
        // Each set of bindings is one possible set of constraints that must be
        // satisified for the rule to be true.  They could be OR'ed together to enter
        // into a system like SQL.
        //
        // Each constraint is emitted as a binding named (partial_SOMETHING).
        // This is just really hacky, there should be a separate output for these.
        // They all just be AND'd together.
        //
        // Really simple unification works fine...
        assert_partial_expression!(next_binding(), "a", "_this = 1");

        assert_partial_expression!(next_binding(), "a", "_this = 2");

        let next = next_binding();
        // LOOKUPS also work.. but obviously the expression could be merged and simplified.
        // The basic information is there though.
        assert_partial_expression!(next, "a", "_value_1_11 = _this.a");
        assert_partial_expression!(next, "_value_1_11", "_this = 3");

        let next = next_binding();
        assert_partial_expression!(next, "a", "_value_2_12 = _this.b");
        assert_partial_expression!(next, "_value_2_12", "_this = 4");

        // Print messages
        while let Some(msg) = query.next_message() {
            println!("{:?}", msg);
        }

        Ok(())
    }

    #[test]
    fn test_partial_and() -> Result<(), crate::error::PolarError> {
        let polar = Polar::new();
        polar.load_str(r#"f(x, y, z) if x = y and x = z;"#).unwrap();

        let mut query = polar.new_query_from_term(
            term!(call!("f", [Constraints::new(sym!("a")), 1, 2])),
            false,
        );

        let mut next_binding = || {
            if let QueryEvent::Result { bindings, .. } = query.next_event().unwrap() {
                bindings
            } else {
                panic!("not bindings");
            }
        };

        let next = next_binding();
        assert_partial_expression!(next, "a", "_this = 1 and _this = 2");

        Ok(())
    }

    #[test]
    fn test_partial_two_rule() -> Result<(), crate::error::PolarError> {
        let polar = Polar::new();
        polar
            .load_str(r#"f(x, y, z) if x = y and x = z and g(x);"#)
            .unwrap();
        polar.load_str(r#"g(x) if x = 3;"#).unwrap();
        polar.load_str(r#"g(x) if x = 4 or x = 5;"#).unwrap();

        let mut query = polar.new_query_from_term(
            term!(call!("f", [Constraints::new(sym!("a")), 1, 2])),
            false,
        );

        let mut next_binding = || {
            if let QueryEvent::Result { bindings, .. } = query.next_event().unwrap() {
                bindings
            } else {
                panic!("not bindings");
            }
        };

        let next = next_binding();
        assert_partial_expression!(next, "a", "_this = 1 and _this = 2 and _this = 3");

        let next = next_binding();
        assert_partial_expression!(next, "a", "_this = 1 and _this = 2 and _this = 4");

        let next = next_binding();
        assert_partial_expression!(next, "a", "_this = 1 and _this = 2 and _this = 5");

        Ok(())
    }

    #[test]
    fn test_partial_isa() -> Result<(), crate::error::PolarError> {
        let polar = Polar::new();
        polar.load_str(r#"f(x: Post) if x.foo = 1;"#).unwrap();
        polar.load_str(r#"f(x: User) if x.bar = 1;"#).unwrap();

        let mut query =
            polar.new_query_from_term(term!(call!("f", [Constraints::new(sym!("a"))])), false);

        let mut next_binding = || {
            if let QueryEvent::Result { bindings, .. } = query.next_event().unwrap() {
                bindings
            } else {
                panic!("not bindings");
            }
        };

        let next = next_binding();
        assert_partial_expression!(next, "a", "_this matches Post{} and _value_1_9 = _this.foo");
        assert_partial_expression!(next, "_value_1_9", "_this = 1");

        let next = next_binding();
        assert_partial_expression!(
            next,
            "a",
            "_this matches User{} and _value_2_11 = _this.bar"
        );
        assert_partial_expression!(
            next,
            "a",
            "_this matches User{} and _value_2_11 = _this.bar"
        );

        Ok(())
    }

    #[test]
    fn test_partial_isa_two_rule() -> Result<(), crate::error::PolarError> {
        let polar = Polar::new();
        polar
            .load_str(r#"f(x: Post) if x.foo = 0 and g(x);"#)
            .unwrap();
        polar
            .load_str(r#"f(x: User) if x.bar = 1 and g(x);"#)
            .unwrap();

        polar
            .load_str(r#"g(x: Post) if x.baz = 1 and g(x);"#)
            .unwrap();
        polar
            .load_str(r#"g(x: User) if x.bar = 1 and g(x);"#)
            .unwrap();

        let mut query =
            polar.new_query_from_term(term!(call!("f", [Constraints::new(sym!("a"))])), false);

        let mut next_binding = || {
            if let QueryEvent::Result { bindings, .. } = query.next_event().unwrap() {
                bindings
            } else {
                panic!("not bindings");
            }
        };

        let next = next_binding();
        assert_partial_expression!(next, "a", "_this matches Post{} and _value_1_9 = _this.foo");
        assert_partial_expression!(next, "_value_1_9", "_this = 1");

        let next = next_binding();
        assert_partial_expression!(
            next,
            "a",
            "_this matches User{} and _value_2_11 = _this.bar"
        );
        assert_partial_expression!(
            next,
            "a",
            "_this matches User{} and _value_2_11 = _this.bar"
        );

        Ok(())
    }
}
