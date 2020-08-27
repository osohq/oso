use crate::terms::{Operation, Operator, Symbol, Term, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Expression {
    operations: Vec<Operation>,
    variable: Symbol,
}

impl Expression {
    pub fn new(variable: Symbol) -> Self {
        Expression {
            operations: vec![],
            variable,
        }
    }

    pub fn unify(&mut self, other: Term) {
        self.operations
            .push(op!(Unify, self.variable_term(), other));
    }

    pub fn lookup(&mut self, field: Term, value: Term) -> (Symbol, Term) {
        // Note this is a 2-arg lookup (Dot) not 3-arg. (Pre rewrite).
        assert!(matches!(field.value(), Value::String(_)));

        self.operations.push(op!(
            Unify,
            value.clone(),
            term!(op!(Dot, self.variable_term(), field))
        ));

        let name = value.value().clone().symbol().unwrap();
        (
            name.clone(),
            Term::new_temporary(Value::Partial(Expression::new(name))),
        )
    }

    /// Return a regular expression consisting of the expression represented by this partial.
    pub fn finalize(self) -> Term {
        Term::new_temporary(Value::Expression(Operation {
            operator: Operator::And,
            args: self
                .operations
                .into_iter()
                .map(|op| Term::new_temporary(Value::Expression(op)))
                .collect(),
        }))
    }

    pub fn name(&self) -> &Symbol {
        &self.variable
    }

    fn variable_term(&self) -> Term {
        Term::new_temporary(Value::Variable(self.variable.clone()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::events::QueryEvent;
    use crate::formatting::ToPolarString;
    use crate::polar::Polar;
    use crate::terms::Call;

    #[test]
    fn basic_test() {
        let polar = Polar::new();
        polar.load_str(r#"f(x) if x = 1;"#).unwrap();
        polar.load_str(r#"f(x) if x = 2;"#).unwrap();
        polar.load_str(r#"f(x) if x.a = 3 or x.b = 4;"#).unwrap();

        let mut query =
            polar.new_query_from_term(term!(call!("f", [Expression::new(sym!("a"))])), false);

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
        assert_eq!(
            next_binding().get(&sym!("partial_a")).unwrap().to_polar(),
            "a = 1"
        );

        assert_eq!(
            next_binding().get(&sym!("partial_a")).unwrap().to_polar(),
            "a = 2"
        );

        let next = next_binding();
        // LOOKUPS also work.. but obviously the expression could be merged and simplified.
        // The basic information is there though.
        assert_eq!(
            next.get(&sym!("partial_a")).unwrap().to_polar(),
            "_value_1_11 = a.a"
        );
        assert_eq!(
            next.get(&sym!("partial__value_1_11")).unwrap().to_polar(),
            "_value_1_11 = 3"
        );

        let next = next_binding();
        assert_eq!(
            next.get(&sym!("partial_a")).unwrap().to_polar(),
            "_value_2_12 = a.b"
        );
        assert_eq!(
            next.get(&sym!("partial__value_2_12")).unwrap().to_polar(),
            "_value_2_12 = 4"
        );

        // Print messages
        while let Some(msg) = query.next_message() {
            println!("{:?}", msg);
        }
    }
}
