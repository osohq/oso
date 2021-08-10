use crate::counter::Counter;
use crate::error::{OperationalError, PolarResult};
use crate::events::QueryEvent;
use crate::formatting::ToPolarString;
use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Pattern, Symbol, Term, Value};

use crate::kb::Bindings;

use std::collections::HashSet;

fn path(x: &Term) -> Vec<Term> {
    match x.value() {
        Value::Expression(Operation {
            operator: Operator::Dot,
            args,
        }) => [vec![args[0].clone()], path(&args[1])].concat(),
        _ => vec![x.clone()],
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
    pub fn new(
        existing: Vec<Operation>,
        proposed: Operation,
    ) -> Self {
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
    ) -> (Option<QueryEvent>, Option<QueryEvent>) {
        // TODO(gj): check non-`Isa` constraints, e.g., `(Unify, partial, 1)` against `(Isa,
        // partial, Integer)`.
        if constraint.operator != Operator::Isa {
            return (None, None);
        }

        let constraint_path = path(&constraint.args[0]);
        let proposed_path = path(&self.proposed.args[0]);
    //                println!("EXISC {:?} :: {:?} {:?}", self.proposed.operator, proposed_path, constraint_path);

        // Not comparable b/c one of the matches statements has a LHS that isn't a variable or dot
        // op.
        if constraint_path.is_empty() || proposed_path.is_empty() {
            return (None, None);
        }

        if constraint_path.len() == 1
            && proposed_path.len() == 1
            && matches!(&constraint.args[0].value().as_symbol(), Ok(Symbol(_)))
            && matches!(&self.proposed.args[0].value().as_symbol(), Ok(Symbol(_)))
        {
            let sym = constraint.args[0].value().as_symbol().unwrap();
            let proposed = self.proposed.args[0].value().as_symbol().unwrap();
        } else if constraint_path
            // a.b.c vs. d
            .iter()
            .zip(proposed_path.iter())
            .any(|(a, b)| a != b)
        {
            return (None, None);
        }

        // TODO(gj): why are we popping here?
        // let proposed = self.proposed.args.pop().unwrap();
        let proposed = self.proposed.args.last().unwrap();
        let existing = constraint.args.last().unwrap();

        // if DEBUGGING {
        //     eprintln!(
        //         "existing: {:?} {}, proposed: {:?} {}, ",
        //         constraint_path, existing.to_polar(),
        //         proposed_path, proposed.to_polar()
        //     );
        // }

        // x matches A{} vs. x matches B{}
        if constraint_path == proposed_path {
            match (proposed.value(), existing.value()) {
                (
                    Value::Pattern(Pattern::Instance(proposed)),
                    Value::Pattern(Pattern::Instance(existing)),
                ) if proposed.tag != existing.tag => {
                    let call_id = counter.next();
                    self.last_call_id = call_id;

                    (
                        Some(QueryEvent::ExternalIsSubclass {
                            call_id,
                            left_class_tag: proposed.tag.clone(),
                            right_class_tag: existing.tag.clone(),
                        }),
                        Some(QueryEvent::ExternalIsSubclass {
                            call_id,
                            left_class_tag: existing.tag.clone(),
                            right_class_tag: proposed.tag.clone(),
                        }),
                    )
                }
                _ => (None, None),
            }
        } else if constraint_path.len() < proposed_path.len() {
            // Proposed path is a superset of existing path. Take the existing tag, the additional
            // path segments from the proposed path, and the proposed tag.
            //
            // E.g., given `a.b matches B{}` and `a.b.c.d matches D{}`, we want to assemble an
            // `ExternalIsaWithPath` of `B`, [c, d], and `D`.
            match (proposed.value(), existing.value()) {
                (
                    Value::Pattern(Pattern::Instance(proposed)),
                    Value::Pattern(Pattern::Instance(existing)),
                ) => {
                    let call_id = counter.next();
                    self.last_call_id = call_id;
                    (
                        Some(QueryEvent::ExternalIsaWithPath {
                            call_id,
                            base_tag: existing.tag.clone(),
                            path: proposed_path[constraint_path.len()..].to_vec(),
                            class_tag: proposed.tag.clone(),
                        }),
                        None,
                    )
                }
                _ => (None, None),
            }
        } else {
            // Comparing existing `x.a.b matches B{}` vs. `proposed x.a matches A{}`.
            (None, None)
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
                //
                return Ok(QueryEvent::Done { result: false });
            }
        }

        let counter = counter.expect("IsaConstraintCheck requires a Counter");
        loop {
            // If there's an alternative waiting to be checked, check it.
            if let Some(alternative) = self.alternative_check.take() {
                return Ok(alternative);
            } else if let Some(constraint) = self.existing.pop() {
                let (maybe_primary, maybe_alternative) = self.check_constraint(constraint, counter);
                if let Some(alternative) = maybe_alternative {
                    self.alternative_check = Some(alternative);
                }
                if let Some(primary) = maybe_primary {
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
