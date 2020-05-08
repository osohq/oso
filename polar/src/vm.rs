use std::collections::HashMap;
use std::fmt;

use super::types::*;

#[must_use = "goals don't do anything unless they are used"]
#[derive(Debug)]
pub enum Goal {
    Backtrack,
    Bind {
        variable: Symbol,
        value: Term,
    },
    Bindings,
    Choice {
        choices: Vec<Goals>,
        bsp: usize,
    }, // binding stack pointer
    Cut,
    TestExternal {
        name: Symbol,
    }, // POC
    Halt,
    Isa {
        left: Term,
        right: Term,
    },
    Lookup {
        instance: Instance,
        field: Term,
    },
    LookupExternal {
        instance: Instance,
        field: Term,
    },
    Query {
        predicate: Predicate,
        tail: Vec<Predicate>,
    },
    Result {
        name: Symbol,
        value: Option<i64>,
    },
    Unify {
        left: Term,
        right: Term,
    },
}

impl fmt::Display for Goal {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Goal::Bind { variable, value } => write!(
                fmt,
                "Bind {{ {} := {} }}",
                variable.to_polar(),
                value.to_polar()
            ),
            Goal::Choice { choices, bsp } => write!(
                fmt,
                "Choice {{ bsp: {}, choices: [{}] }}",
                bsp,
                choices
                    .iter()
                    .map(|goals| format!(
                        "[{}]",
                        goals
                            .iter()
                            .map(|g| format!("{}", g))
                            .collect::<Vec<String>>()
                            .join(", ")
                    ))
                    .collect::<Vec<String>>()
                    .join("\n\t")
            ),
            Goal::Unify { left, right } => write!(
                fmt,
                "Unify {{ left: {}, right: {} }}",
                left.to_polar(),
                right.to_polar()
            ),
            g => write!(fmt, "{:?}", g),
        }
    }
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
    genvar_counter: usize,
}

// Methods which aren't goals/instructions
impl PolarVirtualMachine {
    /// Push a new goal onto the stack
    pub fn push_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
    }

    /// Push multiple goals onto the stack (in reverse order)
    fn append_goals(&mut self, mut goals: Goals) {
        goals.reverse();
        self.goals.append(&mut goals);
    }

    pub fn new(kb: KnowledgeBase, goals: Goals) -> Self {
        Self {
            goals,
            bindings: vec![],
            kb,
            genvar_counter: 0,
        }
    }

    pub fn run(&mut self) -> QueryEvent {
        while let Some(goal) = self.goals.pop() {
            eprintln!(
                "{} stack [{}]\n",
                goal,
                self.goals
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            );
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
                Goal::TestExternal { name } => return self.test_external(name), // POC
                Goal::Halt => self.halt(),
                Goal::Isa { .. } => unimplemented!("isa"),
                Goal::Lookup { .. } => unimplemented!("lookup"),
                Goal::LookupExternal { .. } => unimplemented!("lookup external"),
                Goal::Query { predicate, tail } => self.query(predicate, tail),
                Goal::Result { name, value } => self.result(&name, value),
                Goal::Unify { left, right } => self.unify(&left, &right),
            }
        }
        QueryEvent::Done
    }

    /// Unifies a symobol `var` with a term `value`.
    ///
    /// This is sort of a "sub-goal" of `Unify`
    fn unify_var(&mut self, left: &Symbol, right: &Term) {
        let left_value = self.value(&left).cloned();
        let mut right_value = None;
        if let Value::Symbol(ref right_sym) = right.value {
            right_value = self.value(right_sym).cloned();
        }

        match (left_value, right_value) {
            (Some(left), Some(right)) => {
                // both are bound, unify these as values
                self.push_goal(Goal::Unify { left, right });
            }
            (Some(left), _) => {
                // only left is bound, unify with whatever right is
                self.push_goal(Goal::Unify {
                    left,
                    right: right.clone(),
                });
            }
            (None, Some(value)) => {
                // left is unbound, right is bound
                // bind left to the value of right
                // TODO: could merge this branch and the (None, None)
                // branch, but this avoids an additional goal
                self.push_goal(Goal::Bind {
                    variable: left.clone(),
                    value,
                });
            }
            (None, None) => {
                // neither is bound, so bind these together
                // TODO: should theoretically bind the earliest one here?
                self.push_goal(Goal::Bind {
                    variable: left.clone(),
                    value: right.clone(),
                });
            }
        }
    }

    /// Looks up a variable from the bindings and returns a
    /// reference to the term
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
        term.map(&mut |value| match value {
            Value::Symbol(sym) => Value::Symbol(self.genvar(&sym.0)),
            _ => value.clone(),
        })
    }
}

