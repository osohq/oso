use super::types::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Goal {
    Backtrack,
    Bind { variable: Symbol, value: Term },
    Bindings,
    Choice { choices: Vec<Goals>, bsp: usize }, // binding stack pointer
    Cut,
    TestExternal { name: Symbol }, // POC
    Halt,
    Isa { left: Term, right: Term },
    Lookup { instance: Instance, field: Term },
    LookupExternal { instance: Instance, field: Term },
    Query { predicate: Predicate },
    Result { name: Symbol, value: i64 },
    Unify { left: Term, right: Term },
}

// Stack of goals.
type Goals = Vec<Goal>;

#[derive(Debug)]
struct Binding(Symbol, Term);

#[derive(Default)]
pub struct PolarVirtualMachine {
    goals: Goals,
    bindings: Vec<Binding>,
    kb: KnowledgeBase,

    /// Used to track temporary variable names.
    genvar_counter: usize
}

impl PolarVirtualMachine {
    pub fn new(kb: KnowledgeBase, goals: Goals) -> Self {
        Self {
            goals,
            bindings: vec![],
            kb,
            genvar_counter: 0
        }
    }

    pub fn run(&mut self) -> QueryEvent {
        while let Some(goal) = self.goals.pop() {
            //println!("{:?}", goal);
            match goal {
                Goal::Backtrack => self.backtrack(),
                Goal::Bind { variable, value } => self.bind(&variable, &value),
                Goal::Bindings => {
                    return QueryEvent::Result {
                        bindings: self.bindings(),
                    };
                }
                Goal::Choice { choices, bsp } => self.choice(choices, bsp),
                Goal::Cut => self.cut(),
                Goal::TestExternal { name } => return QueryEvent::TestExternal { name }, // POC
                Goal::Halt => self.halt(),
                Goal::Isa { .. } => unimplemented!("isa"),
                Goal::Lookup { .. } => unimplemented!("lookup"),
                Goal::LookupExternal { .. } => unimplemented!("lookup external"),
                Goal::Query { predicate } => self.query(&predicate),
                Goal::Result { name, value } => self.result(&name, value),
                Goal::Unify { left, right } => self.unify(&left, &right),
            }
        }
        QueryEvent::Done
    }

    fn backtrack(&mut self) {
        while let Some(goal) = self.goals.pop() {
            match goal {
                Goal::Choice { choices, bsp } => {
                    self.bindings.drain(bsp..);
                    if choices.len() > 0 {
                        self.push_goal(Goal::Choice { choices, bsp });
                    }
                    break;
                }
                _ => (),
            }
        }
    }

