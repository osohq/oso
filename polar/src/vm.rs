use std::collections::HashMap;

use crate::types::*;
use crate::ToPolarString;

pub const MAX_CHOICES: usize = 10_000;
pub const MAX_GOALS: usize = 10_000;

#[derive(Clone, Debug, PartialEq)]
#[must_use = "ignored goals are never accomplished"]
#[allow(clippy::large_enum_variant)]
#[allow(dead_code)]
pub enum Goal {
    Backtrack,
    Break,
    Cut,
    Halt,
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
        /// Consistent ID of the external instance the lookup is on
        instance_id: u64,
        /// ID to track which call the result is for
        call_id: u64,
        /// Field to be looked up (e.g. a symbol for a field name)
        field: Term,
        /// The value the result will be unified with
        value: Symbol,
    },
    MakeExternal {
        literal: InstanceLiteral,
        instance_id: u64,
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
            call_id_symbols: HashMap::new(),
        }
    }

    pub fn new_id(&mut self) -> u64 {
        self.kb.new_id()
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
                Goal::Break => return Ok(QueryEvent::Breakpoint),
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
                    instance_id,
                } => return Ok(self.make_external(literal, instance_id)),
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

    fn push_alternatives(&mut self, mut alternatives: Alternatives) {
        if alternatives.is_empty() {
            return;
        }
        let first = alternatives.pop().unwrap();
        self.push_choice(Choice {
            alternatives,
            bsp: self.bsp(),
            goals: self.goals.clone(),
        });
        self.append_goals(first);
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
                    let new = self.kb.gensym(&sym.0);
                    renames.insert(sym.clone(), new.clone());
                    Value::Symbol(new)
                }
            }
            _ => value.clone(),
        })
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
}

/// Implementations of instructions.
impl PolarVirtualMachine {
    /// Remove all bindings after the last choice point, and try the
    /// next available alternative. If no choice is possible, halt.
    fn backtrack(&mut self) {
        eprintln!("⇒ backtrack");
        match self.choices.pop() {
            None => self.push_goal(Goal::Halt),
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
        let (field_name, args) = match field.value.clone() {
            Value::Call(Predicate { name, args }) => (name, args),
            _ => panic!("call must be a predicate"),
        };

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
            args,
        }
    }

    pub fn make_external(&mut self, literal: InstanceLiteral, instance_id: u64) -> QueryEvent {
        QueryEvent::MakeExternal {
            instance_id,
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

                        self.push_alternatives(alternatives);
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
                            Value::ExternalInstance(ExternalInstance { instance_id }) => {
                                let call_id = self.new_id();
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
                            }
                            _ => panic!("can only perform lookups on dicts and instances"),
                        }
                    }
                    Operator::Make => {
                        assert_eq!(args.len(), 2);
                        let literal = args[0].clone();
                        let external = args[1].clone();

                        let literal = match literal.value {
                            Value::ExternalInstanceLiteral(instance_literal) => instance_literal,
                            _ => panic!("Wasn't rewritten or something?"),
                        };

                        let instance_id = match external.value {
                            Value::ExternalInstance(ExternalInstance { instance_id }) => {
                                instance_id
                            }
                            _ => panic!("Can only make external instances."),
                        };

                        // @TODO: Cache external instance ids so we don't call constructor twice.
                        self.push_goal(Goal::MakeExternal {
                            literal,
                            instance_id,
                        });
                    }
                    _ => todo!("can't query for expression: {:?}", operator),
                }
            }
            _ => todo!("can't query for: {}", term.value.to_polar()),
        }
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

#[cfg(test)]
mod docs {
    use super::*;

    /// Set up the polar VM with the knowledge base:
    /// ```no_run
    /// f(1); f(2);
    /// g(1); g(2);
    /// h(2);
    /// k(x) := f(x), g(x), h(x);
    /// ```
    /// for the query `k(x)`.
    #[allow(dead_code)]
    fn docs_setup() -> PolarVirtualMachine {
        let mut polar = crate::Polar::new();
        polar
            .load_str("f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);")
            .unwrap();
        let query = polar.new_query("k(x)").unwrap();
        query.vm
    }

