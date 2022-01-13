use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
};

use crate::{
    folder::Folder,
    kb::KnowledgeBase,
    terms::{List, Operation, Operator, Symbol, Term, ToPolarString, Value, Variable},
};

#[derive(Clone, Default)]
pub struct BindingManager {
    frames: Vec<HashMap<String, Term>>,
    bindings: HashMap<String, Term>,
}

impl BindingManager {
    pub fn new_with_kb(kb: Arc<RwLock<KnowledgeBase>>) -> Self {
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
            bindings: Default::default(),
        }
    }

    pub fn push_frame(&mut self) {
        let mut bindings = Default::default();
        std::mem::swap(&mut self.bindings, &mut bindings);
        self.frames.push(bindings);
    }

    pub fn pop_frame(&mut self) {
        self.bindings = self.frames.pop().unwrap();
    }

    pub fn bind(&mut self, var: &Variable, value: Term) {
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

    pub fn deref(&mut self, term: Term) -> Term {
        Derefer::new(self).fold_term(term)
    }

    pub fn get_bindings(&self, variables: &[String]) -> HashMap<Symbol, Value> {
        assert_eq!(self.frames.len(), 1); // we should be left with _just_ global bindings
        println!("Get results: {{ {} }}", print_bindings(&self.bindings));
        variables
            .iter()
            .map(|v| {
                let var = Variable::new(v.clone());
                (
                    Symbol(v.clone()),
                    self.deref_var(&var)
                        .map(|t| t.value().clone()) // convert to value
                        .unwrap_or_else(|| Value::Variable(var)), // default to an unbound variable (should be error?)
                )
            })
            .collect()
    }

    /// Deref var recursively follows all bindings until it gets to
    /// (a) a concrete value
    /// (b) the variable is pointing to itself
    /// (c) the variable is unbound (and it will promptly bind it to itself now)
    ///
    /// Originating term is used to get source info for any newly created variables
    fn deref_var(&self, var: &Variable) -> Option<Term> {
        // Dereference the variable exactly once
        let deref_once = |bm: &Self, var: &Variable| -> Option<Term> {
            if var.name.0 == "_" {
                // anonymous variables always count as unbound
                None
            } else if var.frame == usize::MAX {
                // Case 1: the variable frame is "unset" which means the
                // variable is coming from the AST

                // First check whether this is bound as a constant
                // Otherwise, look for a binding in the current frame
                bm.frames[0]
                    .get(&var.name.0)
                    .or_else(|| bm.bindings.get(&var.name.0))
                    .cloned()
            } else {
                // Otherwise, the variable has a frame set

                // Get the relevant bindings (TODO: is there any benefit having bindings on a separate variable?)
                let bindings = if var.frame == bm.frames.len() {
                    &bm.bindings
                } else {
                    &bm.frames[var.frame]
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
}

struct Derefer<'bm> {
    bm: &'bm mut BindingManager,
    seen: HashSet<u64>,
}

impl<'a> Derefer<'a> {
    fn new(bm: &'a mut BindingManager) -> Self {
        Self {
            bm,
            seen: Default::default(),
        }
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl<'bm> Folder for Derefer<'bm> {
    fn fold_term(&mut self, t: Term) -> Term {
        match t.value() {
            Value::Variable(var) => {
                let hash_value = calculate_hash(&var);
                if !self.seen.insert(hash_value) {
                    panic!("circular reference detected in term: {}.", t,)
                }
                let derefed_var = self.bm.deref_var_or_create(var, &t);
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
                let derefed_rest_var = self.bm.deref_var_or_create(rv, &t);
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

fn print_bindings(bindings: &HashMap<String, Term>) -> String {
    bindings
        .iter()
        .map(|(k, v)| format!("{} => {},", k, v))
        .collect::<Vec<String>>()
        .join("\n\t")
}
