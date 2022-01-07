use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{
    kb::KnowledgeBase,
    terms::{Call, Operation, Operator, Symbol, Term, ToPolarString, Value},
};

pub struct Query {
    pub variables: Vec<String>,
    pub(crate) term: Term,
    pub kb: Arc<RwLock<KnowledgeBase>>,
}

pub struct Bindings {
    variables: HashMap<String, Term>,
}

trait Goal {
    type Results: Iterator<Item = State>;
    fn run(self, state: State) -> Self::Results;
}

impl Query {
    pub fn run(self) -> impl Iterator<Item = HashMap<Symbol, Value>> {
        let Self {
            term,
            variables,
            kb,
        } = self;
        let state = State {
            kb: kb.clone(),
            ..Default::default()
        };
        term.run(state).map(move |state| {
            variables
                .iter()
                .map(|v| {
                    (
                        Symbol(v.clone()),
                        state
                            .bindings
                            .get(v)
                            .map(|t| t.value().clone())
                            .unwrap_or_else(|| Value::Variable(Symbol(v.clone()))),
                    )
                })
                .collect()
        })
    }
}

impl Goal for Call {
    type Results = Box<dyn Iterator<Item = State>>;

    fn run(self, state: State) -> Self::Results {
        println!("run call: {}", self.to_polar());
        let kb = state.kb.clone();
        if let Some(gr) = state.kb().get_generic_rule(&self.name) {
            Box::new(
                gr.get_applicable_rules(&self.args)
                    .into_iter()
                    .flat_map(move |r| {
                        // for each applicable rule
                        // create a set of bindings for the input arguments
                        // and construct the goals needed to evaluate the rule
                        let bindings = HashMap::new();
                        let mut goals = vec![];
                        let mut inner_state = State {
                            bindings,
                            kb: kb.clone(),
                        };

                        let mut unify = true;
                        for (arg, param) in self.args.iter().zip(r.params.iter()) {
                            if !inner_state.unify(arg.clone(), param.parameter.clone()) {
                                unify = false;
                                break;
                            }
                            if let Some(ref specializer) = param.specializer {
                                goals.push(term!(op!(
                                    Isa,
                                    param.parameter.clone(),
                                    specializer.clone()
                                )))
                            }
                        }
                        if unify {
                            goals.push(r.body.clone());

                            // let match_goals = .flat_map(|(arg, param)| {
                            // })

                            Box::new(
                                term!(Value::Expression(Operation {
                                    operator: Operator::And,
                                    args: goals
                                }))
                                .run(inner_state),
                            ) as Box<dyn Iterator<Item = State>>
                        } else {
                            Box::new(std::iter::empty())
                        }
                    }),
            )
        } else {
            panic!("no rules for: {}", self.name);
            Box::new(std::iter::empty())
        }
    }
}

impl Goal for Term {
    type Results = Box<dyn Iterator<Item = State>>;
    fn run(self, state: State) -> Self::Results {
        println!("run term: {}", self.to_polar());
        use Value::*;
        match self.value() {
            Call(call) => {
               return Box::new(call.clone().run(state))
            }
            Expression(op) => return Box::new(op.clone().run(state)),
            v => todo!("Implementing query for: {}", v.to_polar())
            // Number(_) => todo!(),
            // String(_) => todo!(),
            // Boolean(_) => todo!(),
            // ExternalInstance(_) => todo!(),
            // Dictionary(_) => todo!(),
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
        println!("run operation: {}", self.to_polar());
        match self.operator {
            Unify | Eq => {
                if self.args[0] == self.args[1] {
                    Box::new(std::iter::once(state))
                } else {
                    Box::new(std::iter::empty())
                }
            }
            And => Box::new(self.args.into_iter().fold(
                Box::new(std::iter::once(state)) as Box<dyn Iterator<Item = State>>,
                |states, term| Box::new(states.flat_map(move |state| term.clone().run(state))),
            )),
            o => todo!("implementing run for operation {}", o.to_polar()),
        }
    }
}

#[derive(Clone, Default)]
pub struct State {
    kb: Arc<RwLock<KnowledgeBase>>,
    pub bindings: HashMap<String, Term>,
}

struct Unify {
    left: Term,
    right: Term,
}

impl Goal for Unify {
    type Results = std::vec::IntoIter<State>;

    fn run(self, mut state: State) -> Self::Results {
        if state.unify(self.left, self.right) {
            vec![state].into_iter()
        } else {
            vec![].into_iter()
        }
    }
}

impl State {
    fn walk(&self, term: Term) -> Term {
        match term.value() {
            Value::Variable(var) => {
                if let Some(t) = self.bindings.get(&var.0) {
                    self.walk(t.clone())
                } else {
                    term
                }
            }
            _ => term,
        }
    }

    fn unify(&mut self, left: Term, right: Term) -> bool {
        match (self.walk(left).value(), self.walk(right).value()) {
            (left, right) if left == right => true,
            (Value::Variable(var), value) | (value, Value::Variable(var)) => {
                self.bindings
                    .insert(var.0.clone(), Term::new_temporary(value.clone()));
                true
            }
            _ => false,
        }
    }

    fn kb(&self) -> RwLockReadGuard<KnowledgeBase> {
        self.kb.read().unwrap()
    }
}
// impl State {
//     pub fn bindings(&self) -> HashMap<Symbol, Value> {
//         self.bindings
//             .iter()
//             .map(|(k, v)| (Symbol(k.clone()), v.value().clone()))
//             .collect()
//     }
// }
