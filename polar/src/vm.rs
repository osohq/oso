use std::collections::HashMap;
use std::fmt;

use super::types::*;

#[derive(Debug)]
#[must_use = "ignored goals are never accomplished"]
pub enum Goal {
    Backtrack,
    Bind {
        variable: Symbol,
        value: Term,
    },
    Choice {
        choices: Vec<Goals>,
        bsp: usize, // binding stack pointer
    },
    Cut,
    TestExternal {
        name: Symbol, // POC
    },
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
        head: Predicate,
        tail: Vec<Predicate>,
    },
    Result {
        name: Symbol,
        value: Option<i64>,
    },
    Return,
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
                "Bind {{ {} ← {} }}",
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

#[derive(Debug)]
struct Binding(Symbol, Term);

type Goals = Vec<Goal>;
type Choices = Vec<Goals>;
type Bindings = Vec<Binding>;

#[derive(Default)]
pub struct PolarVirtualMachine {
    /// Stack of goals.
    goals: Goals,

    /// Stack of bindings.
    bindings: Bindings,

    /// Rules and types.
    kb: KnowledgeBase,

    /// For temporary variable names.
    genvar_counter: usize,
}

// Methods which aren't goals/instructions
impl PolarVirtualMachine {
    /// Push a new goal onto the stack.
    pub fn push_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
    }

    /// Push multiple goals onto the stack in reverse order.
    fn append_goals(&mut self, mut goals: Goals) {
        goals.reverse();
        self.goals.append(&mut goals);
    }

    /// Make a new virtual machine with an initial list of goals.
    /// Reverse the list for the sanity of callers.
    pub fn new(kb: KnowledgeBase, mut goals: Goals) -> Self {
        goals.reverse();
        Self {
            goals,
            bindings: vec![],
            kb,
            genvar_counter: 0,
        }
    }

    /// Run the virtual machine: while there are goals on the stack,
    /// pop them off and execute them one at at time until we have a
    /// `QueryEvent` to return. May be called multiple times to restart
    /// the machine.
    pub fn run(&mut self) -> QueryEvent {
        while let Some(goal) = self.goals.pop() {
            /*eprintln!(
                "{} stack [{}]",
                goal,
                self.goals
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            );*/
            match goal {
                Goal::Backtrack => self.backtrack(),
                Goal::Bind { variable, value } => self.bind(&variable, &value),
                Goal::Choice { choices, bsp } => self.choice(choices, bsp),
                Goal::Cut => self.cut(),
                Goal::Halt => self.halt(),
                Goal::Isa { .. } => unimplemented!("isa"),
                Goal::Lookup { .. } => unimplemented!("lookup"),
                Goal::LookupExternal { .. } => unimplemented!("lookup external"),
                Goal::Query { head, tail } => self.query(head, tail),
                Goal::Result { name, value } => self.result(&name, value),
                Goal::Return => {
                    return QueryEvent::Result {
                        bindings: self.return_bindings(),
                    };
                }
                Goal::TestExternal { name } => return self.test_external(name), // POC
                Goal::Unify { left, right } => self.unify(&left, &right),
            }
        }
        QueryEvent::Done
    }

    /// Look up a variable in the bindings stack and return
    /// a reference to its value.
    fn value(&self, variable: &Symbol) -> Option<&Term> {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.0 == *variable)
            .map(|binding| &binding.1)
    }

    /// Generate a new variable name.
    fn genvar(&mut self, prefix: &str) -> Symbol {
        let counter = self.genvar_counter;
        self.genvar_counter += 1;
        Symbol(format!("_{}_{}", prefix, counter))
    }

    /// Generate a fresh set of variables for a term
    /// by renaming them to temporaries.
    fn rename_vars(&mut self, term: &Term) -> Term {
        term.map(&mut |value| match value {
            Value::Symbol(sym) => Value::Symbol(self.genvar(&sym.0)),
            _ => value.clone(),
        })
    }
}

