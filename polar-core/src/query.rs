// TODO: Need to figure out how to do bindings correctly.
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

use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
    iter::{empty, once},
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{
    folder::Folder,
    kb::KnowledgeBase,
    terms::{
        Call, InstanceLiteral, List, Operation, Operator, Symbol, Term, ToPolarString, Value,
        Variable,
    },
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
    pub fn run(&self) -> impl Iterator<Item = HashMap<Symbol, Value>> {
        let Self {
            term,
            variables,
            kb,
        } = self;

        let variables = variables.clone();

        let state = State::new(kb.clone());
        term.clone().run(state).map(move |state| {
            assert_eq!(state.frames.len(), 1); // we should be left with _just_ global bindings
            println!("Get results: {{ {} }}", print_bindings(&state.bindings));
            variables
                .iter()
                .map(|v| {
                    let var = Variable::new(v.clone());
                    (
                        Symbol(v.clone()),
                        state
                            .deref_var(&var)
                            .map(|t| t.value().clone()) // convert to value
                            .unwrap_or_else(|| Value::Variable(var)), // default to an unbound variable (should be error?)
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
            let mut state = state.push_frame();
            let mut applicable = true;
            for (arg, param) in self.args.iter().zip(r.params.iter()) {
                // let arg = (&state).walk(arg.clone());
                if !state.unify(arg.clone(), param.parameter.clone()) {
                    applicable = false;
                    println!("Failed to unify: {} and {}", arg, param.parameter);
                    break;
                }
                if let Some(ref specializer) = param.specializer {
                    if !state.isa(arg.clone(), specializer.clone()) {
                        println!("Failed to isa: {} and {}", arg, specializer);
                        applicable = false;
                        break;
                    }
                }
            }
            if applicable {
                Box::new(r.body.clone().run(state).map(|mut state| {
                    state.bindings = state.frames.pop().unwrap();
                    state
                })) as Box<dyn Iterator<Item = State>>
            } else {
                Box::new(empty())
            }
        }))
    }
}

impl Goal for Term {
    type Results = Box<dyn Iterator<Item = State>>;
    fn run(self, mut state: State) -> Self::Results {
        println!("run term: {}", self);
        let term = state.deref(self);
        // println!("Derefed: {}", term);
        use Value::*;
        match term.value() {
            Call(call) => Box::new(call.clone().run(state)),
            Expression(op) => Box::new(op.clone().run(state)),
            Boolean(b) => {
                if *b {
                    Box::new(once(state))
                } else {
                    Box::new(empty())
                }
            }
            v => todo!("Implementing run for: {}", v.to_polar()),
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
            Isa => {
                if state.isa(self.args[0].clone(), self.args[1].clone()) {
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
            // TBD: is this breadth or depth first?
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
                let list = self.args.pop().unwrap();
                let item = self.args.pop().unwrap();
                let iter_state = state.clone();
                match list.value() {
                    Value::List(list) => {
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
                                        args: vec![item, term!(rv)],
                                    }
                                    .run(state),
                                ),
                            )
                        } else {
                            Box::new(elem_iter)
                        }
                    }
                    Value::Dictionary(dict) => {
                        Box::new(dict.fields.clone().into_iter().filter_map(move |(k, v)| {
                            let mut state = iter_state.clone();
                            let kv_list = list.clone_with_value(Value::List(List {
                                elements: vec![term!(k.0), v],
                                rest_var: None,
                            }));
                            if state.unify(item.clone(), kv_list) {
                                Some(state)
                            } else {
                                None
                            }
                        }))
                    }
                    Value::Variable(v) => {
                        todo!("cannot `in` with a variable for now")
                    }
                    _ => todo!("unsupported: in for: {}", list),
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

fn print_bindings(bindings: &HashMap<String, Term>) -> String {
    bindings
        .iter()
        .map(|(k, v)| format!("{} => {},", k, v))
        .collect::<Vec<String>>()
        .join("\n\t")
}

#[derive(Clone, Default)]
pub struct State {
    /// Stack of frames
    /// Zeroth entry reserved for global constants
    /// First entry should always be the query results
    frames: Vec<HashMap<String, Term>>,
    kb: Arc<RwLock<KnowledgeBase>>,
    /// Bindings (most recent frame)
    pub bindings: HashMap<String, Term>,
}

impl State {
    pub fn new(kb: Arc<RwLock<KnowledgeBase>>) -> Self {
        // seed the state with all registered constants
        let bindings = kb
            .read()
            .unwrap()
            .get_registered_constants()
            .iter()
            .map(|(k, v)| (k.0.clone(), v.clone()))
            .collect();

        Self {
            frames: vec![bindings], // instantiate list with constant bindings
            kb,
            bindings: Default::default(),
        }
    }

    pub fn push_frame(&self) -> Self {
        let mut new_state = self.clone();
        let mut bindings = Default::default();
        std::mem::swap(&mut new_state.bindings, &mut bindings);
        new_state.frames.push(bindings);
        new_state
    }

    pub fn deref(&mut self, term: Term) -> Term {
        Derefer::new(self).fold_term(term)
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

struct Derefer<'state> {
    state: &'state mut State,
    seen: HashSet<u64>,
}

impl<'a> Derefer<'a> {
    fn new(state: &'a mut State) -> Self {
        Self {
            state,
            seen: Default::default(),
        }
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl<'state> Folder for Derefer<'state> {
    fn fold_term(&mut self, t: Term) -> Term {
        match t.value() {
            Value::Variable(var) => {
                let hash_value = calculate_hash(&var);
                if !self.seen.insert(hash_value) {
                    panic!("circular reference detected in term: {}.", t,)
                }
                let derefed_var = self.state.deref_var_or_create(var, &t);
                let res = if matches!(derefed_var.value(), Value::Variable(_)) {
                    derefed_var
                } else {
                    self.fold_term(derefed_var)
                };
                self.seen.remove(&hash_value);
                res
            }
            Value::Expression(Operation {
                operator: Operator::Dot,
                args,
            }) => {
                if let Value::String(field) = args[1].value() {
                    match self.fold_term(args[0].clone()).value() {
                        Value::Dictionary(d) => self.fold_term(
                            d.fields
                                .get(&Symbol(field.clone()))
                                .expect("field not found")
                                .clone(),
                        ),
                        Value::InstanceLiteral(lit) => self.fold_term(
                            lit.fields
                                .fields
                                .get(&Symbol(field.clone()))
                                .expect("TODO: accessing a field on a literal that doesn't exist")
                                .clone(),
                        ),
                        _ => crate::folder::fold_term(t.clone(), self),
                    }
                } else {
                    todo!("support lookups: {}", t)
                }
            }
            Value::List(List {
                elements,
                rest_var: Some(rv),
            }) => {
                let derefed_rest_var = self.state.deref_var_or_create(rv, &t);
                match derefed_rest_var.value() {
                    Value::Variable(v) => t.clone_with_value(Value::List(List {
                        elements: elements.clone(),
                        rest_var: Some(v.clone()),
                    })),
                    Value::List(l) => {
                        // let l = self.fold_list(l.clone());
                        let mut elements = elements.clone();
                        elements.extend(l.elements.clone());
                        self.fold_term(t.clone_with_value(Value::List(List {
                            elements,
                            rest_var: l.rest_var.clone(),
                        })))
                    }
                    v => panic!("unexpected value for rest var: {}", v),
                }
            }

            _ => crate::folder::fold_term(t, self),
        }
    }
}

impl State {
    /// Deref var recursively follows all bindings until it gets to
    /// (a) a concrete value
    /// (b) the variable is pointing to itself
    /// (c) the variable is unbound (and it will promptly bind it to itself now)
    ///
    /// Originating term is used to get source info for any newly created variables
    fn deref_var(&self, var: &Variable) -> Option<Term> {
        // Dereference the variable exactly once
        let deref_once = |state: &State, var: &Variable| -> Option<Term> {
            if var.name.0 == "_" {
                // anonymous variables always count as unbound
                None
            } else if var.frame == usize::MAX {
                // Case 1: the variable frame is "unset" which means the
                // variable is coming from the AST

                // First check whether this is bound as a constant
                // Otherwise, look for a binding in the current frame
                state.frames[0]
                    .get(&var.name.0)
                    .or_else(|| state.bindings.get(&var.name.0))
                    .cloned()
            } else {
                // Otherwise, the variable has a frame set

                // Get the relevant bindings (TODO: is there any benefit having bindings on a separate variable?)
                let bindings = if var.frame == state.frames.len() {
                    &state.bindings
                } else {
                    &state.frames[var.frame]
                };

                if let Some(term) = bindings.get(&var.name.0) {
                    Some(term.clone())
                } else {
                    unreachable!("the variable {} claims it is bound to frame {}, but no such binding exists. Variable: {:#?}", var.name, var.frame, var)
                }
            }
        };

        let derefed = deref_once(self, var);
        match derefed.as_ref().map(|d| d.value()) {
            Some(Value::Variable(v)) if v != var => self.deref_var(v), // keep derefing
            _ => derefed,
        }
    }

    fn deref_var_or_create(&mut self, var: &Variable, originating_term: &Term) -> Term {
        if let Some(t) = self.deref_var(var) {
            t
        } else {
            let mut new_var = var.clone();
            new_var.frame = self.frames.len();
            let new_term = originating_term.clone_with_value(Value::Variable(new_var));
            // initialize this variable
            self.bindings.insert(var.name.0.clone(), new_term.clone());
            new_term
        }
    }

    fn bind(&mut self, var: &Variable, value: Term) {
        println!(
            "Bind: {} = {} [{}]",
            var.to_polar(),
            value,
            self.frames.len()
        );
        // if var.frame == usize::MAX {
        //     panic!("attempting to bind a variable without frame reference")
        // } else
        if var.frame < self.frames.len() {
            self.frames[var.frame].insert(var.name.0.to_string(), value);
        } else {
            self.bindings.insert(var.name.0.to_string(), value);
        }
    }

    fn unify(&mut self, left: Term, right: Term) -> bool {
        let left = self.deref(left);
        let right = self.deref(right);
        println!("Unify: {} = {}", left, right);
        match (left.value(), right.value()) {
            (left, right) if left == right => {
                println!("Exactly equal");
                true
            }
            (Value::Variable(left_var), Value::Variable(right_var)) => {
                // always bind the newest to the oldest
                if right_var.frame <= left_var.frame {
                    self.bind(left_var, right);
                } else {
                    self.bind(right_var, left);
                }
                true
            }
            (Value::Variable(var), _) => {
                self.bind(var, right);
                true
            }
            (_, Value::Variable(var)) => {
                self.bind(var, left);
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
        let left = self.deref(left);
        let right = self.deref(right);
        println!("Isa: {} matches {}", left, right);
        let tag_check = match (left.value(), right.value()) {
            (left, right) if left == right => return true, // identical values always isa to true
            // var isa Foo{...}
            (Value::Variable(var), Value::InstanceLiteral(lit)) => {
                if let Some(tag) = &var.type_info {
                    tag == &lit.tag.0
                } else {
                    let mut new_var = var.clone();
                    new_var.type_info = Some(lit.tag.0.clone());
                    self.bindings.insert(
                        var.name.0.clone(),
                        left.clone_with_value(Value::Variable(new_var)),
                    );
                    true
                }
            }
            (Value::Variable(_), Value::Dictionary(_))
            | (Value::Dictionary(_), Value::Dictionary(_)) => true,
            _ => false,
        };
        if !tag_check {
            return false;
        }

        // check fields
        match (left.value(), right.value()) {
            (left, right) if left == right => true,
            // var isa Foo{fields} or var isa {fields}
            (
                _,
                Value::InstanceLiteral(InstanceLiteral { fields, .. }) | Value::Dictionary(fields),
            ) => fields.fields.iter().all(|(k, v)| {
                // construct LHS as ${var}.{k}
                let lhs = left.clone_with_value(Value::Expression(Operation {
                    operator: Operator::Dot,
                    args: vec![left.clone(), term!(k.0.to_string())],
                }));
                self.isa(lhs, v.clone())
            }),
            _ => todo!("isa with a RHS of: {}", right),
        }
    }

    fn kb(&self) -> RwLockReadGuard<KnowledgeBase> {
        self.kb.read().unwrap()
    }
}
