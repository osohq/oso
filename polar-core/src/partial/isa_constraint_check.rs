use crate::counter::Counter;
use crate::error::{OperationalError, PolarResult};
use crate::events::QueryEvent;

use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Pattern, Symbol, Term, Value};

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
    proposed_names: HashSet<Symbol>,
}

enum Check {
    None,
    One(QueryEvent),
    Two(QueryEvent, QueryEvent),
}

impl IsaConstraintCheck {
    pub fn new(
        existing: Vec<Operation>,
        proposed: Operation,
        proposed_names: HashSet<Symbol>,
    ) -> Self {
        Self {
            existing,
            proposed,
            result: None,
            alternative_check: None,
            last_call_id: 0,
            proposed_names,
        }
    }

    /// Check if existing constraints are compatible with the proposed constraint.
    ///
    /// If either the existing or proposed constraint is not a type constraint or if they are
    /// constraints for the same type, there's no external check required, and we return `None` to
    /// indicate compatibility.
    ///
    /// Otherwise, we return a collection of `QueryEvent`s to check whether the type
    /// constraints are compatible. The constraints are compatible if either of their types is a
    /// subclass of the other's.
    ///
    /// Returns:
    /// Zero, one or two query events.
    fn check_constraint(&mut self, constraint: Operation, counter: &Counter) -> Check {
        // TODO(gj): check non-`Isa` constraints, e.g., `(Unify, partial, 1)` against `(Isa,
        // partial, Integer)`.
        if constraint.operator != Operator::Isa {
            return Check::None;
        }

        let constraint_path = path(&constraint.args[0]);
        let proposed_path = path(&self.proposed.args[0]);

        // Not comparable b/c one of the matches statements has a LHS that isn't a variable or dot
        // op.
        if constraint_path.is_empty() || proposed_path.is_empty() {
            return Check::None;
        }

        let just_vars = constraint_path.len() == 1
            && proposed_path.len() == 1
            && matches!(&constraint.args[0].value().as_symbol(), Ok(Symbol(_)))
            && matches!(&self.proposed.args[0].value().as_symbol(), Ok(Symbol(_)));

        // FIXME(gw): this logic is hard to follow!
        if just_vars {
            let sym = constraint.args[0].value().as_symbol().unwrap();
            if !self.proposed_names.contains(sym) {
                return Check::None;
            }
        } else if constraint_path
            // a.b.c vs. d
            .iter()
            .zip(proposed_path.iter())
            .any(|(a, b)| a != b)
        // FIXME(gw): is this right? what if the first elements are aliases?
        {
            return Check::None;
        }

        let existing = constraint.args.last().unwrap();

        if constraint_path == proposed_path {
            // x matches A{} vs. x matches B{}
            self.subclass_compare(existing, counter)
        } else if constraint_path.len() < proposed_path.len() {
            // Proposed path is a superset of existing path.
            self.path_compare(proposed_path, constraint_path, existing, counter)
        } else if just_vars {
            self.subclass_compare(existing, counter)
        } else {
            // Comparing existing `x.a.b matches B{}` vs. `proposed x.a matches A{}`.
            Check::None
        }
    }

    fn subclass_compare(&mut self, existing: &Term, counter: &Counter) -> Check {
        let proposed = self.proposed.args.last().unwrap();
        match (proposed.value(), existing.value()) {
            (
                Value::Pattern(Pattern::Instance(proposed)),
                Value::Pattern(Pattern::Instance(existing)),
            ) if proposed.tag != existing.tag => {
                let call_id = counter.next();
                self.last_call_id = call_id;

                Check::Two(
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
                )
            }
            _ => Check::None,
        }
    }

    fn path_compare(
        &mut self,
        proposed_path: Vec<Term>,
        constraint_path: Vec<Term>,
        existing: &Term,
        counter: &Counter,
    ) -> Check {
        // given `a.b matches B{}` and `a.b.c.d matches D{}`, we want to assemble an
        // `ExternalIsaWithPath` of `B`, [c, d], and `D`.
        let proposed = self.proposed.args.last().unwrap();
        match (proposed.value(), existing.value()) {
            (
                Value::Pattern(Pattern::Instance(proposed)),
                Value::Pattern(Pattern::Instance(existing)),
            ) => {
                let call_id = counter.next();
                self.last_call_id = call_id;
                Check::One(QueryEvent::ExternalIsaWithPath {
                    call_id,
                    base_tag: existing.tag.clone(),
                    path: proposed_path[constraint_path.len()..].to_vec(),
                    class_tag: proposed.tag.clone(),
                })
            }
            _ => Check::None,
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

        // If there's an alternative waiting to be checked, check it.
        if let Some(alternative) = self.alternative_check.take() {
            return Ok(alternative);
        }

        let counter = counter.expect("IsaConstraintCheck requires a Counter");
        loop {
            match self.existing.pop() {
                None => return Ok(QueryEvent::Done { result: true }),
                Some(constraint) => match self.check_constraint(constraint, counter) {
                    Check::None => (),
                    Check::One(a) => return Ok(a),
                    Check::Two(a, b) => {
                        self.alternative_check = Some(b);
                        return Ok(a);
                    }
                },
            }
        }
    }

    fn external_question_result(&mut self, call_id: u64, answer: bool) -> PolarResult<()> {
        if call_id != self.last_call_id {
            return Err(OperationalError::InvalidState {
                msg: String::from("Unexpected call id"),
            }
            .into());
        }

        self.result = Some(answer);
        Ok(())
    }

    fn clone_runnable(&self) -> Box<dyn Runnable> {
        Box::new(self.clone())
    }
}