// Implementations of instructions
impl PolarVirtualMachine {
    /// Backtrack from the current goal, stopping once we reach
    /// a choice point with no more choices (which means this)
    ///
    /// Remove all bindings that were added after any choices points
    fn backtrack(&mut self) {
        while let Some(goal) = self.goals.pop() {
            match goal {
                Goal::Choice {
                    ref choices,
                    ref bsp,
                } => {
                    self.bindings.drain(bsp..);
                    // `is_empty` is more idiomatic
                    if choices.len() > 0 {
                        self.push_goal(goal);
                    }
                    break;
                }
                _ => (),
            }
        }
    }

    /// Binds a value to a variable
    /// I.e. directly "writes" to memory
    fn bind(&mut self, var: &Symbol, value: &Term) {
        //println!("{:?} ← {:?}", var, value);
        self.bindings.push(Binding(var.clone(), value.clone()));
    }

    /// Retrieves all the current bindings, pausing the VM and returning
    /// the value of bindings
    ///
    /// @TODO(?) Currently, this also pushes a `Backtrack` goal onto the stack
    /// because bindings is used to indicate a query branch has solved
    fn bindings(&mut self) -> Bindings {
        let mut bindings = HashMap::new();
        for binding in &self.bindings {
            bindings.insert(binding.0.clone(), binding.1.clone());
        }
        self.push_goal(Goal::Backtrack);
        bindings
    }

    // sp [[a,b,c][d,e,f]]
    // [[d,e,f] a, b, c]
    // [[] a, b, c, d, e, f]
    //
    // [a,b.c] [other choices later]

    /// Resolves a choice by removing the next list of goals
    /// and adding them to the stack.
    ///
    /// If there are no more choices, then this goal
    /// is a no-op
    fn choice(&mut self, mut choices: Vec<Goals>, bsp: usize) {
        if choices.len() > 0 {
            let choice = choices.remove(0);
            self.push_goal(Goal::Choice { choices, bsp });
            self.append_goals(choice);
        }
    }

    /// A cut operation indicates that no other choices should
    /// be considered.
    ///
    /// This goal implements this by iterating through all
    /// goals and clearing all other choice branches.
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

    /// Test goal for waiting for external input
    ///
    /// Pushes a `Halt` goal onto the stack so the
    /// program terminates if we don't get a response
    ///
    /// Also pushes another `TestExternal` goal for the same symbol
    /// so that we continue to poll for more results
    fn test_external(&mut self, name: Symbol) -> QueryEvent {
        // push another test external goal to handle any further results
        self.push_goal(Goal::TestExternal { name: name.clone() });
        // halt if we don't get back a response from the application
        self.push_goal(Goal::Halt);
        QueryEvent::TestExternal { name }
    }

    /// Halts the VM by clearing all goals
    pub fn halt(&mut self) {
        self.goals.clear();
    }

    // pub fn isa(&mut self) {}
    // pub fn lookup(&mut self) {}
    // pub fn lookup_external(&mut self) {}

    /// Queries for the provided predicate.
    ///
    /// Uses the knowledge base to get an ordered list of rules
    /// Creates a choice point over each rule, where the choice point
    /// consists of (a) unifying the arguments (unifying the head), and
    /// (b) appending all clauses from the body onto the query
    /// and (c) concluding the query with a new `Bindings` goal if there
    /// are no further causes, or creating a new query otherwise
    fn query(&mut self, predicate: Predicate, tail: Vec<Predicate>) {
        // Select applicable rules for predicate.
        // Sort applicable rules by specificity.
        // Create a choice over the applicable rules.

        if let Some(generic_rule) = self.kb.rules.get(&predicate.name) {
            let generic_rule = generic_rule.clone();
            assert_eq!(generic_rule.name, predicate.name);

            let mut choices = vec![];
            for rule in generic_rule.rules {
                // TODO: Should maybe parse these as terms.
                let var = Term::new(Value::List(rule.params.clone()));
                let val = Term::new(Value::List(predicate.args.clone()));

                // (a) First goal is to unify the heads
                let mut goals = vec![Goal::Unify {
                    left: var.clone(),
                    right: val.clone(),
                }];

                let mut tail = tail.clone();

                // (b) add clauses from body to query tail
                for clause in rule.body.into_iter() {
                    if let Value::Call(pred) = clause.value {
                        tail.push(pred);
                    } else {
                        todo!("can clauses in a rule body be anything but predicates?")
                    }
                }

                if tail.is_empty() {
                    // (c) this predicate is the last goal; return bindings
                    goals.push(Goal::Bindings);
                // @TODO (and backtrack, if we move it from `bindings` method)
                } else {
                    // (c) create a new query with the new list of predicates
                    let predicate = tail.remove(0);
                    goals.push(Goal::Query { predicate, tail });
                }
                choices.push(goals)
            }

            self.push_goal(Goal::Choice {
                choices,
                bsp: self.bindings.len(),
            });
        } else {
            // no applicable rules, so backtrack to the last choice point
            self.push_goal(Goal::Backtrack)
        }
    }

