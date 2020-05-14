use std::collections::HashMap;
use std::fmt;

use super::types::*;

#[derive(Clone, Debug)]
#[must_use = "ignored goals are never accomplished"]
pub enum Goal {
    Backtrack,
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
        instance: InstanceLiteral,
        field: Term,
    },
    LookupExternal {
        instance: ExternalInstance,
        field: Term,
    },
    Noop,
    Query {
        term: Term,
    },
    Unify {
        left: Term,
        right: Term,
    },
}

impl fmt::Display for Goal {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Goal::Unify { left, right } => {
                write!(fmt, "Unify({}, {})", left.to_polar(), right.to_polar())
            }
            Goal::Query { term } => write!(fmt, "Query({})", term.to_polar()),
            g => write!(fmt, "{:?}", g),
        }
    }
}

#[derive(Debug)]
struct Binding(Symbol, Term);

#[derive(Debug)]
pub struct Choice {
    alternatives: Alternatives,
    bsp: usize, // binding stack pointer
    /// The goal that led to these choices.  Goal to retry when exhausting alternatives.
    retry: Goal,
}

type Alternatives = Vec<Goals>;
type Bindings = Vec<Binding>;
type Choices = Vec<Choice>;
type Goals = Vec<Goal>;

#[derive(Default)]
pub struct PolarVirtualMachine {
    /// Stacks.
    goals: Goals,
    bindings: Bindings,
    choices: Choices,

    /// Rules and types.
    kb: KnowledgeBase,

    /// For temporary variable names.
    genvar_counter: usize,
}

// Methods which aren't goals/instructions.
impl PolarVirtualMachine {
    /// Make a new virtual machine with an initial list of goals.
    /// Reverse the goal list for the sanity of callers.
    pub fn new(kb: KnowledgeBase, mut goals: Goals) -> Self {
        goals.reverse();
        Self {
            goals,
            bindings: vec![],
            choices: vec![],
            kb,
            genvar_counter: 0,
        }
    }

    /// Run the virtual machine. While there are goals on the stack,
    /// pop them off and execute them one at at time until we have a
    /// `QueryEvent` to return. May be called multiple times to restart
    /// the machine.
    pub fn run(&mut self) -> PolarResult<QueryEvent> {
        if self.goals.is_empty() {
            if self.choices.is_empty() {
                return Ok(QueryEvent::Done);
            } else {
                self.backtrack();
            }
        }

        while let Some(goal) = self.goals.pop() {
            eprintln!("{}", goal);
            match goal {
                Goal::Backtrack => self.backtrack(),
                Goal::Cut => self.cut(),
                Goal::Halt => return Ok(self.halt()),
                Goal::Isa { .. } => unimplemented!("isa"),
                Goal::Lookup { .. } => unimplemented!("lookup"),
                Goal::LookupExternal { .. } => unimplemented!("lookup external"),
                Goal::Noop => (),
                Goal::Query { term } => self.query(term),
                Goal::TestExternal { name } => return Ok(self.test_external(name)), // POC
                Goal::Unify { left, right } => self.unify(&left, &right),
            }
        }

        Ok(QueryEvent::Result {
            bindings: self.bindings(),
        })
    }

    pub fn is_halted(&self) -> bool {
        self.goals.is_empty() && self.choices.is_empty()
    }

