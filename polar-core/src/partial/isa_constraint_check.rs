use crate::counter::Counter;
use crate::error::{OperationalError, PolarResult};
use crate::events::QueryEvent;
use crate::formatting::ToPolarString;
use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Pattern, Symbol, Term, Value};

fn base(x: &Term, depth: usize) -> Option<(Symbol, usize)> {
    match x.value() {
        Value::Variable(v) => Some((v.clone(), depth)),
        Value::Expression(Operation {
            operator: Operator::Dot,
            args,
        }) => base(&args[0], depth + 1),
        _ => None,
    }
}

#[derive(Clone)]
pub struct IsaConstraintCheck {
    existing: Vec<Operation>,
    proposed: Operation,
    result: Option<bool>,
    alternative_check: Option<QueryEvent>,
    last_call_id: u64,
}

impl IsaConstraintCheck {
    pub fn new(mut existing: Vec<Operation>) -> Self {
        let proposed = existing.pop().unwrap();
        Self {
            existing,
            proposed,
            result: None,
            alternative_check: None,
            last_call_id: 0,
        }
    }

    /// Check if existing constraints are compatible with the proposed constraint.
    ///
    /// If either the existing or proposed constraint is not a type constraint or if they are
    /// constraints for the same type, there's no external check required, and we return `None` to
    /// indicate compatibility.
    ///
    /// Otherwise, we return a pair of `QueryEvent::ExternalIsSubclass`es to check whether the type
    /// constraints are compatible. The constraints are compatible if either of their types is a
    /// subclass of the other's.
    ///
    /// Returns:
    /// - `None` if compatible.
    /// - A pair of `QueryEvent::ExternalIsSubclass` checks if compatibility cannot be determined
    /// locally.
    fn check_constraint(
        &mut self,
        mut constraint: Operation,
        counter: &Counter,
    ) -> Option<(QueryEvent, QueryEvent)> {
        // TODO(gj): check non-`Isa` constraints, e.g., `(Unify, partial, 1)` against `(Isa,
        // partial, Integer)`.
        eprintln!(
            "check_constraint => constraint: {} @@@@@@@@@@@@ proposed: {}",
            constraint.to_polar(),
            self.proposed.to_polar()
        );
        if constraint.operator != Operator::Isa {
            return None;
        }

        let constraint_base = base(&constraint.args[0], 0);
        let proposed_base = base(&self.proposed.args[0], 0);

        if constraint_base.is_none() || proposed_base.is_none() {
            return None;
        }

        let (constraint_base, constraint_depth) = constraint_base.unwrap();
        let (proposed_base, proposed_depth) = proposed_base.unwrap();

        if constraint_base != proposed_base {
            return None;
        }

        if constraint.args[0] == self.proposed.args[0] {
            let proposed = self.proposed.args.pop().unwrap();
            let existing = constraint.args.pop().unwrap();
            match (proposed.value(), existing.value()) {
                (
                    Value::Pattern(Pattern::Instance(proposed)),
                    Value::Pattern(Pattern::Instance(existing)),
                ) if proposed.tag != existing.tag => {
                    let call_id = counter.next();
                    self.last_call_id = call_id;

                    Some((
                        QueryEvent::ExternalIsSubclass {
                            call_id,
                            left_class_tag: proposed.tag.clone(),
                            right_class_tag: existing.tag.clone(),
                        },
                        QueryEvent::ExternalIsSubclass {
                            call_id,
                            left_class_tag: existing.tag.clone(),
                            right_class_tag: proposed.tag.clone(),
                        },
                    ))
                }
                _ => None,
            }
        } else if constraint_depth > proposed_depth {
            panic!("AAAAAAAAAAAAAAAAAAAA");
        } else {
            let call_id = counter.next();
            self.last_call_id = call_id;

            Some((
                QueryEvent::ExternalIsa {
                    call_id,
                    instance: op!(
                        And,
                        constraint.clone().into_term(),
                        self.proposed.clone().into_term()
                    )
                    .into_term(),
                    class_tag: sym!(""),
                },
                QueryEvent::ExternalIsa {
                    call_id,
                    instance: op!(
                        And,
                        constraint.into_term(),
                        self.proposed.clone().into_term()
                    )
                    .into_term(),
                    class_tag: sym!(""),
                },
            ))
        }
    }
}

impl Runnable for IsaConstraintCheck {
    fn run(&mut self, counter: Option<&mut Counter>) -> PolarResult<QueryEvent> {
        if let Some(result) = self.result.take() {
            if result {
                // If the primary check succeeds, there's no need to check the alternative.
                self.alternative_check = None;
            } else if self.alternative_check.is_none() {
                // If both checks fail, we fail.
                return Ok(QueryEvent::Done { result: false });
            }
        }

        let counter = counter.expect("IsaConstraintCheck requires a Counter");
        loop {
            // If there's an alternative waiting to be checked, check it.
            if let Some(alternative) = self.alternative_check.take() {
                return Ok(alternative);
            } else if let Some(constraint) = self.existing.pop() {
                if let Some((primary, alternative)) = self.check_constraint(constraint, &counter) {
                    self.alternative_check = Some(alternative);
                    return Ok(primary);
                }
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