    /// Signifies that a new result has been provided
    ///
    /// If the value is `Some(_)` then we have a result and
    /// add a new `Bind` goal to bind the symbol to the value
    ///
    /// If the value is `None` then the external has no (more)
    /// results, so we make sure to clear the trailing `TestExternal`
    /// goal that would otherwise follow
    pub fn result(&mut self, name: &Symbol, value: Option<i64>) {
        // externals are always followed by a halt
        assert!(matches!(self.goals.pop(), Some(Goal::Halt)));

        if let Some(value) = value {
            // we have a value and should bind
            self.push_goal(Goal::Bind {
                variable: name.clone(),
                value: Term::new(Value::Integer(value)),
            });
        } else {
            // no more values, so no further queries to resolve
            assert!(matches!(
                self.goals.pop(),
                Some(Goal::TestExternal { name }) if name == name
            ));
        }
    }

    /// The `Unify` goal attempts to unify `left` and `right` terms
    ///
    /// This is effectively going to be a giant match statement.
    ///
    /// Outcomes of a unification are:
    ///  - Successful unification => a `Bind` goal is created for a symbol
    ///  - Recursive unification => more `Unify` goals are created
    ///  - Failure => this branch is false, and a `Backtrack` goal is created
    fn unify(&mut self, left: &Term, right: &Term) {
        match (&left.value, &right.value) {
            // left of right is a symbol = unify as variables
            (Value::Symbol(var), _) => self.unify_var(var, right),
            (_, Value::Symbol(var)) => self.unify_var(var, left),

            // unifying two lists is done by unifying elements
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

            // integers unify by directly comparing values
            (Value::Integer(left), Value::Integer(right)) => {
                if left != right {
                    self.push_goal(Goal::Backtrack);
                }
            }

            // strings unify by directly comparing values
            (Value::String(left), Value::String(right)) => {
                if left != right {
                    self.push_goal(Goal::Backtrack);
                }
            }

            // bools unify by directly comparing values
            (Value::Boolean(left), Value::Boolean(right)) => {
                if left != right {
                    self.push_goal(Goal::Backtrack);
                }
            }
            (_, _) => unimplemented!("unhandled unification {:?} = {:?}", left, right),
        }
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
        let _ = vm.run();
        assert_eq!(vm.value(&x), Some(&zero));
        assert_eq!(vm.value(&y), None);
    }

    #[test]
    fn halt() {
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::new(), vec![Goal::Halt]);
        let _ = vm.run();
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
        let _ = vm.run();
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

        let _ = vm.run();

        // Left variable bound to bound right variable
        assert_eq!(vm.value(&Symbol("x".to_string())), Some(&one));

        // Left variable bound to value
        vm.append_goals(vec![
            Goal::Bind {
                variable: Symbol("z".to_string()),
                value: one.clone(),
            },
            Goal::Unify {
                left: Term::new(Value::Symbol(Symbol("z".to_string()))),
                right: Term::new(Value::Integer(1)),
            },
            // If unify failed, then backtrack instruction would throw away bind because
            // it pops stack until choice instruction is found.
            Goal::Bind {
                variable: Symbol("success".to_string()),
                value: one.clone(),
            },
        ]);

        let _ = vm.run();

        assert_eq!(vm.value(&Symbol("success".to_string())), Some(&one));

        // Left variable bound to value
        vm.append_goals(vec![
            Goal::Bind {
                variable: Symbol("z".to_string()),
                value: one.clone(),
            },
            Goal::Unify {
                left: Term::new(Value::Symbol(Symbol("z".to_string()))),
                right: Term::new(Value::Integer(2)),
            },
            // If unify failed, then backtrack instruction would throw away bind because
            // it pops stack until choice instruction is found.
            Goal::Bind {
                variable: Symbol("not_success".to_string()),
                value: one.clone(),
            },
        ]);

        let _ = vm.run();

        assert_ne!(vm.value(&Symbol("not_success".to_string())), Some(&one));
    }

    #[test]
    fn test_gen_var() {
        let term = Term::new(Value::List(vec![
            Term::new(Value::Integer(1)),
            Term::new(Value::Symbol(Symbol("x".to_string()))),
            Term::new(Value::List(vec![Term::new(Value::Symbol(Symbol(
                "y".to_string(),
            )))])),
        ]));

        let mut vm = PolarVirtualMachine::default();

        let renamed_term = vm.rename_vars(&term);
        let x_value = match renamed_term.clone().value {
            Value::List(terms) => match &terms[1].value {
                Value::Symbol(sym) => Some(sym.0.clone()),
                _ => None,
            },
            _ => None,
        };

        assert_eq!(x_value.unwrap(), "_x_0");

        let y_value = match renamed_term.value {
            Value::List(terms) => match &terms[2].value {
                Value::List(terms) => match &terms[0].value {
                    Value::Symbol(sym) => Some(sym.0.clone()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };

        assert_eq!(y_value.unwrap(), "_y_1");
    }
}
