use std::{collections::HashMap, sync::Arc};

use crate::{
    kb::KnowledgeBase,
    terms::{Operation, Symbol, Term, ToPolarString, Value},
};

pub struct Query {
    pub(crate) term: Term,
}

pub struct Bindings {
    variables: HashMap<String, Term>,
}

impl Query {
    pub fn run(self, state: State) -> Box<dyn Iterator<Item = State>> {
        use Value::*;
        match self.term.value() {
            Expression(op) => return Box::new(op.clone().run(state)),
            v => todo!("Implementing query for: {}", v.to_polar())
            // Number(_) => todo!(),
            // String(_) => todo!(),
            // Boolean(_) => todo!(),
            // ExternalInstance(_) => todo!(),
            // Dictionary(_) => todo!(),
            // Call(_) => todo!(),
            // Pattern(_) => todo!(),
            // List(_) => todo!(),
            // Variable(_) => todo!(),
            // RestVariable(_) => todo!(),
        }

        Box::new(std::iter::empty())
    }
}

impl Operation {
    fn run(self, state: State) -> Box<dyn Iterator<Item = State>> {
        use crate::terms::Operator::*;
        match self.operator {
            Eq => return Box::new(std::iter::once(state)),
            o => todo!("implementing run for operation {}", o.to_polar()),
        }
        Box::new(std::iter::empty())
    }
}

#[derive(Clone, Default)]
pub struct State {
    kb: Arc<KnowledgeBase>,
    pub bindings: HashMap<String, Term>,
}

impl State {
    pub fn bindings(&self) -> HashMap<Symbol, Value> {
        self.bindings
            .iter()
            .map(|(k, v)| (Symbol(k.clone()), v.value().clone()))
            .collect()
    }
}