    /// # The Polar VM
    ///
    /// <Insert nice description of the VM>
    #[test]
    fn docs1_vm() {}

    /// ## Choices stack
    ///
    /// Every time there are multiple ways to solve a target, the VM
    /// inserts a new choice point, by adding a `Choice` to the `choices` stack.
    ///
    /// Choices points are (currently) only added when resolving a `Query` goal,
    /// but this might be:
    ///  - Choices over which predicate to match
    ///  - Results from an external lookup (dot operator)
    ///  - Disjunctions (OR operator)
    ///
    /// Any time we want to add a new choice point to the virtual machine,
    /// we need to do 3 things:
    /// 1. "Save" the current goal stack in the choice point
    /// 2. Create a list of goals for each choice
    /// 3. Pushing the first set of goals onto the goal stack
    #[test]
    fn docs2_choices() {
        // VM starts with an empty KB and a single Halt goal
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::default(), vec![Goal::Halt]);
        assert_eq!(vm.goals[0], Goal::Halt);

        // Push some alternatives
        // First is a Noop, second is a break
        let alternatives = vec![vec![Goal::Break], vec![Goal::Noop]];
        vm.push_alternatives(alternatives);

        // Now the Vm has new goals for the first alternative
        assert_eq!(vm.goals, vec![Goal::Halt, Goal::Noop]);
        // and a single choice point with the "Break" goal
        assert_eq!(vm.choices.len(), 1);
        assert_eq!(vm.choices[0].goals, vec![Goal::Halt]);
        assert_eq!(vm.choices[0].alternatives[0], vec![Goal::Break]);
    }

    /// # Goals stack
    ///
    /// The VM maintains a stack of goals. The main purpose of this is
    /// to enable pausing, resuming, and forking execution.
    ///
    /// Pausing is accomplished by exiting from the main `run` loop.
    /// For example, the debugging `Break` goal will always return
    /// from the loop.
    ///
    /// Resuming is accomplished by entering the `run` loop again.
    #[test]
    fn docs3_goal_stack() {
        let goals = vec![Goal::Noop, Goal::Noop, Goal::Break, Goal::Halt];

        // VM starts with an empty KB and the above goals (in reverse order)
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::default(), goals);
        // Runs until the `Break` goal is hit
        let _ = vm.run();
        // At which point the goals contains the rest of the stack: the `Halt` goal
        assert_eq!(vm.goals, vec![Goal::Halt]);

        // Resuming will now run the VM to completion
        assert_eq!(vm.run().unwrap(), QueryEvent::Done);
    }

    /// # Backtracking
    ///
    /// Backtracking is the process of (a) going back to the last choice point
    /// and (b) resuming the stack from that point.
    ///
    /// Backtracking drains each choice point before moving onto the next one.
    #[test]
    pub fn docs4_backtracking() {
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::default(), Vec::new());

        // Set up some useful terms
        let q1 = Goal::Query {
            term: term!(value!(1)),
        };
        let q2 = Goal::Query {
            term: term!(value!(2)),
        };
        let qa = Goal::Query {
            term: term!(value!("a")),
        };
        let qb = Goal::Query {
            term: term!(value!("b")),
        };

        // Choice point 1: query for 1 or 2
        let alternatives = vec![vec![q2.clone()], vec![q1.clone()]];
        vm.push_alternatives(alternatives);

        // Choice point 2: query for "a" or "b"
        let alternatives = vec![vec![qb.clone()], vec![qa.clone()]];
        vm.push_alternatives(alternatives);

        // Two goals queued up in the VM
        assert_eq!(vm.goals, &[q1.clone(), qa.clone()]);
        // Plus two choice points
        assert_eq!(vm.choices.len(), 2);

        // Backtracking => we try the next option
        vm.backtrack();
        assert_eq!(vm.goals, &[q1.clone(), qb.clone()]);
        // only one choice point left
        assert_eq!(vm.choices.len(), 1);

        // Backtracking => run out of the qa/qb options, so on to q2
        vm.backtrack();
        assert_eq!(vm.goals, &[q2.clone()]);
        vm.backtrack();
        // No more choices left, so a halt goal gets pushed
        assert_eq!(vm.goals, &[q2.clone(), Goal::Halt]);
    }

    /// # Breakpoints
    ///
    /// Pushing a `Break` goal onto the stack will cause the VM
    /// to return from the main `run` loop at any arbitrary point
    #[test]
    fn docs4_breakpoints() {
        let goals = vec![
            Goal::Noop,
            Goal::Noop,
            Goal::Noop,
            Goal::Break, // break here; everything above the line will have executed already
            Goal::Noop,
            Goal::Noop,
        ];
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::default(), goals);

        assert_eq!(vm.run().unwrap(), QueryEvent::Breakpoint);
        assert_eq!(vm.goals.len(), 2);
    }

    /// # Cut
    ///
    /// A `cut` goal indicates that, upon hitting, all other alternatives for the current choice
    /// should be thrown away.
    /// In other words, the VM is committing to the current branch.
    #[test]
    fn docs4_cut() {
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::default(), vec![]);

        // this branch fails
        let fail_branch = vec![Goal::Unify {
            left: term!(value!(1)),
            right: term!(value!(2)),
        }];

        // this branch succeeds
        let ok_branch = vec![Goal::Unify {
            left: term!(value!(@sym "x")),
            right: term!(value!(1)),
        }];

        // First, we try the two branches normally
        //
        // Unify(1, 2)
        // Backtrack
        // ⇒ backtrack
        // Unify(x, 1)
        // ⇒ bind: x ← 1
        // ⇒ result
        vm.push_alternatives(vec![ok_branch.clone(), fail_branch.clone()]);
        assert!(matches!(vm.run().unwrap(), QueryEvent::Result { .. }));

        // If we cut on the first branch, we never get to the second:
        //
        // Cut
        // Unify(1, 2)
        // Backtrack
        // ⇒ backtrack
        // Halt
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::default(), vec![]);
        let mut cut_fail_branch = fail_branch.clone();
        cut_fail_branch.insert(0, Goal::Cut);
        vm.push_alternatives(vec![ok_branch.clone(), cut_fail_branch]);
        assert_eq!(vm.run().unwrap(), QueryEvent::Done);

        // But if we cut on the second branch, the cut happens on the successful branch,
        // so we still get the first return value (but would not get any more)
        //
        // Unify(1, 2)
        // Backtrack
        // ⇒ backtrack
        // Cut
        // Unify(x, 1)
        // ⇒ bind: x ← 1
        // ⇒ result
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::default(), vec![]);
        let mut cut_ok_branch = ok_branch.clone();
        cut_ok_branch.insert(0, Goal::Cut);
        vm.push_alternatives(vec![ok_branch.clone(), cut_ok_branch, fail_branch.clone()]);
        assert!(matches!(vm.run().unwrap(), QueryEvent::Result { .. }));
        assert_eq!(vm.run().unwrap(), QueryEvent::Done);
    }

    /// # is a ...
    ///
    /// TODO
    #[test]
    fn docs4_isa() {
        // todo
    }

    /// # LookupExternal
    ///
    /// A `LookupExternal` goal is created when an attribute/method lookup is
    /// required on an external instance.
    ///
    /// In this case, the VM's strategy is to return control back to the caller,
    /// and push another choice point with a single "LookupExternal" goal.
    #[test]
    #[ignore]
    fn docs4_lookup_external() {
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::default(), vec![]);

        vm.push_goal(Goal::LookupExternal {
            instance_id: 0,
            call_id: 0,
            // the "field" is a method call foo()
            field: term!(value!(pred!("foo", value!(Vec::new())))),
            value: sym!("x"),
        });
        // run the VM -> get back an external call event
        let res = vm.run().unwrap();
        assert!(matches!(res, QueryEvent::ExternalCall { .. }));

        // goal stack is empty, and we have a single choice point
        // for making another external call
        assert!(vm.goals.is_empty());
        assert_eq!(vm.choices.len(), 1);

        if let QueryEvent::ExternalCall { call_id, .. } = res {
            // return the result `instance.foo = 1`
            vm.external_call_result(call_id, Some(term!(value!(1))));
        }
    }
}
