use std::{
    collections::HashMap,
    iter::{empty, once},
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{
    kb::KnowledgeBase,
    terms::{Call, List, Operation, Operator, Symbol, Term, ToPolarString, Value, Variable},
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

        let state = State::new(kb);
        term.run(state).map(move |state| {
            variables
                .iter()
                .map(|v| {
                    (
                        Symbol(v.clone()),
                        state
                            .bindings
                            .get(&0)
                            .unwrap()
                            .get(v) // get binding
                            .map(|t| state.walk(t.clone())) // walk to deref
                            .map(|t| t.value().clone()) // convert to value
                            .unwrap_or_else(|| Value::Variable(Variable::new(v.clone()))), // default to an unbound variable (should be error?)
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
        // TODO: walk the call either here, or in the Query goal to make sure that
        // we _only_ have frame-specific variables.
        let kb = state.kb.clone();
        let rules = state
            .kb()
            .get_generic_rule(&self.name)
            .expect(&format!("no matching rules for {}", self.name))
            .get_applicable_rules(&self.args);
        Box::new(rules.into_iter().flat_map(move |r| {
            println!("matching: {}", r);
            // for each applicable rule
            // create a set of bindings for the input arguments
            // and construct the goals needed to evaluate the rule
            let mut inner_state = State::new(kb.clone());
            let mut applicable = true;
            for (arg, param) in self.args.iter().zip(r.params.iter()) {
                let arg = (&state).walk(arg.clone());
                if !inner_state.unify(arg.clone(), param.parameter.clone()) {
                    applicable = false;
                    println!("Failed to unify: {} and {}", arg, param.parameter);
                    break;
                }
                if let Some(ref specializer) = param.specializer {
                    if !inner_state.isa(arg.clone(), specializer.clone()) {
                        println!("Failed to isa: {} and {}", arg, specializer);
                        applicable = false;
                        break;
                    }
                }
            }
            if applicable {
                let cloneable_state = state.clone();
                // run the body using the new frame (inner state)
                // then map the resultant state to recombine with the current frame (state)
                Box::new(r.body.clone().run(inner_state).map(move |inner_state| {
                    let mut new_state = cloneable_state.clone();
                    // TODO: could run this like query since we want to get a specific set of
                    // bindings out
                    // Also, check for any unresolved partials

                    // TODO: Need to figure out how to do bindings here correctly.
                    // I think the trick will be to walk every term that we see, so that we can ensure
                    // that all variables are instantiated to the earliest version of it.
                    // This sort of amounts to a rewrite.. But basically every time we see an unbound variable,
                    // we should deref it and replace with the framed version of a variable.
                    //
                    // e.g. f(x, y) if x in y;
                    //
                    // If we have y = 1 and f(y, [1, 2, 3]) we should get:
                    //              ^ y@0 := 1
                    //                        ^ x@1 := y@0
                    //                            ^ y@1 := [1, 2, 3]
                    //
                    // (inside rule, x in y) => x in y => x@1 in y@1 => y@0 in [1, 2, 3] => 1 in [1, 2, 3] => true
                    //
                    // If we have f(1, [x, y, z]) we should get:
                    //              ^ x@1 := 1
                    //                      ^ y@1 := [x@0, y@0, z@0]
                    //
                    // (inside rule, x in y) => x in y => x@1 in y@1 => 1 in [x@0, y@0, z@0] => x@0 = 1 or y@0 = 1 or z@0 = 1
                    // I think this implies we _always _pass by reference?
                    for (k, v) in new_state.bindings.get(&state.frame).unwrap().iter() {
                        state.bind(k, v.clone())
                    }
                    // for v in &variables {
                    //     new_state.bindings.insert(
                    //         v.clone(),
                    //         inner_state
                    //             .walk(inner_state.bindings.get(v).expect("must be bound").clone()),
                    //     );
                    // }
                    new_state
                })) as Box<dyn Iterator<Item = State>>
            } else {
                Box::new(empty())
            }
        }))
    }
}

impl Goal for Term {
    type Results = Box<dyn Iterator<Item = State>>;
    fn run(self, state: State) -> Self::Results {
        println!("run term: {}", self.to_polar());
        use Value::*;
        match self.value() {
            Call(call) => {
               Box::new(call.clone().run(state))
            }
            Expression(op) => Box::new(op.clone().run(state)),
            Boolean(b) => if *b {
                Box::new(once(state))
            } else {
                Box::new(empty())
            },
            v => todo!("Implementing run for: {}", v.to_polar())
            // Number(_) => todo!(),
            // String(_) => todo!(),
            // ExternalInstance(_) => todo!(),
            // Dictionary(_) => todo!(),
            // Pattern(_) => todo!(),
            // List(_) => todo!(),
            // Variable(_) => todo!(),
            // RestVariable(_) => todo!(),
        }
    }
}

impl Operation {
    fn run(mut self, mut state: State) -> Box<dyn Iterator<Item = State>> {
        use crate::terms::Operator::*;
        println!("run operation: {}", self.to_polar());
        match self.operator {
            Unify | Eq => {
                if state.unify(self.args[0].clone(), self.args[1].clone()) {
                    Box::new(once(state))
                } else {
                    Box::new(empty())
                }
            }
            // The `And` goal is constructed by sequentially chaining all state streams created by evaluating each
            // successive goal
            //
            // I.e. (x = 1 or x = 2) and (y = 3) first produces a stream of two states (x=1), (x=2)
            // and we append the result of running (y=3) onto each of these.
            //
            // TBD: is this breadth of depth first?
            And => Box::new(self.args.into_iter().fold(
                Box::new(once(state)) as Box<dyn Iterator<Item = State>>,
                |states, term| Box::new(states.flat_map(move |state| term.clone().run(state))),
            )),
            // The `Or` goal is constructed by cloning the state and creating an iterator for each goal
            Or => Box::new(
                self.args
                    .into_iter()
                    .flat_map(move |term| term.run(state.clone())),
            ),
            Not =>
            // this is not proper negation yet... but the idea is fail
            // if we get any results, and dont bind anything otherwise
            {
                if self.args.pop().unwrap().run(state.clone()).next().is_some() {
                    Box::new(empty())
                } else {
                    Box::new(once(state))
                }
            }

            In => {
                let list = state.walk(self.args.pop().unwrap());
                let item = state.walk(self.args.pop().unwrap());
                if let Value::List(list) = list.value() {
                    let iter_state = state.clone();
                    let iter_item = item.clone();
                    // attempt to unify item with each element
                    let elem_iter = list.elements.clone().into_iter().filter_map(move |elem| {
                        let mut state = iter_state.clone();
                        if state.unify(iter_item.clone(), elem) {
                            Some(state)
                        } else {
                            None
                        }
                    });
                    if let Some(rv) = &list.rest_var {
                        // if there's a rest var, the item could be in that list instead
                        // chain on those goals
                        Box::new(
                            elem_iter.chain(
                                Operation {
                                    operator: Operator::In,
                                    args: vec![item.clone(), term!(rv.clone())],
                                }
                                .run(state),
                            ),
                        )
                    } else {
                        Box::new(elem_iter)
                    }
                } else if let Value::Variable(v) = &list.value() {
                    todo!("cannot `in` with a variable for now")
                } else {
                    todo!("unsupported: in for: {}", list)
                }
            }
            Print => {
                println!(
                    "{}",
                    self.args
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                Box::new(once(state))
            }
            o => todo!("implementing run for operation {}", o.to_polar()),
        }
    }
}

use std::sync::atomic::{AtomicU64, Ordering};
pub static COUNTER: AtomicU64 = AtomicU64::new(2);

#[derive(Default)]
pub struct State {
    /// current frame. 0 is unset, 1 is global, locals start from 2
    pub frame: u64,
    kb: Arc<RwLock<KnowledgeBase>>,
    pub bindings: HashMap<u64, HashMap<String, Term>>,
}

impl Clone for State {
    fn clone(&self) -> Self {
        Self {
            frame: COUNTER.fetch_add(1, Ordering::SeqCst),
            kb: self.kb.clone(),
            bindings: self.bindings.clone(),
        }
    }
}

impl State {
    pub fn new(kb: Arc<RwLock<KnowledgeBase>>) -> Self {
        // seed the state with all registered constants
        let frame_bindings = kb
            .read()
            .unwrap()
            .get_registered_constants()
            .iter()
            .map(|(k, v)| (k.0.clone(), v.clone()))
            .collect();
        let frame = 0; // start out from 0
        let bindings = [(frame, frame_bindings)].into();
        Self {
            frame,
            kb,
            bindings,
        }
    }
}

/// A struct to represent a unify _goal_
///
/// The question: when do you use the goal versus calling unify directly?
/// There are two cases:
/// 1. You need to perform a unification after some other goal
/// 2. Unification might result in multiple new states
///
/// Currently (2) never happens. So always prefer to use the direct unification
/// for efficiency.
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
        println!(
            "Bindings: {{ {} }}",
            self.bindings
                .iter()
                .map(|(k, v)| format!("{} => {},", k, v))
                .collect::<Vec<String>>()
                .join("\n\t")
        );
        match term.value() {
            match_var!(var) => {
                match self.bindings.get(&var.0) {
                    Some(t) if t == &term => {
                        // var is unbound
                        t.clone()
                    }
                    Some(t) => {
                        let t = t.clone();
                        self.walk(t)
                    }
                    _ => term,
                }
            }
            Value::Expression(Operation {
                operator: Operator::Dot,
                args,
            }) => {
                if let Value::String(field) = args[1].value() {
                    match self.walk(args[0].clone()).value() {
                        Value::Dictionary(d) => d
                            .fields
                            .get(&Symbol(field.clone()))
                            .expect("field not found")
                            .clone(),
                        Value::InstanceLiteral(lit) => {
                            if let Some(v) = lit.fields.fields.get(&Symbol(field.clone())) {
                                v.clone()
                            } else {
                                todo!("accessing a field on a literal that doesn't exist")
                            }
                        }
                        _ => todo!("lookup on a non-literal"),
                    }
                } else {
                    todo!("support lookups: {}", term)
                }
            }
            _ => term,
        }
    }

    fn bind(&mut self, var: &str, value: Term) {
        println!("Bind: {} = {}", var, value);
        self.bindings
            .insert((self.frame_id, var.to_string()), value);
    }

    fn get_frame_binding(&self, frame_id: u64, var: &str) -> Option<&Term> {
        self.bindings.get(&(frame_id, var))
    }

    fn get_binding(&self, var: &str) -> Option<&Term> {
        self.get_frame_binding(self.frame, var)
    }

    fn unify(&mut self, left: Term, right: Term) -> bool {
        println!("Unify: {} = {}", left, right);

        match (self.walk(left).value(), self.walk(right).value()) {
            (left, right) if left == right => {
                println!("Exactly equal");
                true
            }
            (match_var!(var), value) | (value, match_var!(var)) => {
                self.bind(&var.0, term!(value.clone()));
                true
            }
            (Value::List(l), Value::List(r)) => self.unify_lists(l, r),
            (l, r) => {
                println!("Unify failed: {} = {}", l, r);
                false
            }
        }
    }

    fn unify_lists(&mut self, left: &List, right: &List) -> bool {
        match (
            left.elements.len(),
            &left.rest_var,
            right.elements.len(),
            &right.rest_var,
        ) {
            // make sure left <= right in length
            (l_len, _, r_len, _) if r_len < l_len => self.unify_lists(right, left),

            // equal lengths
            (l_len, lrv, r_len, rrv) if l_len == r_len => {
                let res = match (lrv, rrv) {
                    // left rest var and right rest_var are the same lists
                    // TODO: add list constraint to vars?
                    (Some(lrv), Some(rrv)) => self.unify(term!(lrv.clone()), term!(rrv)),
                    // rest var must be empty list
                    (Some(rv), None) | (None, Some(rv)) => self.unify(term!(rv.clone()), term!([])),
                    _ => true,
                };
                res && left
                    .elements
                    .iter()
                    .zip(right.elements.iter())
                    .all(|(l, r)| self.unify(l.clone(), r.clone()))
            }

            // l_len <= r_len since we swap
            (l_len, Some(lrv), _, rrv) => {
                let res = if let Some(rrv) = rrv {
                    // left rest var is the full suffix of the right
                    self.unify(
                        term!(lrv),
                        term!(List {
                            elements: right.elements[l_len..].to_vec(),
                            rest_var: Some(rrv.clone())
                        }),
                    )
                } else {
                    // left rest var is the suffix _and_ the rest var of right
                    self.unify(term!(lrv.clone()), term!(right.elements[l_len..].to_vec()))
                };
                res && left
                    .elements
                    .iter()
                    .zip(right.elements.iter())
                    .all(|(l, r)| self.unify(l.clone(), r.clone()))
            }
            _ => false,
        }
    }

    fn isa(&mut self, left: Term, right: Term) -> bool {
        use Value::*;
        let left = self.walk(left);
        match (left.value(), self.walk(right).value()) {
            (left, right) if left == right => true,
            // var isa Foo{...}
            (Variable(var), InstanceLiteral(lit)) => {
                if let Some(tag) = &var.type_info {
                    tag == &lit.tag.0
                } else {
                    let mut new_var = var.clone();
                    new_var.type_info = Some(lit.tag.0.clone());
                    self.bindings
                        .insert(var.name.0.clone(), left.clone_with_value(Variable(new_var)));
                    true
                }
                // TODO: isa fields too
            }
            _ => false,
        }
    }

    fn kb(&self) -> RwLockReadGuard<KnowledgeBase> {
        self.kb.read().unwrap()
    }
}
