use serde::{Deserialize, Serialize};

use crate::error::PolarResult;
use crate::events::QueryEvent;
use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Pattern, Symbol, Term, Value};

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

    pub fn operations(&self) -> &Vec<Operation> {
        &self.operations
    }

    pub fn operations_mut(&mut self) -> &mut Vec<Operation> {
        &mut self.operations
    }

    pub fn unify(&mut self, other: Term) {
        let op = op!(Unify, self.variable_term(), other);
        self.operations.push(op);
    }

    pub fn isa(&mut self, other: Term) -> Box<dyn Runnable> {
        let isa_op = op!(Isa, self.variable_term(), other);

        let constraint_check = Box::new(IsaConstraintCheck::new(
            self.operations.clone(),
            isa_op.clone(),
        ));

        self.operations.push(isa_op);
        constraint_check
    }

    pub fn compare(&mut self, operator: Operator, other: Term) {
        assert!(matches!(
            operator,
            Operator::Lt
                | Operator::Gt
                | Operator::Leq
                | Operator::Geq
                | Operator::Eq
                | Operator::Neq
        ));

        let op = Operation {
            operator,
            args: vec![self.variable_term(), other],
        };

        self.operations.push(op);
    }

    pub fn is_compatible<F>(&self, check: F) -> bool
    where
        F: Fn(&Operation) -> bool,
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

    pub fn clone_with_operations(&self, operations: Vec<Operation>) -> Self {
        let mut new = self.clone();
        new.operations = operations;
        new
    }

    pub fn name(&self) -> &Symbol {
        &self.variable
    }

    fn variable_term(&self) -> Term {
        Term::new_temporary(Value::Variable(sym!("_this")))
    }
}

#[derive(Clone)]
struct IsaConstraintCheck {
    existing: Vec<Operation>,
    proposed_tag: Option<Symbol>,
    result: Option<bool>,
}

impl IsaConstraintCheck {
    pub fn new(existing: Vec<Operation>, mut proposed: Operation) -> Self {
        let right = proposed.args.pop().unwrap();
        let proposed_tag = if let Value::Pattern(Pattern::Instance(instance)) = right.value() {
            Some(instance.tag.clone())
        } else {
            None
        };

        Self {
            existing,
            proposed_tag,
            result: None,
        }
    }

    /// Check if constraint is compatible with proposed.
    ///
    /// Returns: None if compatible, QueryEvent::Done { false } if incompatible,
    /// or QueryEvent to ask for compatibility.
    fn check_constraint(&self, mut constraint: Operation) -> Option<QueryEvent> {
        if constraint.operator != Operator::Isa {
            return None;
        }

        let right = constraint.args.pop().unwrap();
        if let Value::Pattern(Pattern::Instance(instance)) = right.value() {
            // is_subclass check of instance tag against proposed
            return Some(QueryEvent::ExternalIsSubSpecializer {
                call_id: 700000,
                instance_id: None,
                left_class_tag: self.proposed_tag.clone().unwrap(),
                right_class_tag: instance.tag.clone(),
            });

            // TODO check fields for compatibility.
        }

        None
    }
}

impl Runnable for IsaConstraintCheck {
    fn run(&mut self) -> PolarResult<QueryEvent> {
        if self.proposed_tag.is_none() {
            return Ok(QueryEvent::Done { result: true });
        }

        if let Some(result) = self.result.take() {
            if !result {
                return Ok(QueryEvent::Done { result: false });
            }
        }

        loop {
            let next = self.existing.pop();
            if let Some(constraint) = next {
                if let Some(event) = self.check_constraint(constraint) {
                    return Ok(event);
                }

                continue;
            } else {
                return Ok(QueryEvent::Done { result: true });
            }
        }
    }

    fn external_question_result(&mut self, call_id: u64, answer: bool) -> PolarResult<()> {
        assert_eq!(call_id, 700000);

        self.result = Some(answer);
        Ok(())
    }

