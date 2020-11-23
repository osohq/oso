use crate::counter::Counter;
use crate::error::{OperationalError, PolarResult};
use crate::events::QueryEvent;
use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Pattern, Symbol, Value};

#[derive(Clone)]
pub struct IsaConstraintCheck {
    existing: Vec<Operation>,
    proposed_tag: Option<Symbol>,
    result: Option<bool>,
    alternative_check: Option<QueryEvent>,
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
            alternative_check: None,
            last_call_id: 0,
        }
    }

    /// Check if existing constraints are compatible with the proposed type constraint (`_this
    /// matches Post{}`).
    ///
    /// If the existing constraint is also a type constraint , we return a pair of
    /// `QueryEvent::ExternalIsSubclass`es to check whether the type constraints are compatible.
    /// The constraints are compatible if they are the same class or if either is a subclass of the
    /// other.
    ///
    /// If the existing constraint is not a type constraint, there's no external check required,
    /// and we return `None` to indicate compatibility.
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
        if constraint.operator != Operator::Isa {
            return None;
        }

        let right = constraint.args.pop().unwrap();
        if let Value::Pattern(Pattern::Instance(instance)) = right.value() {
            let call_id = counter.next();
            self.last_call_id = call_id;

            let existing = instance.tag.clone();
            let proposed = self.proposed_tag.clone().unwrap();
            return Some((
                QueryEvent::ExternalIsSubclass {
                    call_id,
                    left_class_tag: proposed.clone(),
                    right_class_tag: existing.clone(),
                },
                QueryEvent::ExternalIsSubclass {
                    call_id,
                    left_class_tag: existing,
                    right_class_tag: proposed,
                },
            ));

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
