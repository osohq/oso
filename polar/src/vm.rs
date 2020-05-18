use std::collections::HashMap;
use std::fmt;

use super::types::*;

pub const MAX_CHOICES: usize = 10_000;
pub const MAX_GOALS: usize = 10_000;

#[derive(Clone, Debug)]
#[must_use = "ignored goals are never accomplished"]
#[allow(clippy::large_enum_variant)]
pub enum Goal {
    Backtrack,
    Cut,
    Halt,
    #[allow(dead_code)]
    Isa {
        left: Term,
        right: Term,
    },
    Lookup {
        dict: Dictionary,
        field: Symbol,
        value: Term,
    },
    LookupExternal {
        instance_id: u64,
        call_id: u64,
        field: Term,
        value: Symbol,
    },
    MakeExternal {
        literal: InstanceLiteral,
        external_id: u64,
    },
    #[allow(dead_code)]
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
            Goal::Lookup { dict, field, value } => write!(
                fmt,
                "Lookup({}, {}, {})",
                dict.to_polar(),
                field.to_polar(),
                value.to_polar()
            ),
            Goal::LookupExternal {
                instance_id,
                field,
                value,
                ..
            } => write!(
                fmt,
                "LookupExternal({}, {}, {})",
                instance_id,
                field.to_polar(),
                value.to_polar(),
            ),
            Goal::Query { term } => write!(fmt, "Query({})", term.to_polar()),
            Goal::Unify { left, right } => {
                write!(fmt, "Unify({}, {})", left.to_polar(), right.to_polar())
            }
            g => write!(fmt, "{:?}", g),
        }
    }
}

#[derive(Debug)]
struct Binding(Symbol, Term);

#[derive(Debug)]
pub struct Choice {
    alternatives: Alternatives,
    bsp: usize,   // binding stack pointer
    goals: Goals, // goal stack snapshot
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

    /// For temporary variable names, call IDs, etc.
    counter: usize,

