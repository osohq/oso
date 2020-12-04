use serde::{Deserialize, Serialize};

use crate::runnable::Runnable;
use crate::terms::{
    Dictionary, InstanceLiteral, Operation, Operator, Pattern, Symbol, Term, Value,
};

use super::isa_constraint_check::IsaConstraintCheck;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Partial {
    pub constraints: Vec<Operation>,
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

impl Partial {
    pub fn new(variable: Symbol) -> Self {
        Self {
            constraints: vec![],
            variable,
        }
    }

    /// Augment our constraints with those on `other`.
    ///
    /// Invariant: both partials must have the same variable.
    pub fn merge_constraints(&mut self, other: Self) {
        assert_eq!(self.variable, other.variable);
        self.constraints.extend(other.constraints);
    }

    pub fn inverted_constraints(&self, csp: usize) -> Vec<Operation> {
        let (old, new) = self.constraints.split_at(csp);
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

    pub fn constraints(&self) -> &Vec<Operation> {
        &self.constraints
    }

    pub fn add_constraint(&mut self, o: Operation) {
        self.constraints.push(o);
    }

    pub fn unify(&mut self, other: Term) {
        let op = op!(Unify, self.variable_term(), other);
        self.add_constraint(op);
    }

    /// Add an isa constraint of the form `_this matches other`.
    ///
    /// ## Returns
    ///
    /// - Some(Runnable) if a compatibility check needs to be performed.
    ///   If the runnable succeeds, this partial should be used and the
    ///   query should continue.
    ///
    ///   If the runnable fails, the query should backtrack because the
    ///   existing constraints are incompatible with the proposed constraint.
    /// - None if no compatibility check needs to be performed and the partial
    ///   is compatible with the new constraint.
    pub fn isa(&mut self, other: Term) -> Option<Box<dyn Runnable>> {
        match other.value() {
            Value::Pattern(Pattern::Dictionary(fields)) => {
                // Add field constraints.
                for (field, value) in fields.fields.iter().rev() {
                    self.add_constraint(op!(
                        Unify,
                        term!(op!(Dot, self.variable_term(), term!(field.clone()))),
                        value.clone()
                    ));
                }
                None
            }
            Value::Pattern(Pattern::Instance(InstanceLiteral { fields, tag })) => {
                // Construct field-less Isa operation since field constraints will be added
                // separately.
                let tag_pattern = term!(pattern!(instance!(tag.clone())));
                let type_constraint = op!(Isa, self.variable_term(), tag_pattern);
                let existing = self.constraints.clone();
                let check = Box::new(IsaConstraintCheck::new(existing, type_constraint.clone()));

                self.add_constraint(type_constraint);

                // Add field constraints.
                let fields = term!(pattern!(fields.clone()));
                assert!(self.isa(fields).is_none());

                Some(check)
            }
            _ => {
                // Punt to unify for non-patterns.
                self.unify(other);
                None
            }
        }
    }

    /// Add a constraint that this must contain some known value.
    ///
    /// From OTHER in THIS where other is not a partial or variable.
    pub fn in_contains(&mut self, other: Term) {
        // TODO for now, this is represented as an in operation, but we may
        // want some other representation eventually.
        // TODO what about non-ground compound terms like [x, 1] in THIS

        assert!(!(matches!(other.value(), Value::Variable(_) | Value::Partial(_))));

        let in_op = op!(In, other, self.variable_term());
        self.add_constraint(in_op);
    }

    /// Add a constraint that a variable or partial value must be in this.
    ///
    /// `other` must be a partial or a variable.
    ///
    /// Returns: A new partial to use for additional constraints on `other`.
    pub fn in_unbound(&mut self, other: Term) -> Term {
        let name = match other.value() {
            Value::Partial(constraints) => constraints.name().clone(),
            Value::Variable(sym) => sym.clone(),
            _ => panic!(
                "Unexpected in LHS value {:?}, maybe you meant to call Constraints::contains()",
                other.value()
            ),
        };

        let in_op = op!(In, term!(name.clone()), self.variable_term());
        self.add_constraint(in_op);

        Term::new_temporary(Value::Partial(Partial::new(name)))
    }

    pub fn compare(&mut self, operator: Operator, operand: Operand) {
        use Operator::{Eq, Geq, Gt, Leq, Lt, Neq};
        let asymmetric_op = matches!(operator, Gt | Geq | Lt | Leq);
        let symmetric_op = matches!(operator, Eq | Neq);
        assert!(asymmetric_op || symmetric_op);

        // Normalize comparison operations so this is always on LHS.
        let args = vec![self.variable_term(), operand.term];
        let mut operation = Operation { operator, args };
        if operand.side == Side::Left && asymmetric_op {
            operation.operator = match operation.operator {
                Gt => Lt,
                Geq => Leq,
                Lt => Gt,
                Leq => Geq,
                _ => unreachable!(),
            };
        }
        self.add_constraint(operation);
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
        Term::new_temporary(Value::Partial(Partial::new(name.clone())))
    }

    pub fn into_term(self) -> Term {
        Term::new_temporary(Value::Partial(self))
    }

    pub fn into_expression(mut self) -> Term {
        if self.constraints.len() == 1 {
            Term::new_temporary(Value::Expression(self.constraints.pop().unwrap()))
        } else {
            Term::new_temporary(Value::Expression(Operation {
                operator: Operator::And,
                args: self
                    .constraints
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

    pub fn clone_with_constraints(&self, constraints: Vec<Operation>) -> Self {
        let mut new = self.clone();
        new.constraints = constraints;
        new
    }

    pub fn name(&self) -> &Symbol {
        &self.variable
    }

    fn variable_term(&self) -> Term {
        Term::new_temporary(Value::Variable(sym!("_this")))
    }
}

#[derive(PartialEq)]
enum Side {
    Left,
    Right,
}

/// Is the non-this arg the left or right operand?
pub struct Operand {
    side: Side,
    term: Term,
}

impl Operand {
    pub fn left(term: Term) -> Self {
        Self {
            side: Side::Left,
            term,
        }
    }

    pub fn right(term: Term) -> Self {
        Self {
            side: Side::Right,
            term,
        }
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
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_and() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y, z) if x = y and x = z;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a"), 1, 2])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 1 and _this = 2");
        assert_query_done!(q);
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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this matches Post{} and _this.foo = 1");
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this matches User{} and _this.bar = 1");
        assert_partial_expression!(
            next_binding(&mut q)?,
            "a",
            "_this matches Post{} and _this.y matches User{} and _this.y.z = 1"
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
        assert_partial_expression!(next_binding(), "a", "_this matches Post{} and _this.id = 1");
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this matches Post{} and _this.id = 1 and _this.id = 2"
        );
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this matches Post{} and _this.id = 1 and _this.id = 2"
        );
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this matches Post{} and _this.id = 1 and _this.id = 2 and _this.bar = 2"
        );
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this matches Post{} and _this.id = 1 and _this.bar = 3 and _this.id = 2 and _this.y = 1"
        );
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this.id = 1 and _this.bar = 1 and _this.id = 2"
        );
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this.id = 1 and _this.id = 2 and _this.bar = 2"
        );
        assert_partial_expression!(next_binding(), "a", "_this.id = 1");
        assert_partial_expression!(next_binding(), "a", "_this.id = 1 and _this.id = 2");
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this.id = 1 and _this matches Post{} and _this.id = 2"
        );
        assert_partial_expression!(next_binding(), "a", "_this = 1");
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
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this matches Foo{} and _this.y matches Bar{} and _this.y.z = 1"
        );
        assert_partial_expression!(
            next_binding(),
            "a",
            "_this matches Foo{} and _this.y matches Baz{} and _this.y.z = 1"
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
        let mut q = p.new_query_from_term(term!(call!("positive", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this > 0");
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this > 0 and _this < 0");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("zero", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this == 0");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_comparison_dot() -> TestResult {
        let p = Polar::new();
        p.load_str("positive(x) if x.a > 0 and 0 < x.a;")?;
        let mut q = p.new_query_from_term(term!(call!("positive", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.a > 0");
        assert_query_done!(q);
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
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "a",
            "_this.y = 0 and _this.y > 1 and _this.y.z > 1 and _this = 2"
        );
        assert_query_done!(q);
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
        assert_query_done!(q);
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
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this <= 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this != 1 or _this != 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this != 1 and _this != 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("j", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("k", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this != 1");
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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.foo != 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.foo != 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.foo.bar != 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.foo.bar != 1");
        assert_query_done!(q);
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
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this != 1 or _this = 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [partial!("a")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this = 1 and _this != 2 and _this != 3");
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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "");
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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this.foo != 1 and _this.foo != 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this != 1 and _this != 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [partial!("a")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "a", "_this.foo.bar != 1 and _this.foo.bar != 2");
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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_in_partial_lhs() -> TestResult {
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
        assert_query_done!(q);

        // Inverting an `in` produces a conjunction of the inverted disjunctive constraints.
        let mut q = p.new_query_from_term(term!(call!("not_lhs", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this != 1 and _this != 2");
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

        let mut q = p.new_query_from_term(term!(call!("contains", [1, partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "1 in _this");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("contains_dot", [1, partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "1 in _this.foo");
        assert_query_done!(q);

        let mut q =
            p.new_query_from_term(term!(call!("contains_dot_dot", [1, partial!("a")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "a",
            "1 in _this.foo.bar and _this.foo = 2"
        );
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_in_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if y in x.values;
               g(x, y) if y in x.values;
               h(x) if y in x.values and (y.bar = 1 and y.baz = 2) or y.bar = 3;"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        // TODO (dhatch): This doesn't work now, but ultimately this should have
        // no constraints since nothing is added to `y`.
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "a" => "() in _this.values",
            "_y_12" => ""
        );
        assert_query_done!(q);

        // Not sure about this one, where there's an output binding.  There are still
        // no constraints on b.
        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a"), partial!("b")])), false);
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "a" => "() in _this.values",
            "b" => "",
            "_y_17" => ""
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [partial!("a")])), false);
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "a" => "(_this.bar = 1 and _this.baz = 2) in _this.values"
        );
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "a" => "(_this.bar = 3) in _this.values"
        );
        assert_query_done!(q);

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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 1 and _this = 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a"), value!(2)])), false);
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
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(error, PolarError {
            kind: ErrorKind::Runtime(RuntimeError::TypeError { .. }), ..}));

        let mut q = p.new_query_from_term(term!(call!("g", [partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this = 1");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn nonlogical_inversions() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if not print(x);")?;
        let mut q = p.new_query_from_term(term!(call!("f", [partial!("a")])), false);
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_nested_dot_in() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y) if x in y.a.b.c;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [1, partial!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "1 in _this.a.b.c");
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

        let mut q = p.new_query_from_term(term!(call!("f", [1, partial!("a")])), false);

        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.a.b.c = 1");
        assert_partial_expression!(
            next_binding(&mut q)?,
            "a",
            "_this.a.b.c < 1 and _this.a.b > 1 and _this.a.b.c > 1"
        );
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.a = 1");
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.a.b = 1");
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.a.b.c.d = 1");
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.a.b.c.d.e = 1");
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this.a.b.c.d.e.f = 1");
        Ok(())
    }
}