    /// Push a goal onto the goal stack.
    pub fn push_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
    }

    /// Push a choice onto the choice stack.
    fn push_choice(&mut self, choice: Choice) {
        self.choices.push(choice);
    }

    /// Push multiple goals onto the stack in reverse order.
    fn append_goals(&mut self, mut goals: Goals) {
        goals.reverse();
        self.goals.append(&mut goals);
    }

    /// Push a binding onto the binding stack.
    fn bind(&mut self, var: &Symbol, value: &Term) {
        eprintln!("=> bind {:?} ← {:?}", var, value);
        self.bindings.push(Binding(var.clone(), value.clone()));
    }

    /// Retrieve the current bindings and return them as a hash map.
    fn bindings(&mut self) -> HashMap<Symbol, Term> {
        let mut bindings = HashMap::new();
        for Binding(var, value) in &self.bindings {
            if self.is_temporary_var(&var) {
                continue;
            }

            bindings.insert(
                var.clone(),
                match &value.value {
                    Value::Symbol(sym) => self.value(&sym).unwrap_or(value).clone(),
                    _ => value.clone(),
                },
            );
        }
        bindings
    }

    /// Return the current binding stack pointer.
    fn bsp(&self) -> usize {
        self.bindings.len()
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

    /// Return `true` if `var` is a temporary.
    fn is_temporary_var(&self, name: &Symbol) -> bool {
        name.0.starts_with("_")
    }

    /// Generate a fresh set of variables for a term
    /// by renaming them to temporaries.
    fn rename_vars(&mut self, term: &Term) -> Term {
        let mut renames = HashMap::<Symbol, Symbol>::new();
        term.map(&mut |value| match value {
            Value::Symbol(sym) => {
                if let Some(new) = renames.get(sym) {
                    Value::Symbol(new.clone())
                } else {
                    let new = self.genvar(&sym.0);
                    renames.insert(sym.clone(), new.clone());
                    Value::Symbol(new)
                }
            }
            _ => value.clone(),
        })
    }
}

/// Implementations of instructions.
impl PolarVirtualMachine {
    /// Remove all bindings after the last choice point, and try the
    /// next available alternative. If no choice is possible, halt.
    fn backtrack(&mut self) {
        eprintln!("{}", "=> backtrack");
        let mut retries = vec![];
        loop {
            match self.choices.pop() {
                None => return self.push_goal(Goal::Halt),
                Some(Choice {
                    ref mut alternatives,
                    ref bsp,
                    ref retry,
                }) => {
                    // Unwind bindings.
                    self.bindings.drain(*bsp..);

                    // Find an alternate path.
                    if let Some(alternative) = alternatives.pop() {
                        // TODO order
                        self.append_goals(retries.iter().cloned().rev().collect());
                        self.append_goals(alternative);

                        // Leave a choice for any remaining alternatives.
                        return self.push_choice(Choice {
                            alternatives: alternatives.clone(),
                            bsp: *bsp,
                            retry: retry.clone(),
                        });
                    } else {
                        retries.push(retry.clone())
                    }
                }
            }
        }
    }

    /// A cut operation indicates that no other choices should
    /// be considered.
    ///
    /// This goal implements this by iterating through all
    /// goals and clearing all other choice branches.
    fn cut(&mut self) {
        unimplemented!("cut!");
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
        // TERM CANNOT BE VARIABLE IF SOMETHING PASSES THROUGH FFI
    }

    /// Halt the VM by clearing all goals and choices.
    pub fn halt(&mut self) -> QueryEvent {
        self.goals.clear();
        self.choices.clear();
        assert!(self.is_halted());
        QueryEvent::Done
    }

    // pub fn isa(&mut self) {}
    // pub fn lookup(&mut self) {}
    // pub fn lookup_external(&mut self) {}

    /// Query for the provided term.
    ///
    /// Uses the knowledge base to get an ordered list of rules.
    /// Creates a choice point over each rule, where each alternative
    /// consists of unifying the rule head with the arguments, then
    /// querying for each body clause.
    fn query(&mut self, term: Term) {
        match &term.value {
            Value::Call(predicate) =>
            // Select applicable rules for predicate.
            // Sort applicable rules by specificity.
            // Create a choice over the applicable rules.
            {
                match self.kb.rules.get(&predicate.name) {
                    None => self.push_goal(Goal::Backtrack),
                    Some(generic_rule) => {
                        let generic_rule = generic_rule.clone();
                        assert_eq!(generic_rule.name, predicate.name);

                        let mut alternatives = vec![];
                        for rule in generic_rule.rules.iter().rev() {
                            // Rename the parameters and body at the same time.
                            // FIXME: This is terrible right now.
                            // TODO(?): Should maybe parse these as terms.
                            let renames = self.rename_vars(&Term::new(Value::List(vec![
                                Term::new(Value::List(rule.params.clone())),
                                rule.body.clone(),
                            ])));
                            let renames = match renames.value {
                                Value::List(renames) => renames,
                                _ => panic!("expected a list of renamed parameters and body"),
                            };
                            let params = &renames[0];
                            let body = &renames[1];
                            let mut goals = vec![];

                            // Unify the arguments with the formal parameters.
                            goals.push(Goal::Unify {
                                left: Term::new(Value::List(predicate.args.clone())),
                                right: params.clone(),
                            });

                            // Query for the body clauses.
                            goals.push(Goal::Query {
                                term: Term::new(body.value.clone()),
                            });

                            alternatives.push(goals)
                        }

                        // Choose the first alternative, and push a choice for the rest.
                        self.append_goals(alternatives.pop().expect("a choice"));
                        self.push_choice(Choice {
                            alternatives,
                            retry: Goal::Query { term },
                            bsp: self.bsp(),
                        });
                    }
                }
            }
            Value::Expression(Operation {
                operator: Operator::And,
                args,
            }) => {
                for arg in args.iter() {
                    self.push_goal(Goal::Query { term: arg.clone() });
                }
            }
            Value::Expression(Operation {
                operator: Operator::Unify,
                args,
            }) => {
                assert_eq!(args.len(), 2);
                self.push_goal(Goal::Unify {
                    left: args[0].clone(),
                    right: args[1].clone(),
                });
            }
            _ => todo!("can't query for: {}", term.value.to_polar()),
        }
    }