    /// Call ID -> result variable name table.
    call_id_symbols: HashMap<u64, Symbol>,
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
            counter: 0,
            call_id_symbols: HashMap::new(),
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
                Goal::Isa { .. } => todo!("isa"),
                Goal::Lookup { dict, field, value } => self.lookup(dict, field, value),
                Goal::LookupExternal {
                    call_id,
                    instance_id,
                    field,
                    value,
                } => return Ok(self.lookup_external(call_id, instance_id, field, value)),
                Goal::MakeExternal {
                    literal,
                    external_id,
                } => return Ok(self.make_external(literal, external_id)),
                Goal::Noop => (),
                Goal::Query { term } => self.query(term),
                Goal::Unify { left, right } => self.unify(&left, &right),
            }
        }

        eprintln!("⇒ result");
        Ok(QueryEvent::Result {
            bindings: self.bindings(),
        })
    }

    pub fn is_halted(&self) -> bool {
        self.goals.is_empty() && self.choices.is_empty()
    }

    /// Push a goal onto the goal stack.
    pub fn push_goal(&mut self, goal: Goal) {
        assert!(self.goals.len() < MAX_GOALS, "too many goals");
        self.goals.push(goal);
    }

    /// Push a choice onto the choice stack. Do nothing if there are no
    /// alternatives; this saves every caller a conditional, and maintains
    /// the invariant that only choice points with alternatives are on the
    /// choice stack.
    fn push_choice(&mut self, choice: Choice) {
        assert!(self.choices.len() < MAX_CHOICES, "too many choices");
        if !choice.alternatives.is_empty() {
            self.choices.push(choice);
        }
    }

    /// Push multiple goals onto the stack in reverse order.
    fn append_goals(&mut self, mut goals: Goals) {
        goals.reverse();
        self.goals.append(&mut goals);
    }

    /// Push a binding onto the binding stack.
    fn bind(&mut self, var: &Symbol, value: &Term) {
        eprintln!("⇒ bind: {} ← {}", var.to_polar(), value.to_polar());
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

    /// Return a monotonically increasing integer ID.
    fn id(&mut self) -> u64 {
        let id = self.counter;
        self.counter += 1;
        id as u64
    }

    /// Generate a new variable name.
    fn genvar(&mut self, prefix: &str) -> Symbol {
        Symbol(format!("_{}_{}", prefix, self.id()))
    }

    /// Return `true` if `var` is a temporary.
    fn is_temporary_var(&self, name: &Symbol) -> bool {
        name.0.starts_with('_')
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
        eprintln!("⇒ backtrack");
        match self.choices.pop() {
            None => return self.push_goal(Goal::Halt),
            Some(Choice {
                mut alternatives,
                bsp,
                goals,
            }) => {
                self.bindings.drain(bsp..);
                self.goals = goals.clone();
                self.append_goals(alternatives.pop().expect("expected an alternative"));
                self.push_choice(Choice {
                    alternatives,
                    bsp,
                    goals,
                });
            }
        }
    }

    /// Commit to the current choice.
    fn cut(&mut self) {
        self.choices.pop();
    }

    /// Halt the VM by clearing all goals and choices.
    pub fn halt(&mut self) -> QueryEvent {
        self.goals.clear();
        self.choices.clear();
        assert!(self.is_halted());
        QueryEvent::Done
    }

    // pub fn isa(&mut self) {}

    pub fn lookup(&mut self, dict: Dictionary, field: Symbol, value: Term) {
        if let Some(retrieved) = dict.fields.get(&field) {
            self.push_goal(Goal::Unify {
                left: retrieved.clone(),
                right: value,
            });
        } else {
            self.push_goal(Goal::Backtrack);
        }
    }

    /// Return an external call event to look up a field's value
    /// in an external instance. Push a `Goal::LookupExternal` as
    /// an alternative on the last choice point to poll for results.
    pub fn lookup_external(
        &mut self,
        call_id: u64,
        instance_id: u64,
        field: Term,
        value: Symbol,
    ) -> QueryEvent {
        let field_name = field_name(&field);
        self.push_choice(Choice {
            alternatives: vec![vec![Goal::LookupExternal {
                call_id,
                instance_id,
                field,
                value,
            }]],
            bsp: self.bsp(),
            goals: self.goals.clone(),
        });

        QueryEvent::ExternalCall {
            call_id,
            instance_id,
            attribute: field_name,
            args: vec![],
        }
    }

    pub fn make_external(&mut self, literal: InstanceLiteral, external_id: u64) -> QueryEvent {
        QueryEvent::ExternalConstructor {
            instance_id: external_id,
            instance: literal,
        }
    }

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
                        let goals = self.goals.clone();
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
                            bsp: self.bsp(),
                            goals,
                        });
                    }
                }
            }
            Value::Expression(Operation { operator, args }) => {
                match operator {
                    Operator::And => self.append_goals(
                        args.iter()
                            .map(|a| Goal::Query { term: a.clone() })
                            .collect(),
                    ),
                    Operator::Unify => {
                        assert_eq!(args.len(), 2);
                        self.push_goal(Goal::Unify {
                            left: args[0].clone(),
                            right: args[1].clone(),
                        });
                    }
                    Operator::Dot => {
                        assert_eq!(args.len(), 3);
                        let object = args[0].clone();
                        let field = args[1].clone();
                        let value = args[2].clone();

                        match object.value {
                            Value::Dictionary(dict) => self.push_goal(Goal::Lookup {
                                dict,
                                field: field_name(&field),
                                value,
                            }),
                            Value::InstanceLiteral(instance) => {
                                // Arrive here with an InstanceLiteral
                                // Look up the tag in kb.types and retrieve an Internal or External class
                                // For the external case, pass the instance to the External constructor

                                if true {
                                    // external case
                                    let instance_id = 1; // @TODO
                                    let call_id = self.id();
                                    let value = match value {
                                        Term {
                                            value: Value::Symbol(value),
                                            ..
                                        } => value,
                                        _ => panic!("bad lookup value: {}", value.to_polar()),
                                    };
                                    self.call_id_symbols.insert(call_id, value.clone());
                                    self.push_goal(Goal::LookupExternal {
                                        call_id,
                                        instance_id,
                                        field,
                                        value,
                                    });
                                } else {
                                    // internal
                                    self.push_goal(Goal::Lookup {
                                        dict: instance.fields,
                                        field: field_name(&field),
                                        value,
                                    });
                                }
                            }
                            _ => panic!("can only perform lookups on dicts and instances"),
                        }
                    }
                    Operator::Make => {
                        assert_eq!(args.len(), 3);
                        let literal = args[0].clone();
                        let external = args[1].clone();

                        let literal = match literal.value {
                            Value::ExternalInstanceLiteral(instance_literal) => instance_literal,
                            _ => panic!("Wasn't rewritten or something?"),
                        };

                        let external_id = match external.value {
                            Value::ExternalInstance(ExternalInstance { external_id }) => {
                                external_id
                            }
                            _ => panic!("Can only make external instances."),
                        };

                        // @todo, see if we have one.
                        //      just bind it
                        // else
                        self.push_goal(Goal::MakeExternal {
                            literal,
                            external_id,
                        });
                    }
                    _ => todo!("can't query for expression: {:?}", operator),
                }
            }
            _ => todo!("can't query for: {}", term.value.to_polar()),
        }
    }

    /// Handle an external result provided by the application.
    ///
    /// If the value is `Some(_)` then we have a result, and bind the
    /// symbol associated with the call ID to the result value. If the
    /// value is `None` then the external has no (more) results, so we
    /// backtrack to the choice point left by `Goal::LookupExternal`.
    pub fn external_call_result(&mut self, call_id: u64, term: Option<Term>) {
        // TODO: Open question if we need to pass errors back down to rust.
        // For example what happens if the call asked for a field that doesn't exist?

        if let Some(value) = term {
            self.bind(
                &self
                    .call_id_symbols
                    .get(&call_id)
                    .expect("unregistered external call ID")
                    .clone(),
                &value,
            );
        } else {
            // No more results. Clean up, cut out the retry alternative,
            // and backtrack.
            self.call_id_symbols.remove(&call_id).expect("bad call ID");
            self.push_goal(Goal::Backtrack);
            self.push_goal(Goal::Cut);
        }
    }

    /// Called with the result of an external construct. The instance id
    /// gives a handle to the external instance.
    #[allow(dead_code)]
    pub fn external_construct_result(&mut self, _instance_id: u64) {
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
    #[allow(clippy::cognitive_complexity)]
    fn and_expression() {
        let one = Term::new(Value::Integer(1));
        let two = Term::new(Value::Integer(2));
        let three = Term::new(Value::Integer(3));
        let f1 = Rule {
            name: Symbol::new("f"),
            params: vec![one.clone()],
            body: Term::new(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![],
            })),
        };
        let f2 = Rule {
            name: Symbol::new("f"),
            params: vec![two.clone()],
            body: Term::new(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![],
            })),
        };
        let rule = GenericRule {
            name: Symbol::new("f"),
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
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        let f1 = Term::new(Value::Call(Predicate {
            name: Symbol::new("f"),
            args: vec![one],
        }));
        let f2 = Term::new(Value::Call(Predicate {
            name: Symbol::new("f"),
            args: vec![two],
        }));
        let f3 = Term::new(Value::Call(Predicate {
            name: Symbol::new("f"),
            args: vec![three],
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
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::new(), vec![]);
        let one = Term::new(Value::Integer(1));
        let two = Term::new(Value::Integer(2));
        vm.push_goal(Goal::Query {
            term: Term::new(Value::Expression(Operation {
                operator: Operator::Unify,
                args: vec![one.clone(), one.clone()],
            })),
        });
        assert!(matches!(vm.run().unwrap(), QueryEvent::Result{bindings} if bindings.is_empty()));
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        vm.push_goal(Goal::Query {
            term: Term::new(Value::Expression(Operation {
                operator: Operator::Unify,
                args: vec![one, two],
            })),
        });
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());
    }

    #[test]
    fn lookup() {
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::new(), vec![]);
        let x = Symbol("x".to_string());

        // Lookup with correct value
        let one = Value::Integer(1);
        let mut dict = Dictionary::new();
        dict.fields.insert(x.clone(), Term::new(one.clone()));
        vm.push_goal(Goal::Lookup {
            dict: dict.clone(),
            field: x.clone(),
            value: Term::new(one.clone()),
        });
        assert!(matches!(vm.run().unwrap(), QueryEvent::Result{bindings} if bindings.is_empty()));
        assert!(vm.is_halted());

        // Lookup with incorrect value
        let two = Value::Integer(2);
        vm.push_goal(Goal::Lookup {
            dict: dict.clone(),
            field: x.clone(),
            value: Term::new(two),
        });
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // Lookup with unbound value
        let y = Symbol("y".to_string());
        vm.push_goal(Goal::Lookup {
            dict,
            field: x,
            value: Term::new(Value::Symbol(y.clone())),
        });
        assert!(
            matches!(vm.run().unwrap(), QueryEvent::Result{bindings} if bindings.get(&y).unwrap().value == one)
        );
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
                left: vars,
                right: vals,
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
            left: Term::new(Value::Symbol(x)),
            right: Term::new(Value::Symbol(y)),
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
            right: two,
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
