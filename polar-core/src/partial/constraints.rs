use serde::{Deserialize, Serialize};

use crate::counter::Counter;
use crate::error::{OperationalError, PolarResult};
use crate::events::QueryEvent;
use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Pattern, Symbol, Term, Value};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Constraints {
    pub operations: Vec<Operation>,
    pub variable: Symbol,
}

/// Invert operators.
fn invert_operation(Operation { operator, args }: Operation) -> Operation {
    Operation {
        operator: match operator {
            Operator::And => Operator::Or,
            Operator::Or => Operator::And,
            Operator::Unify | Operator::Eq => Operator::Neq,
            Operator::Neq => Operator::Unify,
            Operator::Gt => Operator::Leq,
            Operator::Geq => Operator::Lt,
            Operator::Lt => Operator::Geq,
            Operator::Leq => Operator::Gt,
            Operator::Debug | Operator::Print | Operator::New | Operator::Dot => operator,
            _ => todo!("negate {:?}", operator),
        },
        args,
    }
}

impl Constraints {
    pub fn new(variable: Symbol) -> Self {
        Self {
            operations: vec![],
            variable,
        }
    }

    /// Augment our constraints with those on `other`.
    ///
    /// Invariant: both partials must have the same variable.
    pub fn merge_constraints(&mut self, other: Self) {
        assert_eq!(self.variable, other.variable);
        self.operations.extend(other.operations);
    }

    pub fn inverted_operations(&self, csp: usize) -> Vec<Operation> {
        let (old, new) = self.operations.split_at(csp);
        let mut combined = old.to_vec();
        match new.len() {
            // Do nothing to an empty partial.
            0 => (),

            // Invert a single constraint.
            1 => combined.push(invert_operation(new[0].clone())),

            // Invert the conjunction of multiple constraints, yielding a disjunction of their
            // inverted selves. (De Morgan's Law)
            _ => {
                let inverted = new.iter().cloned().map(invert_operation);
                let inverted = inverted.map(|o| Term::new_temporary(Value::Expression(o)));
                let inverted = Operation {
                    operator: Operator::Or,
                    args: inverted.collect(),
                };
                combined.push(inverted);
            }
        }
        combined
    }

    pub fn operations(&self) -> &Vec<Operation> {
        &self.operations
    }

    pub fn add_constraint(&mut self, o: Operation) {
        self.operations.push(o);
    }

    pub fn unify(&mut self, other: Term) {
        let op = op!(Unify, self.variable_term(), other);
        self.add_constraint(op);
    }

    pub fn isa(&mut self, other: Term) -> Box<dyn Runnable> {
        let isa_op = op!(Isa, self.variable_term(), other);

        let constraint_check = Box::new(IsaConstraintCheck::new(
            self.operations.clone(),
            isa_op.clone(),
        ));

        self.add_constraint(isa_op);
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

        self.add_constraint(op);
    }

    /// Add lookup of `field` assigned to `value` on `self.
    ///
    /// Returns: A partial expression for `value`.
    pub fn lookup(&mut self, field: Term, value: Term) -> Term {
        // Note this is a 2-arg lookup (Dot) not 3-arg. (Pre rewrite).
        assert!(matches!(field.value(), Value::String(_)));

        self.add_constraint(op!(
            Unify,
            value.clone(),
            term!(op!(Dot, self.variable_term(), field))
        ));

        let name = value.value().as_symbol().unwrap();
        Term::new_temporary(Value::Partial(Constraints::new(name.clone())))
    }

    pub fn into_term(self) -> Term {
        Term::new_temporary(Value::Partial(self))
    }