    fn clone_runnable(&self) -> Box<dyn Runnable> {
        Box::new(self.clone())
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
                    .expression()
                    .unwrap()
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
        assert_eq!(next_binding().get(&sym!("a")).unwrap(), &term!(1));

        assert_eq!(next_binding().get(&sym!("a")).unwrap(), &term!(2));


        let next = next_binding();
        // LOOKUPS also work.. but obviously the expression could be merged and simplified.
        // The basic information is there though.
        assert_partial_expression!(next, "a", "3 = _this.a");

        let next = next_binding();
        assert_partial_expression!(next, "a", "4 = _this.b");

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
        assert_partial_expression!(
            next,
            "a",
            "_this matches Post{} and 1 = _this.foo"
        );

        let next = next_binding();
        assert_partial_expression!(
            next,
            "a",
            "_this matches User{} and 1 = _this.bar"
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
        polar.load_str(r#"g(x: Post) if x.post = 1;"#).unwrap();
        polar
            .load_str(r#"g(x: PostSubclass) if x.post_subclass = 1;"#)
            .unwrap();
        polar.load_str(r#"g(x: User) if x.user = 1;"#).unwrap();
        polar
            .load_str(r#"g(x: UserSubclass) if x.user_subclass = 1;"#)
            .unwrap();

        let mut query =
            polar.new_query_from_term(term!(call!("f", [Constraints::new(sym!("a"))])), false);

        let mut next_binding = || loop {
            match query.next_event().unwrap() {
                QueryEvent::Result { bindings, .. } => return bindings,
                QueryEvent::ExternalIsSubSpecializer {
                    call_id,
                    instance_id,
                    left_class_tag,
                    right_class_tag,
                } => {
                    assert!(instance_id.is_none());
                    assert_eq!(call_id, 700000);
                    eprintln!("left: {:?}, right: {:?}", &left_class_tag, &right_class_tag);
                    query
                        .question_result(call_id, left_class_tag.0.starts_with(&right_class_tag.0));
                }
                _ => panic!("not bindings"),
            }
        };

        let next = next_binding();
        assert_partial_expression!(
            next,
            "a",
            "_this matches Post{} and 0 = _this.foo and _this matches Post{} and 1 = _this.post"
        );

        let next = next_binding();
        assert_partial_expression!(
            next,
            "a",
            "_this matches Post{} and 0 = _this.foo and _this matches PostSubclass{} and 1 = _this.post_subclass"
        );

        let next = next_binding();
        assert_partial_expression!(
            next,
            "a",
            "_this matches User{} and 1 = _this.bar and _this matches User{} and 1 = _this.user"
        );

        let next = next_binding();
        assert_partial_expression!(
            next,
            "a",
            "_this matches User{} and 1 = _this.bar and _this matches UserSubclass{} and 1 = _this.user_subclass"
        );

        assert!(matches!(query.next_event().unwrap(), QueryEvent::Done { .. }));

        Ok(())
    }

    #[test]
    fn test_partial_comparison() -> Result<(), crate::error::PolarError> {
        let polar = Polar::new();
        polar.load_str(r#"positive(x) if x > 0;"#).unwrap();
        polar.load_str(r#"positive(x) if x > 0 and x < 0;"#).unwrap();

        let mut query = polar.new_query_from_term(
            term!(call!("positive", [Constraints::new(sym!("a"))])),
            false,
        );

        let mut next_binding = || {
            let event = query.next_event().unwrap();
            if let QueryEvent::Result { bindings, .. } = event {
                bindings
            } else {
                panic!("not bindings, {:?}", &event);
            }
        };

        let next = next_binding();
        assert_partial_expression!(next, "a", "_this > 0");

        let next = next_binding();
        assert_partial_expression!(next, "a", "_this > 0 and _this < 0");

        Ok(())
    }

    #[test]
    fn test_partial_comparison_dot() -> Result<(), crate::error::PolarError> {
        let polar = Polar::new();
        polar.load_str(r#"positive(x) if x.a.b > 0;"#).unwrap();
        polar.load_str(r#"positive(x) if x > 0 and x < 0;"#).unwrap();

        let mut query = polar.new_query_from_term(
            term!(call!("positive", [Constraints::new(sym!("a"))])),
            false,
        );

        let mut next_binding = || {
            let event = query.next_event().unwrap();
            if let QueryEvent::Result { bindings, .. } = event {
                bindings
            } else {
                panic!("not bindings, {:?}", &event);
            }
        };

        let next = next_binding();
        assert_partial_expression!(next, "a", "_this.a > 0");

        Ok(())
    }
}