/// Implementations of instructions.
impl PolarVirtualMachine {
    /// Backtrack from the current goal, stopping once we reach
    /// a choice point with no more choices (which means this).
    ///
    /// Remove all bindings that were added after the last choice point.
    fn backtrack(&mut self) {
        while let Some(goal) = self.goals.pop() {
            match goal {
                Goal::Choice {
                    ref choices,
                    ref bsp,
                } => {
                    self.bindings.drain(bsp..);
                    if !choices.is_empty() {
                        self.push_goal(goal);
                    }
                    break;
                }
                _ => (),
            }
        }
    }

    /// Bind a variable to a value, i.e., directly "write" to memory.
    fn bind(&mut self, var: &Symbol, value: &Term) {
        self.bindings.push(Binding(var.clone(), value.clone()));
    }

    /// Retrieve the current bindings and return them as a hash map.
    fn return_bindings(&mut self) -> HashMap<Symbol, Term> {
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
    /// If there are no more choices, then this goal is a no-op.
    fn choice(&mut self, mut choices: Choices, bsp: usize) {
        if !choices.is_empty() {
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

    /// Test goal: wait for external input.
    ///
    /// Pushes a `Halt` goal onto the stack so the
    /// program terminates if we don't get a response.
    ///
    /// Also pushes another `TestExternal` goal for the same symbol
    /// so that we continue to poll for more results.
    fn test_external(&mut self, name: Symbol) -> QueryEvent {
        self.append_goals(vec![Goal::Halt, Goal::TestExternal { name: name.clone() }]);
        QueryEvent::TestExternal { name }
    }

    /// Halt the VM by clearing all goals.
    pub fn halt(&mut self) {
        self.goals.clear();
    }

    // pub fn isa(&mut self) {}
    // pub fn lookup(&mut self) {}
    // pub fn lookup_external(&mut self) {}

    /// Query for the provided predicate.
    ///
    /// Uses the knowledge base to get an ordered list of rules.
    /// Creates a choice point over each rule, where the choice point
    /// consists of (a) unifying the arguments (unifying the head),
    /// and (b) appending all clauses from the body onto the query
    /// and (c) concluding the query with a new `Return` goal if there
    /// are no further causes, or creating a new query otherwise.
    fn query(&mut self, head: Predicate, tail: Vec<Predicate>) {
        // Select applicable rules for predicate.
        // Sort applicable rules by specificity.
        // Create a choice over the applicable rules.

        if let Some(generic_rule) = self.kb.rules.get(&head.name) {
            let generic_rule = generic_rule.clone();
            assert_eq!(generic_rule.name, head.name);

            let mut choices = vec![];
            for rule in generic_rule.rules {
                // TODO(?): Should maybe parse these as terms.
                let parameter = Term::new(Value::List(rule.params.clone()));
                let argument = Term::new(Value::List(head.args.clone()));

                // (a) First goal is to unify the heads.
                let mut goals = vec![Goal::Unify {
                    left: parameter.clone(),
                    right: argument.clone(),
                }];

                let mut tail = tail.clone();

                // (b) Add clauses from body to query tail.
                for clause in rule.body.into_iter() {
                    if let Value::Call(predicate) = clause.value {
                        tail.push(predicate);
                    } else {
                        todo!("can clauses in a rule body be anything but predicates?")
                    }
                }

                if tail.is_empty() {
                    // (c) This is the last goal; return bindings.
                    goals.push(Goal::Return);
                } else {
                    // (c) Create a new query from the tail.
                    let head = tail.remove(0);
                    goals.push(Goal::Query { head, tail });
                }
                choices.push(goals)
            }

            self.push_goal(Goal::Choice {
                choices,
                bsp: self.bindings.len(),
            });
        } else {
            // No applicable rules, so backtrack.
            self.push_goal(Goal::Backtrack)
        }
    }

    /// Handle an external result provided by the application.
    ///
    /// If the value is `Some(_)` then we have a result and
    /// add a new `Bind` goal to bind the symbol to the value.
    ///
    /// If the value is `None` then the external has no (more)
    /// results, so we make sure to clear the trailing `TestExternal`
    /// goal that would otherwise follow.
    pub fn result(&mut self, name: &Symbol, value: Option<i64>) {
        // Externals are always followed by a halt.
        assert!(matches!(self.goals.pop(), Some(Goal::Halt)));

        if let Some(value) = value {
            // We have a value and should bind our variable to it.
            self.push_goal(Goal::Bind {
                variable: name.clone(),
                value: Term::new(Value::Integer(value)),
            });
        } else {
            // No more values, so no further queries to resolve.
            assert!(matches!(
                self.goals.pop(),
                Some(Goal::TestExternal { name }) if name == name
            ));
        }
    }

    /// Unify `left` and `right` terms.
    ///
    /// Outcomes of a unification are:
    ///  - Successful unification => a `Bind` goal is created for a symbol
    ///  - Recursive unification => more `Unify` goals are created
    ///  - Failure => this branch is false, and a `Backtrack` goal is created
    fn unify(&mut self, left: &Term, right: &Term) {
        // Unify a symbol `left` with a term `right`.
        // This is sort of a "sub-goal" of `Unify`.
        let mut unify_var = |left: &Symbol, right: &Term| {
            let left_value = self.value(&left).cloned();
            let mut right_value = None;
            if let Value::Symbol(ref right_sym) = right.value {
                right_value = self.value(right_sym).cloned();
            }

            match (left_value, right_value) {
                (Some(left), Some(right)) => {
                    // Both are bound, unify their values.
                    self.push_goal(Goal::Unify { left, right });
                }
                (Some(left), _) => {
                    // Only left is bound, unify with whatever right is.
                    self.push_goal(Goal::Unify {
                        left,
                        right: right.clone(),
                    });
                }
                (None, Some(value)) => {
                    // Left is unbound, right is bound;
                    // bind left to the value of right.
                    // TODO(?): could merge this branch and the (None, None)
                    // branch, but this avoids an additional goal.
                    self.push_goal(Goal::Bind {
                        variable: left.clone(),
                        value,
                    });
                }
                (None, None) => {
                    // Neither is bound, so bind them together.
                    // TODO: should theoretically bind the earliest one here?
                    self.push_goal(Goal::Bind {
                        variable: left.clone(),
                        value: right.clone(),
                    });
                }
            }
        };

        // Unify generic terms.
        match (&left.value, &right.value) {
            // Unify symbols as variables.
            (Value::Symbol(var), _) => unify_var(var, right),
            (_, Value::Symbol(var)) => unify_var(var, left),

            // Unify lists by recursively unifying the elements.
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

            // Unify integers by value.
            (Value::Integer(left), Value::Integer(right)) => {
                if left != right {
                    self.push_goal(Goal::Backtrack);
                }
            }

            // Unify strings by value.
            (Value::String(left), Value::String(right)) => {
                if left != right {
                    self.push_goal(Goal::Backtrack);
                }
            }

            // Unify bools by value.
            (Value::Boolean(left), Value::Boolean(right)) => {
                if left != right {
                    self.push_goal(Goal::Backtrack);
                }
            }

            // Anything else is an error.
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

        let mut vm = PolarVirtualMachine::new(KnowledgeBase::new(), vec![]);

        // Left variable bound to bound right variable.
        vm.append_goals(vec![
            Goal::Bind {
                variable: Symbol("y".to_string()),
                value: one.clone(),
            },
            Goal::Unify {
                left: Term::new(x),
                right: Term::new(y),
            },
        ]);
        let _ = vm.run();
        assert_eq!(vm.value(&Symbol("x".to_string())), Some(&one));

        // Left variable bound to value.
        vm.append_goals(vec![
            Goal::Bind {
                variable: Symbol("z".to_string()),
                value: one.clone(),
            },
            Goal::Unify {
                left: Term::new(Value::Symbol(Symbol("z".to_string()))),
                right: Term::new(Value::Integer(1)),
            },
            // If unify failed, then backtracking will throw away this Bind.
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
            // If the unify fails, then backtracking will throw away this Bind.
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
        let mut vm = PolarVirtualMachine::default();
        let term = Term::new(Value::List(vec![
            Term::new(Value::Integer(1)),
            Term::new(Value::Symbol(Symbol("x".to_string()))),
            Term::new(Value::List(vec![Term::new(Value::Symbol(Symbol(
                "y".to_string(),
            )))])),
        ]));
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