    /// Return the expression represented by this partial's constraints.
    pub fn into_expression(mut self) -> Term {
        if self.operations.len() == 1 {
            Term::new_temporary(Value::Expression(self.operations.pop().unwrap()))
        } else {
            Term::new_temporary(Value::Expression(Operation {
                operator: Operator::And,
                args: self
                    .operations
                    .into_iter()
                    .map(|op| Term::new_temporary(Value::Expression(op)))
                    .collect(),
            }))
        }
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
    last_call_id: u64,
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
            last_call_id: 0,
        }
    }

    /// Check if the existing constraints set is compatible with the proposed
    /// matches class.
    ///
    /// Returns: None if compatible, QueryEvent::Done { false } if incompatible,
    /// or QueryEvent to ask for compatibility.
    fn check_constraint(
        &mut self,
        mut constraint: Operation,
        counter: &Counter,
    ) -> Option<QueryEvent> {
        if constraint.operator != Operator::Isa {
            return None;
        }

        let right = constraint.args.pop().unwrap();
        if let Value::Pattern(Pattern::Instance(instance)) = right.value() {
            let call_id = counter.next();
            self.last_call_id = call_id;

            // is_subclass check of instance tag against proposed
            return Some(QueryEvent::ExternalIsSubclass {
                call_id,
                left_class_tag: self.proposed_tag.clone().unwrap(),
                right_class_tag: instance.tag.clone(),
            });

            // TODO check fields for compatibility.
        }

        None
    }
}

impl Runnable for IsaConstraintCheck {
    fn run(&mut self, counter: Option<&mut Counter>) -> PolarResult<QueryEvent> {
        if self.proposed_tag.is_none() {
            return Ok(QueryEvent::Done { result: true });
        }

        if let Some(result) = self.result.take() {
            if !result {
                return Ok(QueryEvent::Done { result: false });
            }
        }

        let counter = counter.expect("IsaConstraintCheck requires a Counter");
        loop {
            let next = self.existing.pop();
            if let Some(constraint) = next {
                if let Some(event) = self.check_constraint(constraint, &counter) {
                    return Ok(event);
                }

                continue;
            } else {
                return Ok(QueryEvent::Done { result: true });
            }
        }
    }

    fn external_question_result(&mut self, call_id: u64, answer: bool) -> PolarResult<()> {
        if call_id != self.last_call_id {
            return Err(OperationalError::InvalidState(String::from("Unexpected call id")).into());
        }

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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 1");
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 2");
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.a = 3");
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.b = 4");
        Ok(())
    }