    pub fn push_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
    }

    fn append_goals(&mut self, mut goals: Goals) {
        goals.reverse();
        self.goals.append(&mut goals);
    }

    fn query(&mut self, predicate: &Predicate) {
        // Select applicable rules for predicate.
        // Sort applicable rules by specificity.
        // Create a choice over the applicable rules.

        if let Some(generic_rule) = self.kb.rules.get(&predicate.name) {
            let generic_rule = generic_rule.clone();
            assert_eq!(generic_rule.name, predicate.name);

            let mut choices = vec![];
            for rule in &generic_rule.rules {
                // TODO: Should maybe parse these as terms.
                let var = Term::new(Value::List(rule.params.clone()));
                let val = Term::new(Value::List(predicate.args.clone()));

                choices.push(vec![
                    Goal::Unify {
                        left: var.clone(),
                        right: val.clone(),
                    },
                    Goal::Bindings,
                ]);
            }

            self.push_goal(Goal::Choice {
                choices,
                bsp: self.bindings.len(),
            });
        } else {
            self.push_goal(Goal::Backtrack)
        }
    }

    fn bindings(&mut self) -> Bindings {
        let mut bindings = HashMap::new();
        for binding in &self.bindings {
            bindings.insert(binding.0.clone(), binding.1.clone());
        }
        self.push_goal(Goal::Backtrack);
        bindings
    }

    fn cut(&mut self) {
        for goal in self.goals.iter_mut().rev() {
            match goal {
                Goal::Choice {
                    ref mut choices,
                    bsp: _,
                } => {
                    choices.clear();
                    break;
                }
                _ => (),
            }
        }
    }

    fn bind(&mut self, var: &Symbol, value: &Term) {
        //println!("{:?} ← {:?}", var, value);
        self.bindings.push(Binding(var.clone(), value.clone()));
    }

    // sp [[a,b,c][d,e,f]]
    // [[d,e,f] a, b, c]
    // [[] a, b, c, d, e, f]
    //
    // [a,b.c] [other choices later]

    fn choice(&mut self, mut choices: Vec<Goals>, bsp: usize) {
        if choices.len() > 0 {
            let choice = choices.remove(0);
            self.push_goal(Goal::Choice { choices, bsp });
            self.append_goals(choice);
        }
    }

    pub fn halt(&mut self) {
        self.goals.clear();
    }

    pub fn result(&mut self, name: &Symbol, value: i64) {
        self.push_goal(Goal::Bind {
            variable: name.clone(),
            value: Term::new(Value::Integer(value)),
        });
    }

    fn unify(&mut self, left: &Term, right: &Term) {
        match (&left.value, &right.value) {
            (Value::Symbol(_), _) => self.unify_var(left, right),
            (_, Value::Symbol(_)) => self.unify_var(right, left),
            (Value::List(left), Value::List(right)) => {
                if left.len() == right.len() {
                    self.append_goals(
                        left.iter()
                            .zip(right)
                            .map(|(left, right)| Goal::Unify {
                                left: left.clone(),
                                right: right.clone(),
                            })
                            .collect(),
                    )
                } else {
                    self.push_goal(Goal::Backtrack);
                }
            }
            (Value::Integer(left), Value::Integer(right)) => {
                if left != right {
                    self.push_goal(Goal::Backtrack);
                }
            }
            (_, _) => unimplemented!("unhandled unification {:?} = {:?}", left, right),
        }
    }

    fn unify_var(&mut self, left: &Term, right: &Term) {
        let left_sym = if let Value::Symbol(left_sym) = &left.value {
            left_sym
        } else {
            panic!("unify_var must be called with left as a Symbol");
        };

        if let Some(left_value) = self.value(&left_sym) {
            let left_value = left_value.clone();
            return self.push_goal(Goal::Unify { left: left_value, right: right.clone() });
        }

        if let Value::Symbol(right_sym) = &right.value {
            if let Some(right_value) = self.value(&right_sym) {
                let right_value = right_value.clone();
                return self.push_goal(Goal::Unify { left: left.clone(), right: right_value });
            }
        }

        self.push_goal(Goal::Bind {
            variable: left_sym.clone(),
            value: right.clone(),
        });
    }

    fn value(&self, variable: &Symbol) -> Option<&Term> {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.0 == *variable)
            .map(|binding| &binding.1)
    }

    /// Generate a new temporary for a name.
    fn genvar(&mut self, name: &str) -> Symbol {
        let counter = self.genvar_counter;
        self.genvar_counter += 1;

        Symbol(format!("_{name}_{counter}", name = name, counter = counter))
    }

    /// Rename variables in a term to generate a fresh set.
    fn rename_vars(&mut self, term: &Term) -> Term {
        term.map(&mut |value| {
            match value {
                Value::Symbol(sym) => {
                    Value::Symbol(self.genvar(&sym.0))
                },
                _ => value.clone()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind() {
        let x = Symbol("x".to_string());
        let y = Symbol("y".to_string());
        let zero = Term::new(Value::Integer(0));
        let mut vm = PolarVirtualMachine::new(
            KnowledgeBase::new(),
            vec![Goal::Bind {
                variable: x.clone(),
                value: zero.clone(),
            }],
        );
        vm.run();
        assert_eq!(vm.value(&x), Some(&zero));
        assert_eq!(vm.value(&y), None);
    }

    #[test]
    fn halt() {
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::new(), vec![Goal::Halt]);
        vm.run();
        assert_eq!(vm.goals.len(), 0);
        assert_eq!(vm.bindings.len(), 0);
    }

    #[test]
    fn unify() {
        let x = Symbol("x".to_string());
        let y = Symbol("y".to_string());
        let vars = Term::new(Value::List(vec![
            Term::new(Value::Symbol(x.clone())),
            Term::new(Value::Symbol(y.clone())),
        ]));
        let zero = Term::new(Value::Integer(0));
        let one = Term::new(Value::Integer(1));
        let vals = Term::new(Value::List(vec![zero.clone(), one.clone()]));
        let mut vm = PolarVirtualMachine::new(
            KnowledgeBase::new(),
            vec![Goal::Unify {
                left: vars.clone(),
                right: vals.clone(),
            }],
        );
        vm.run();
        assert_eq!(vm.value(&x), Some(&zero));
        assert_eq!(vm.value(&y), Some(&one));
    }

    #[test]
    fn unify_var() {
        let x = Value::Symbol(Symbol("x".to_string()));
        let y = Value::Symbol(Symbol("y".to_string()));
        let one = Term::new(Value::Integer(1));

        let mut vm = PolarVirtualMachine::new(
            KnowledgeBase::new(),
            vec![
                Goal::Unify {
                    left: Term::new(x),
                    right: Term::new(y),
                },
                Goal::Bind {
                    variable: Symbol("y".to_string()),
                    value: one.clone(),
                },
            ],
        );

        vm.run();

        // Left variable bound to bound right variable
        assert_eq!(vm.value(&Symbol("x".to_string())), Some(&one));

        // Left variable bound to value
        vm.append_goals(vec![
            Goal::Bind {
                variable: Symbol("z".to_string()),
                value: one.clone()
            },
            Goal::Unify {
                left: Term::new(Value::Symbol(Symbol("z".to_string()))),
                right: Term::new(Value::Integer(1))
            },
            // If unify failed, then backtrack instruction would throw away bind because
            // it pops stack until choice instruction is found.
            Goal::Bind {
                variable: Symbol("success".to_string()),
                value: one.clone()
            }
        ]);

        vm.run();

        assert_eq!(vm.value(&Symbol("success".to_string())), Some(&one));

        // Left variable bound to value
        vm.append_goals(vec![
            Goal::Bind {
                variable: Symbol("z".to_string()),
                value: one.clone()
            },
            Goal::Unify {
                left: Term::new(Value::Symbol(Symbol("z".to_string()))),
                right: Term::new(Value::Integer(2))
            },
            // If unify failed, then backtrack instruction would throw away bind because
            // it pops stack until choice instruction is found.
            Goal::Bind {
                variable: Symbol("not_success".to_string()),
                value: one.clone()
            }
        ]);

        vm.run();

        assert_ne!(vm.value(&Symbol("not_success".to_string())), Some(&one));
    }

    #[test]
    fn test_gen_var() {
        let term = Term::new(Value::List(vec![
            Term::new(Value::Integer(1)),
            Term::new(Value::Symbol(Symbol("x".to_string()))),
            Term::new(Value::List(vec![Term::new(Value::Symbol(Symbol("y".to_string())))])),
            ]));

        let mut vm = PolarVirtualMachine::default();

        let renamed_term = vm.rename_vars(&term);
        let x_value = match renamed_term.clone().value {
            Value::List(terms) => {
                match &terms[1].value {
                    Value::Symbol(sym) => {
                        Some(sym.0.clone())
                    },
                    _ => None
                }
            },
            _ => None
        };

        assert_eq!(x_value.unwrap(), "_x_0");

        let y_value = match renamed_term.value {
            Value::List(terms) => {
                match &terms[2].value {
                    Value::List(terms) => {
                        match &terms[0].value {
                            Value::Symbol(sym) => {
                                Some(sym.0.clone())
                            },
                            _ => None
                        }
                    },
                    _ => None
                }
            },
            _ => None
        };

        assert_eq!(y_value.unwrap(), "_y_1");
    }
}