    /// Handle an external result provided by the application.
    ///
    /// If the value is `Some(_)` then we have a result,
    /// and bind the symbol to the result value.
    ///
    /// If the value is `None` then the external has no (more)
    /// results, so we make sure to clear the trailing `TestExternal`
    /// goal that would otherwise follow.
    pub fn external_call_result(&mut self, call_id: u64, term: Option<Term>) {
        // TODO: Open question if we need to pass errors back down to rust.
        // For example what happens if the call asked for a field that doesn't exist?

        // // Externals are always followed by a halt.
        // assert!(matches!(self.goals.pop(), Some(Goal::Halt)));

        // if let Some(value) = value {
        //     // We have a value and should bind our variable to it.
        //     self.bind(name, &Term::new(Value::Integer(value)))
        // } else {
        //     // No more values, so no further queries to resolve.
        //     assert!(matches!(
        //         self.goals.pop(),
        //         Some(Goal::TestExternal { name }) if name == name
        //     ));
        // }
    }

    /// Called with the result of an external construct. The instance id
    /// gives a handle to the external instance.
    pub fn external_construct_result(&mut self, instance_id: u64) {
        unimplemented!()
    }

    /// Unify `left` and `right` terms.
    ///
    /// Outcomes of a unification are:
    ///  - Successful unification => bind zero or more variables to values
    ///  - Recursive unification => more `Unify` goals are pushed onto the stack
    ///  - Failure => backtrack
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
                    self.bind(left, &value);
                }
                (None, None) => {
                    // Neither is bound, so bind them together.
                    // TODO: should theoretically bind the earliest one here?
                    self.bind(left, right);
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
    use permute::permute;

    #[test]
    fn and_expression() {
        let one = Term::new(Value::Integer(1));
        let two = Term::new(Value::Integer(2));
        let three = Term::new(Value::Integer(3));
        let f1 = Rule {
            name: "f".to_string(),
            params: vec![one.clone()],
            body: Term::new(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![],
            })),
        };
        let f2 = Rule {
            name: "f".to_string(),
            params: vec![two.clone()],
            body: Term::new(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![],
            })),
        };
        let rule = GenericRule {
            name: "f".to_string(),
            rules: vec![f1, f2],
        };

        let mut kb = KnowledgeBase::new();
        kb.rules.insert(rule.name.clone(), rule);
        let goal = Goal::Query {
            term: Term::new(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![],
            })),
        };
        let mut vm = PolarVirtualMachine::new(kb, vec![goal]);
        assert!(matches!(vm.run().unwrap(), QueryEvent::Result{bindings} if bindings.is_empty()));
        assert!(vm.is_halted());

        let f1 = Term::new(Value::Call(Predicate {
            name: "f".to_string(),
            args: vec![one.clone()],
        }));
        let f2 = Term::new(Value::Call(Predicate {
            name: "f".to_string(),
            args: vec![two.clone()],
        }));
        let f3 = Term::new(Value::Call(Predicate {
            name: "f".to_string(),
            args: vec![three.clone()],
        }));

        // Querying for f(1)
        vm.push_goal(Goal::Query {
            term: Term::new(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![f1.clone()],
            })),
        });
        assert!(matches!(vm.run().unwrap(), QueryEvent::Result{bindings} if bindings.is_empty()));
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // Querying for f(1), f(2)
        vm.push_goal(Goal::Query {
            term: Term::new(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![f1.clone(), f2.clone()],
            })),
        });
        assert!(matches!(vm.run().unwrap(), QueryEvent::Result{bindings} if bindings.is_empty()));
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // Querying for f(3)
        vm.push_goal(Goal::Query {
            term: Term::new(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![f3.clone()],
            })),
        });
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // Querying for f(1), f(2), f(3)
        let parts = vec![f1, f2, f3];
        for permutation in permute(parts) {
            vm.push_goal(Goal::Query {
                term: Term::new(Value::Expression(Operation {
                    operator: Operator::And,
                    args: permutation,
                })),
            });
            assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
            assert!(vm.is_halted());
        }
    }

    #[test]
    fn unify_expression() {
        let mut kb = KnowledgeBase::new();
        let mut vm = PolarVirtualMachine::new(kb, vec![]);
        let one = Term::new(Value::Integer(1));
        let two = Term::new(Value::Integer(2));
        vm.push_goal(Goal::Query {
            term: Term::new(Value::Expression(Operation {
                operator: Operator::Unify,
                args: vec![one.clone(), one.clone()],
            })),
        });
        assert!(matches!(vm.run().unwrap(), QueryEvent::Result{bindings} if bindings.is_empty()));
        assert!(vm.is_halted());

        vm.push_goal(Goal::Query {
            term: Term::new(Value::Expression(Operation {
                operator: Operator::Unify,
                args: vec![one.clone(), two.clone()],
            })),
        });
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());
    }

    #[test]
    fn bind() {
        let x = Symbol("x".to_string());
        let y = Symbol("y".to_string());
        let zero = Term::new(Value::Integer(0));
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::new(), vec![]);
        vm.bind(&x, &zero);
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
        let x = Symbol("x".to_string());
        let y = Symbol("y".to_string());
        let z = Symbol("z".to_string());
        let one = Term::new(Value::Integer(1));
        let two = Term::new(Value::Integer(2));

        let mut vm = PolarVirtualMachine::new(KnowledgeBase::new(), vec![]);

        // Left variable bound to bound right variable.
        vm.bind(&y, &one);
        vm.append_goals(vec![Goal::Unify {
            left: Term::new(Value::Symbol(x.clone())),
            right: Term::new(Value::Symbol(y.clone())),
        }]);
        let _ = vm.run();
        assert_eq!(vm.value(&Symbol("x".to_string())), Some(&one));
        vm.backtrack();

        // Left variable bound to value.
        vm.bind(&z, &one);
        vm.append_goals(vec![Goal::Unify {
            left: Term::new(Value::Symbol(z.clone())),
            right: one.clone(),
        }]);
        let _ = vm.run();
        assert_eq!(vm.value(&z), Some(&one));

        // Left variable bound to value
        vm.bind(&z, &one);
        vm.append_goals(vec![Goal::Unify {
            left: Term::new(Value::Symbol(z.clone())),
            right: two.clone(),
        }]);
        let _ = vm.run();
        assert_eq!(vm.value(&z), Some(&one));
    }

    #[test]
    fn test_gen_var() {
        let mut vm = PolarVirtualMachine::default();
        let term = Term::new(Value::List(vec![
            Term::new(Value::Integer(1)),
            Term::new(Value::Symbol(Symbol("x".to_string()))),
            Term::new(Value::Symbol(Symbol("x".to_string()))),
            Term::new(Value::List(vec![Term::new(Value::Symbol(Symbol(
                "y".to_string(),
            )))])),
        ]));
        let renamed_term = vm.rename_vars(&term);

        let x_value = match renamed_term.clone().value {
            Value::List(terms) => {
                assert_eq!(terms[1].value, terms[2].value);
                match &terms[1].value {
                    Value::Symbol(sym) => Some(sym.0.clone()),
                    _ => None,
                }
            }
            _ => None,
        };
        assert_eq!(x_value.unwrap(), "_x_0");

        let y_value = match renamed_term.value {
            Value::List(terms) => match &terms[3].value {
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