    #[test]
    fn test_partial_and() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y, z) if x = y and x = z;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a"), 1, 2])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 1 and _this = 2");
        Ok(())
    }

    #[test]
    fn test_partial_two_rule() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x, y, z) if x = y and x = z and g(x);
               g(x) if x = 3;
               g(x) if x = 4 or x = 5;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a"), 1, 2])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this = 1 and _this = 2 and _this = 3");
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this = 1 and _this = 2 and _this = 4");
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this = 1 and _this = 2 and _this = 5");
        Ok(())
    }

    #[test]
    fn test_partial_isa() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x: Post) if x.foo = 1;
               f(x: User) if x.bar = 1;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this matches Post{} and _this.foo = 1");
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this matches User{} and _this.bar = 1");
        Ok(())
    }

    #[test]
    fn test_partial_isa_with_fields() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x: Post{id: 1});")?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(error, PolarError {
            kind: ErrorKind::Runtime(RuntimeError::Unsupported { .. }), ..}));
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
               g(x: UserSubclass) if x.user_subclass = 1;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
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
            "_this matches Post{} and _this.foo = 0 and _this.post = 1"
        );
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this matches Post{} and _this.foo = 0 and _this matches PostSubclass{} and _this.post_subclass = 1"
        );
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this matches User{} and _this.bar = 1 and _this.user = 1"
        );
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this matches User{} and _this.bar = 1 and _this matches UserSubclass{} and _this.user_subclass = 1"
        );
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));
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
        let mut q = p.new_query_from_term(term!(call!("positive", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this > 0");
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this > 0 and _this < 0");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("zero", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this == 0");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));
        Ok(())
    }

    #[test]
    fn test_partial_comparison_dot() -> TestResult {
        let p = Polar::new();
        p.load_str("positive(x) if x.a > 0;")?;
        let mut q = p.new_query_from_term(term!(call!("positive", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.a > 0");
        Ok(())
    }

    #[test]
    fn test_partial_nested_dot_ops() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x.y.z > 0;
               g(x) if x.y = 0 and x.y > 1 and x.y.z > 1 and x = 2;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.y.z > 0");

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "a",
            "_this.y = 0 and _this.y > 1 and _this.y.z > 1 and _this = 2"
        );
        Ok(())
    }

    #[test]
    fn test_multiple_partials() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y) if x = 1 and y = 2;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a"), partial!("b")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this = 1");
        assert_partial_expression!(next, "b", "_this = 2");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));
        Ok(())
    }

    #[test]
    fn test_partial_in_arithmetic_op() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if x = x + 0;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(error, PolarError {
            kind: ErrorKind::Runtime(RuntimeError::Unsupported { .. }), ..}));
        Ok(())
    }

    #[test]
    fn test_method_call_on_partial() -> TestResult {
        let p = Polar::new();
        p.load_str("g(x) if x.foo();")?;
        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(error, PolarError {
            kind: ErrorKind::Runtime(RuntimeError::Unsupported { .. }), ..}));
        Ok(())
    }

    #[test]
    fn test_unifying_partials() -> TestResult {
        let p = Polar::new();
        p.load_str("h(x, y) if x = y;")?;
        let mut q = p.new_query_from_term(term!(call!("h", [partial!("a"), partial!("b")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(error, PolarError {
            kind: ErrorKind::Runtime(RuntimeError::Unsupported { .. }), ..}));
        Ok(())
    }

    #[test]
    fn test_comparing_partials() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y) if x > y;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a"), partial!("b")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(error, PolarError {
            kind: ErrorKind::Runtime(RuntimeError::Unsupported { .. }), ..}));
        Ok(())
    }

    #[test]
    fn test_dot_lookup_with_partial_as_field() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if {}.(x);")?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(error, PolarError {
            kind: ErrorKind::Runtime(RuntimeError::TypeError { .. }), ..}));
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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this != 1");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this <= 1");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("h", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this != 1 or _this != 2");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("i", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this != 1 and _this != 2");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("j", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 1");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("k", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this != 1");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.foo != 1");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.foo != 1");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("h", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.foo.bar != 1");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("i", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.foo.bar != 1");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));
        Ok(())
    }

    #[test]
    fn partially_negated_constraints() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x = 3 and not (x = 1 and (not x = 2));
               g(x) if not (x = 1 and (not x = 2));
               h(x) if x = 1 and not (x = 2 or x = 3);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this = 3 and _this != 1 or _this = 2");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this != 1 or _this = 2");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("h", [partial!("a")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this = 1 and _this != 2 and _this != 3");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        Ok(())
    }

    #[test]
    fn partial_with_unbound_variables() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if not (x.foo = y);
               g(x) if not (x.foo.bar = y);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));
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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this.foo != 1 and _this.foo != 2");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this != 1 and _this != 2");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("h", [partial!("a")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this.foo.bar != 1 and _this.foo.bar != 2");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));
        Ok(())
    }

    #[test]
    fn test_trivial_partials() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x);
               g(x) if false;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));
        Ok(())
    }

    #[test]
    fn test_in_with_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"lhs(x) if x in [1, 2];
               not_lhs(x) if not x in [1, 2];
               rhs(x) if 1 in x;"#,
        )?;

        // Partials on the LHS of `in` accumulate constraints disjunctively.
        let mut q = p.new_query_from_term(term!(call!("lhs", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 1");
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 2");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        // Inverting an `in` produces a conjunction of the inverted disjunctive constraints.
        let mut q = p.new_query_from_term(term!(call!("not_lhs", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this != 1 and _this != 2");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        // Partials are not allowed on the RHS of `in`.
        let mut q = p.new_query_from_term(term!(call!("rhs", [partial!("a")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(error, PolarError {
            kind: ErrorKind::Runtime(RuntimeError::TypeError { .. }), ..}));

        Ok(())
    }

    #[test]
    fn test_that_cut_with_partial_errors() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if cut;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(1));
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(2));
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));
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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 1 and _this = 2");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(1));
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(2));
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));
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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a"), value!(2)])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(2));
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(1));
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));
        Ok(())
    }

    #[test]
    fn test_assignment_to_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x := 1;
               g(x) if x = 1 and y := x;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(error, PolarError {
            kind: ErrorKind::Runtime(RuntimeError::TypeError { .. }), ..}));

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 1");
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));
        Ok(())
    }

    #[test]
    fn nonlogical_inversions() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if not print(x);")?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert!(matches!(q.next_event()?, QueryEvent::Done { .. }));
        Ok(())
    }
}
