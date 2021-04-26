mod isa_constraint_check;
#[allow(clippy::module_inception)]
mod partial;
mod simplify;

pub use isa_constraint_check::IsaConstraintCheck;
pub use simplify::{simplify_bindings, simplify_partial, sub_this};

use crate::terms::{Operation, Operator, Symbol, Term, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Partial {
    pub constraints: Vec<Operation>,
    // TODO: one or many references
    pub references: HashMap<Symbol, Partial>,
}

impl std::hash::Hash for Partial {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.constraints.hash(state);
        for (o, p) in self.references.iter() {
            o.hash(state);
            p.hash(state);
        }
    }
}

impl Partial {
    pub fn from_expression(expression: Operation) -> Self {
        if expression.operator == Operator::And {
            Self {
                constraints: expression
                    .args
                    .iter()
                    .map(|expr| expr.value().as_expression().unwrap().clone())
                    .collect(),
                references: Default::default(),
            }
        } else {
            Self {
                constraints: vec![expression],
                references: Default::default(),
            }
        }
    }

    pub fn into_expression(self) -> Operation {
        Operation {
            operator: Operator::And,
            args: self
                .constraints
                .into_iter()
                .map(|c| c.into_term())
                .collect(),
        }
    }
}
