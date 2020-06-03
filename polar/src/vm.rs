use std::collections::{HashMap, HashSet};
use std::string::ToString;
use std::sync::Arc;

use super::types::*;
use super::ToPolarString;

pub const MAX_CHOICES: usize = 10_000;
pub const MAX_GOALS: usize = 10_000;
pub const MAX_EXECUTED_GOALS: usize = 10_000;

#[derive(Clone, Debug)]
#[must_use = "ignored goals are never accomplished"]
#[allow(clippy::large_enum_variant)]
pub enum Goal {
    Backtrack,
    /// An explicit breakpoint, causes the VM to return a `QueryEvent::Breakpoint`
    Break,
    Cut,
    Debug {
        message: String,
    },
    Halt,
    Isa {
        left: Term,
        right: Term,
    },
    IsMoreSpecific {
        left: Rule,
        right: Rule,
        args: TermList,
    },
    IsSubspecializer {
        answer: Symbol,
        left: Term,
        right: Term,
        arg: Term,
    },
    Lookup {
        dict: Dictionary,
        field: Term,
        value: Term,
    },
    LookupExternal {
        instance_id: u64,
        call_id: u64,
        field: Term,
    },
    MakeExternal {
        literal: InstanceLiteral,
        instance_id: u64,
    },
    IsaExternal {
        instance_id: u64,
        literal: InstanceLiteral,
    },
    Noop,
    Query {
        term: Term,
    },
    PopQuery {
        term: Term,
    },
    SortRules {
        rules: Rules,
        args: TermList,
        outer: usize,
        inner: usize,
    },
    Unify {
        left: Term,
        right: Term,
    },
}

#[derive(Clone, Debug)]
pub struct Binding(pub Symbol, pub Term);

#[derive(Clone, Debug)]
pub struct Choice {
    pub alternatives: Alternatives,
    bsp: usize,        // binding stack pointer
    pub goals: Goals,  // goal stack snapshot
    queries: TermList, // query stack snapshot
}

pub type Alternatives = Vec<Goals>;
pub type Bindings = Vec<Binding>;
pub type Choices = Vec<Choice>;
pub type Goals = Vec<Goal>;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Breakpoint {
    None,
    Over { queries: TermList },
    Out { queries: TermList },
    Step { goal: Goal },
}

impl Default for Breakpoint {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default)]
pub struct PolarVirtualMachine {
    /// Stacks.
    pub goals: Goals,
    pub bindings: Bindings,
    choices: Choices,
    pub queries: TermList,

    /// Count executed goals
    goal_counter: usize,

    /// If VM is set to `debug=True`, the VM will return a `QueryEvent::Breakpoint`
    /// after every goal
    debug: bool,

    /// If true, stop after the next goal.
    pub breakpoint: Breakpoint,

    /// Rules and types.
    kb: Arc<KnowledgeBase>,

    /// Instance Literal -> External Instance table.
    instances: HashMap<InstanceLiteral, ExternalInstance>,
    /// Call ID -> result variable name table.
    call_id_symbols: HashMap<u64, Symbol>,
}

/// Debugging information exposed by the VM
#[derive(Clone, Debug, Default)]
pub struct DebugInfo {
    // we dont use the type bindings so the types can stay private
    pub goals: Vec<Goal>,
    pub bindings: Vec<Binding>,
    pub choices: Vec<Choice>,
}

// Methods which aren't goals/instructions.
impl PolarVirtualMachine {
    /// Make a new virtual machine with an initial list of goals.
    /// Reverse the goal list for the sanity of callers.
    pub fn new(kb: Arc<KnowledgeBase>, mut goals: Goals) -> Self {
        goals.reverse();
        Self {
            goals,
            bindings: vec![],
            choices: vec![],
            goal_counter: 0,
            debug: false,
            queries: vec![],
            breakpoint: Breakpoint::default(),
            kb,
            instances: HashMap::new(),
            call_id_symbols: HashMap::new(),
        }
    }

    pub fn new_id(&mut self) -> u64 {
        self.kb.new_id()
    }

    fn new_call_id(&mut self, symbol: &Symbol) -> u64 {
        let call_id = self.new_id();
        self.call_id_symbols.insert(call_id, symbol.clone());
        call_id
    }

    pub fn start_debug(&mut self) {
        self.debug = true;
    }

    pub fn stop_debug(&mut self) {
        self.debug = false;
    }

    pub fn debug_info(&self) -> DebugInfo {
        DebugInfo {
            bindings: self.bindings.clone(),
            choices: self.choices.clone(),
            goals: self.goals.clone(),
        }
    }

    /// Try to achieve one goal. Return `Some(QueryEvent)` if an external
    /// result is needed to achieve it, or `None` if it can run internally.
    fn next(&mut self, goal: Goal) -> PolarResult<QueryEvent> {
        if std::env::var("RUST_LOG").is_ok() {
            eprintln!("{}", goal);
        }
        self.goal_counter += 1;
        match goal {
            Goal::Backtrack => self.backtrack()?,
            Goal::Break => return Ok(QueryEvent::Breakpoint),
            Goal::Cut => self.cut(),
            Goal::Debug { message } => return Ok(self.debug(&message)),
            Goal::Halt => return Ok(self.halt()),
            Goal::Isa { left, right } => self.isa(&left, &right)?,
            Goal::IsMoreSpecific { left, right, args } => {
                self.is_more_specific(left, right, args)?
            }
            Goal::IsSubspecializer {
                answer,
                left,
                right,
                arg,
            } => return self.is_subspecializer(answer, left, right, arg),
            Goal::Lookup { dict, field, value } => self.lookup(dict, field, value)?,
            Goal::LookupExternal {
                call_id,
                instance_id,
                field,
            } => return self.lookup_external(call_id, instance_id, field),
            Goal::IsaExternal {
                instance_id,
                literal,
            } => return self.isa_external(instance_id, literal),
            Goal::MakeExternal {
                literal,
                instance_id,
            } => return Ok(self.make_external(literal, instance_id)),
            Goal::Noop => {}
            Goal::Query { term } => return self.query(term),
            Goal::PopQuery { .. } => self.pop_query(),
            Goal::SortRules {
                rules,
                outer,
                inner,
                args,
            } => self.sort_rules(rules, args, outer, inner)?,
            Goal::Unify { left, right } => self.unify(&left, &right)?,
        }
        // don't break when the goal stack is empty or a result wont
        // be returned (this logic seems flaky)
        if self.debug && !self.goals.is_empty() {
            return Ok(QueryEvent::Breakpoint);
        }
        Ok(QueryEvent::None)
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
                self.backtrack()?;
            }
        }

        while let Some(goal) = self.goals.pop() {
            match self.next(goal.clone())? {
                QueryEvent::None => (),
                event => return Ok(event),
            }
            self.maybe_break(Breakpoint::Step { goal });
        }

        if std::env::var("RUST_LOG").is_ok() {
            eprintln!("⇒ result");
        }
        Ok(QueryEvent::Result {
            bindings: self.bindings(),
        })
    }

    /// Return true if there is nothing left to do.
    pub fn is_halted(&self) -> bool {
        self.goals.is_empty() && self.choices.is_empty()
    }

    /// Push a goal onto the goal stack.
    pub fn push_goal(&mut self, goal: Goal) -> PolarResult<()> {
        if self.goals.len() >= MAX_GOALS {
            return Err(RuntimeError::StackOverflow {
                msg: format!("Goal stack overflow! MAX_GOALS = {}", MAX_GOALS),
            }
            .into());
        }
        if self.goal_counter >= MAX_EXECUTED_GOALS {
            return Err(RuntimeError::StackOverflow {
                msg: format!(
                    "Goal count exceeded! MAX_EXECUTED_GOALS = {}",
                    MAX_EXECUTED_GOALS
                ),
            }
            .into());
        }
        self.goals.push(goal);
        Ok(())
    }

    /// Push a non-trivial choice onto the choice stack.
    ///
    /// Params:
    ///
    /// - `alternatives`: an ordered list of alternatives to try in the choice.
    ///   The first element is the first alternative to try.
    ///
    /// Do not modify the goals stack.  This function defers execution of the
    /// choice until a backtrack occurs.  To immediately execute the choice on
    /// top of the current stack, use `choose`.
    ///
    /// Do nothing if there are no alternatives; this saves every caller a
    /// conditional, and maintains the invariant that only choice points with
    /// alternatives are on the choice stack.
    fn push_choice(&mut self, mut alternatives: Alternatives) {
        if !alternatives.is_empty() {
            // Make sure that alternatives are executed in order of first to last.
            alternatives.reverse();
            assert!(self.choices.len() < MAX_CHOICES, "too many choices");
            self.choices.push(Choice {
                alternatives,
                bsp: self.bsp(),
                goals: self.goals.clone(),
                queries: self.queries.clone(),
            });
        }
    }

    /// Push a choice onto the choice stack, and execute immediately by
    /// pushing the first alternative onto the goals stack
    ///
    /// Params:
    ///
    /// - `alternatives`: an ordered list of alternatives to try in the choice.
    ///   The first element is the first alternative to try.
    fn choose(&mut self, mut alternatives: Alternatives) {
        if !alternatives.is_empty() {
            let alternative = alternatives.remove(0);
            self.push_choice(alternatives);
            self.append_goals(alternative);
        }
    }

    /// Push multiple goals onto the stack in reverse order.
    fn append_goals(&mut self, mut goals: Goals) {
        goals.reverse();
        self.goals.append(&mut goals);
    }

    /// Push a binding onto the binding stack.
    fn bind(&mut self, var: &Symbol, value: &Term) {
        if std::env::var("RUST_LOG").is_ok() {
            eprintln!("⇒ bind: {} ← {}", var.to_polar(), value.to_polar());
        }
        self.bindings.push(Binding(var.clone(), value.clone()));
    }

    /// Retrieve the current bindings and return them as a hash map.
    fn bindings(&mut self) -> HashMap<Symbol, Term> {
        let mut bindings = HashMap::new();
        for Binding(var, value) in &self.bindings {
            if self.is_temporary_var(&var) {
                continue;
            }

            bindings.insert(var.clone(), self.deref(value));
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

    fn find_or_make_instance(
        &mut self,
        instance_literal: &InstanceLiteral,
    ) -> (bool, ExternalInstance) {
        if let Some(external_instance) = self.instances.get(instance_literal) {
            (true, external_instance.clone())
        } else {
            let new_external_id = self.new_id();
            let new_external_instance = ExternalInstance {
                instance_id: new_external_id,
                literal: Some(instance_literal.clone()),
            };
            self.instances
                .insert(instance_literal.clone(), new_external_instance.clone());
            (false, new_external_instance)
        }
    }

    /// Recursively dereference a variable.
    pub fn deref(&self, term: &Term) -> Term {
        match &term.value {
            Value::Symbol(symbol) => self.value(&symbol).map_or(term.clone(), |t| self.deref(t)),
            _ => term.clone(),
        }
    }

    /// Takes a term and makes sure it is instantiated by recursively:
    /// - Derefing all symbols
    /// - Converting literals into externals, and pushing goals if needed
    ///
    /// ## Returns
    ///
    /// The newly instantiated term (i.e. with external instances instead of literals)
    /// and some goals which actually do the instance creation (if any are needed).
    /// The goals must be met before the term is used.
    fn instantiate_externals(&mut self, term: &Term) -> (Term, Goals) {
        let mut goals = Vec::new();
        // this is the recursive mapping function
        fn instantiate_map(vm: &mut PolarVirtualMachine, term: &Term, goals: &mut Goals) -> Term {
            term.map(&mut |v| match v {
                Value::InstanceLiteral(instance_literal) => {
                    let (exists, external_instance) = vm.find_or_make_instance(instance_literal);
                    if !exists {
                        goals.push(Goal::MakeExternal {
                            literal: instance_literal.clone(),
                            instance_id: external_instance.instance_id,
                        });
                    }
                    Value::ExternalInstance(external_instance)
                }
                Value::Symbol(s) => {
                    // need to clone since `value` otherwise borrows `vm`.
                    if let Some(t) = vm.value(s).cloned() {
                        instantiate_map(vm, &t, goals).value
                    } else {
                        v.clone()
                    }
                }
                _ => v.clone(),
            })
        }

        let new_term = instantiate_map(self, term, &mut goals);
        (new_term, goals)
    }

    /// Return `true` if `var` is a temporary.
    fn is_temporary_var(&self, name: &Symbol) -> bool {
        name.0.starts_with('_')
    }

    /// Generate a fresh set of variables for a rule
    /// by renaming them to temporaries.
    fn rename_vars(&mut self, rule: &Rule) -> Rule {
        let mut renames = HashMap::<Symbol, Symbol>::new();
        rule.map(&mut move |value| match value {
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
}

/// Implementations of instructions.
impl PolarVirtualMachine {
    /// Remove all bindings after the last choice point, and try the
    /// next available alternative. If no choice is possible, halt.
    fn backtrack(&mut self) -> PolarResult<()> {
        if std::env::var("RUST_LOG").is_ok() {
            eprintln!("⇒ backtrack");
        }
        match self.choices.pop() {
            None => self.push_goal(Goal::Halt),
            Some(Choice {
                mut alternatives,
                bsp,
                goals,
                queries,
            }) => {
                self.bindings.drain(bsp..);
                self.queries = queries.clone();
                self.goals = goals.clone();
                self.append_goals(alternatives.pop().expect("must have alternative"));

                if !alternatives.is_empty() {
                    self.choices.push(Choice {
                        alternatives,
                        bsp,
                        goals,
                        queries,
                    });
                }
                Ok(())
            }
        }
    }

    /// Commit to the current choice.
    fn cut(&mut self) {
        self.choices.pop();
    }

    /// Clean up the query stack after completing a query.
    fn pop_query(&mut self) {
        self.queries.pop();
        self.maybe_break(Breakpoint::Over { queries: vec![] });
    }

    /// Interact with the debugger.
    fn debug(&mut self, message: &str) -> QueryEvent {
        QueryEvent::Debug {
            message: message.to_string(),
        }
    }

    /// Halt the VM by clearing all goals and choices.
    pub fn halt(&mut self) -> QueryEvent {
        self.goals.clear();
        self.choices.clear();
        assert!(self.is_halted());
        QueryEvent::Done
    }

    /// Comparison operator that essentially performs partial unification.
    pub fn isa(&mut self, left: &Term, right: &Term) -> PolarResult<()> {
        match (&left.value, &right.value) {
            (Value::List(left), Value::List(right)) => {
                if left.len() == right.len() {
                    self.append_goals(
                        left.iter()
                            .zip(right)
                            .map(|(left, right)| Goal::Isa {
                                left: left.clone(),
                                right: right.clone(),
                            })
                            .collect(),
                    )
                } else {
                    self.push_goal(Goal::Backtrack)?;
                }
            }

            (Value::Dictionary(left), Value::Dictionary(right)) => {
                // Check that the left is more specific than the right.
                let left_fields: HashSet<&Symbol> = left.fields.keys().collect();
                let right_fields: HashSet<&Symbol> = right.fields.keys().collect();
                if !right_fields.is_subset(&left_fields) {
                    return self.push_goal(Goal::Backtrack);
                }

                // For each field on the right, isa its value against the corresponding value on
                // the left.
                for (k, v) in right.fields.iter() {
                    let left = left
                        .fields
                        .get(&k)
                        .expect("left fields should be a superset of right fields")
                        .clone();
                    self.push_goal(Goal::Isa {
                        left,
                        right: v.clone(),
                    })?
                }
            }

            (Value::InstanceLiteral(_), _) => {
                // COMMENT (leina): do we ALWAYS want to convert an instance literal to an external instance here?
                // Any compelling use case for unifying an instance literal with another instance literal?
                // I can't think of any...

                // Convert instance literal to an external instance
                let (left, mut goals) = self.instantiate_externals(&left);
                goals.push(Goal::Isa {
                    left,
                    right: right.clone(),
                });
                self.append_goals(goals);
            }

            (Value::ExternalInstance(left), Value::Dictionary(right)) => {
                // For each field in the dict, look up the corresponding field on the instance and
                // then isa them.
                for (field, right_value) in right.fields.iter() {
                    let left_value = self.kb.gensym("isa_value");
                    let call_id = self.new_call_id(&left_value);
                    let lookup = Goal::LookupExternal {
                        instance_id: left.instance_id,
                        call_id,
                        field: Term::new(Value::Call(Predicate {
                            name: field.clone(),
                            args: vec![],
                        })),
                    };
                    let isa = Goal::Isa {
                        left: Term::new(Value::Symbol(left_value)),
                        right: right_value.clone(),
                    };
                    self.append_goals(vec![lookup, isa]);
                }
            }

            (Value::Symbol(symbol), _) => {
                if let Some(value) = self.value(&symbol).cloned() {
                    self.push_goal(Goal::Isa {
                        left: value,
                        right: right.clone(),
                    })?;
                } else {
                    self.push_goal(Goal::Unify {
                        left: left.clone(),
                        right: right.clone(),
                    })?;
                }
            }

            (_, Value::Symbol(symbol)) => {
                if let Some(value) = self.value(&symbol).cloned() {
                    self.push_goal(Goal::Isa {
                        left: left.clone(),
                        right: value,
                    })?;
                } else {
                    self.push_goal(Goal::Unify {
                        left: left.clone(),
                        right: right.clone(),
                    })?;
                }
            }

            (Value::ExternalInstance(left), Value::InstanceLiteral(right)) => {
                // Check fields
                self.push_goal(Goal::Isa {
                    left: Term::new(Value::ExternalInstance(left.clone())),
                    right: Term::new(Value::Dictionary(right.clone().fields)),
                })?;
                // Check class
                self.push_goal(Goal::IsaExternal {
                    instance_id: left.instance_id,
                    literal: right.clone(),
                })?;
            }

            _ => self.push_goal(Goal::Unify {
                left: left.clone(),
                right: right.clone(),
            })?,
        }
        Ok(())
    }

    pub fn lookup(&mut self, dict: Dictionary, field: Term, value: Term) -> PolarResult<()> {
        // check if field is a variable
        match &field.value {
            Value::Symbol(_) => {
                let mut alternatives: Vec<Vec<Goal>> = vec![];
                for (k, v) in &dict.fields {
                    let mut goals: Vec<Goal> = vec![];
                    // attempt to unify dict key with field
                    // if `field` is bound, unification will only succeed for the matching key
                    // if `field` is unbound, unification will succeed for all keys
                    goals.push(Goal::Unify {
                        left: Term::new(Value::String(k.clone().0)),
                        right: field.clone(),
                    });
                    // attempt to unify dict value with result
                    goals.push(Goal::Unify {
                        left: v.clone(),
                        right: value.clone(),
                    });
                    alternatives.push(goals);
                }
                self.choose(alternatives);
            }
            _ => {
                if let Some(retrieved) = dict.fields.get(&field_name(&field)) {
                    self.push_goal(Goal::Unify {
                        left: retrieved.clone(),
                        right: value,
                    })?;
                } else {
                    self.push_goal(Goal::Backtrack)?;
                }
            }
        };
        Ok(())
    }

    /// Return an external call event to look up a field's value
    /// in an external instance. Push a `Goal::LookupExternal` as
    /// an alternative on the last choice point to poll for results.
    ///
    /// ## Invariants
    ///
    /// The `field` term _must_ have been instantiated with
    /// `instantiate_externals` before this method is called.
    pub fn lookup_external(
        &mut self,
        call_id: u64,
        instance_id: u64,
        field: Term,
    ) -> PolarResult<QueryEvent> {
        let (field_name, args) = match &field.value {
            Value::Call(Predicate { name, args }) => (
                name.clone(),
                args.iter().map(|arg| self.deref(arg)).collect(),
            ),
            _ => unreachable!("call must be a predicate"),
        };
        self.push_choice(vec![vec![Goal::LookupExternal {
            call_id,
            instance_id,
            field,
        }]]);

        Ok(QueryEvent::ExternalCall {
            call_id,
            instance_id,
            attribute: field_name,
            args,
        })
    }

    pub fn isa_external(
        &mut self,
        instance_id: u64,
        literal: InstanceLiteral,
    ) -> PolarResult<QueryEvent> {
        let result = self.kb.gensym("isa");
        let call_id = self.new_call_id(&result);

        self.push_goal(Goal::Unify {
            left: Term::new(Value::Symbol(result)),
            right: Term::new(Value::Boolean(true)),
        })?;

        Ok(QueryEvent::ExternalIsa {
            call_id,
            instance_id,
            class_tag: literal.tag,
        })
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
    fn query(&mut self, term: Term) -> PolarResult<QueryEvent> {
        match term.value.clone() {
            Value::Call(predicate) =>
            // Select applicable rules for predicate.
            // Sort applicable rules by specificity.
            // Create a choice over the applicable rules.
            {
                match &predicate.name.0[..] {
                    // Built-in predicates.
                    "cut" => self.push_goal(Goal::Cut)?,
                    "debug" => self.push_goal(Goal::Debug {
                        // Should pull message from predicate.args.
                        message: "Welcome to the debugger!".to_string(),
                    })?,

                    // User-defined predicates.
                    //
                    // WOWHACK: probs shouldn't clone the entire KB
                    _ => match self.kb.clone().rules.get(&predicate.name) {
                        None => self.push_goal(Goal::Backtrack)?,
                        Some(generic_rule) => {
                            self.queries.push(term.clone());
                            self.push_goal(Goal::PopQuery { term })?;

                            let generic_rule = generic_rule.clone();
                            assert_eq!(generic_rule.name, predicate.name);
                            self.push_goal(Goal::SortRules {
                                rules: generic_rule
                                    .rules
                                    .into_iter()
                                    .filter(|r| r.params.len() == predicate.args.len())
                                    .collect(),
                                args: predicate.args.clone(),
                                outer: 1,
                                inner: 1,
                            })?;
                        }
                    },
                }
            }
            Value::Expression(Operation { operator, mut args }) => {
                match operator {
                    Operator::And => self
                        .append_goals(args.into_iter().map(|term| Goal::Query { term }).collect()),
                    Operator::Unify => {
                        assert_eq!(args.len(), 2);
                        let right = args.pop().unwrap();
                        let left = args.pop().unwrap();
                        self.push_goal(Goal::Unify { left, right })?;
                    }
                    Operator::Dot => {
                        assert_eq!(args.len(), 3);
                        let object = self.deref(&args[0]);
                        let field = &args[1];
                        let value = &args[2];

                        match object.value {
                            Value::Dictionary(dict) => self.push_goal(Goal::Lookup {
                                dict,
                                field: field.clone(),
                                value: args.remove(2),
                            })?,
                            Value::InstanceLiteral(_) => {
                                // Check if there's an external instance for this.
                                // If there is, use it, if not push a make external then use it.
                                // instantiate the instance and the predicate lookup
                                let (object, obj_goals) = self.instantiate_externals(&object);
                                let (field, field_goals) = self.instantiate_externals(field);
                                args[0] = object;
                                args[1] = field;
                                self.push_goal(Goal::Query {
                                    term: Term::new(Value::Expression(Operation {
                                        operator: Operator::Dot,
                                        args,
                                    })),
                                })?;
                                self.append_goals(field_goals);
                                self.append_goals(obj_goals);
                            }
                            Value::ExternalInstance(ExternalInstance { instance_id, .. }) => {
                                let value =
                                    value.clone().value.symbol().expect("Bad lookup value.");
                                let call_id = self.new_call_id(&value);
                                let (field, mut goals) = self.instantiate_externals(field);

                                goals.push(Goal::LookupExternal {
                                    call_id,
                                    instance_id,
                                    field,
                                });
                                self.append_goals(goals);
                            }
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    msg: format!(
                                    "can only perform lookups on dicts and instances, this is {:?}",
                                    object.value
                                ),
                                }
                                .into())
                            }
                        }
                    }
                    Operator::Or => self.choose(
                        args.iter()
                            .cloned()
                            .map(|term| vec![Goal::Query { term }])
                            .collect(),
                    ),
                    Operator::Not => {
                        assert_eq!(args.len(), 1);
                        let term = args.pop().unwrap();
                        let alternatives = vec![
                            vec![Goal::Query { term }, Goal::Cut, Goal::Backtrack],
                            vec![Goal::Noop],
                        ];

                        self.choose(alternatives);
                    }
                    op @ Operator::Lt
                    | op @ Operator::Gt
                    | op @ Operator::Leq
                    | op @ Operator::Geq
                    | op @ Operator::Eq
                    | op @ Operator::Neq => {
                        assert_eq!(args.len(), 2);
                        let left = self.deref(&args[0]).value;
                        let right = self.deref(&args[1]).value;
                        self.comparison_helper(op, left, right)?;
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: format!("can't query for expression: {:?}", operator),
                        }
                        .into())
                    }
                }
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: format!("can't query for: {}", term.value.to_polar()),
                }
                .into())
            }
        }
        Ok(QueryEvent::None)
    }

    /// Handle an external result provided by the application.
    ///
    /// If the value is `Some(_)` then we have a result, and bind the
    /// symbol associated with the call ID to the result value. If the
    /// value is `None` then the external has no (more) results, so we
    /// backtrack to the choice point left by `Goal::LookupExternal`.
    pub fn external_call_result(&mut self, call_id: u64, term: Option<Term>) -> PolarResult<()> {
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
            self.push_goal(Goal::Backtrack)?;
            self.push_goal(Goal::Cut)?;
        }
        Ok(())
    }

    /// Handle an external response to ExternalIsSubSpecializer,
    /// ExternalIsa, and ExternalUnify.
    pub fn external_question_result(&mut self, call_id: u64, answer: bool) {
        self.bind(
            &self
                .call_id_symbols
                .get(&call_id)
                .expect("unregistered external call ID")
                .clone(),
            &Term::new(Value::Boolean(answer)),
        );
        self.call_id_symbols.remove(&call_id).expect("bad call id");
    }

    /// Unify `left` and `right` terms.
    ///
    /// Outcomes of a unification are:
    ///  - Successful unification => bind zero or more variables to values
    ///  - Recursive unification => more `Unify` goals are pushed onto the stack
    ///  - Failure => backtrack
    fn unify(&mut self, left: &Term, right: &Term) -> PolarResult<()> {
        // Unify a symbol `left` with a term `right`.
        // This is sort of a "sub-goal" of `Unify`.
        let mut unify_var = |left: &Symbol, right: &Term| -> PolarResult<()> {
            let left_value = self.value(&left).cloned();
            let mut right_value = None;
            if let Value::Symbol(ref right_sym) = right.value {
                right_value = self.value(right_sym).cloned();
            }

            match (left_value, right_value) {
                (Some(left), Some(right)) => {
                    // Both are bound, unify their values.
                    self.push_goal(Goal::Unify { left, right })?;
                }
                (Some(left), _) => {
                    // Only left is bound, unify with whatever right is.
                    self.push_goal(Goal::Unify {
                        left,
                        right: right.clone(),
                    })?;
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
            Ok(())
        };

        // Unify generic terms.
        match (&left.value, &right.value) {
            // Unify symbols as variables.
            (Value::Symbol(var), _) => unify_var(var, right)?,
            (_, Value::Symbol(var)) => unify_var(var, left)?,

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
                    self.push_goal(Goal::Backtrack)?;
                }
            }

            (Value::Dictionary(left), Value::Dictionary(right)) => {
                // Check that the set of keys are the same.
                let left_fields: HashSet<&Symbol> = left.fields.keys().collect();
                let right_fields: HashSet<&Symbol> = right.fields.keys().collect();
                if left_fields != right_fields {
                    self.push_goal(Goal::Backtrack)?;
                    return Ok(());
                }

                // For each value, push a unify goal.
                for (k, v) in left.fields.iter() {
                    let right = right
                        .fields
                        .get(&k)
                        .expect("fields should be equal")
                        .clone();
                    self.push_goal(Goal::Unify {
                        left: v.clone(),
                        right,
                    })?
                }
            }

            // Unify integers by value.
            (Value::Integer(left), Value::Integer(right)) => {
                if left != right {
                    self.push_goal(Goal::Backtrack)?;
                }
            }

            // Unify strings by value.
            (Value::String(left), Value::String(right)) => {
                if left != right {
                    self.push_goal(Goal::Backtrack)?;
                }
            }

            // Unify bools by value.
            (Value::Boolean(left), Value::Boolean(right)) => {
                if left != right {
                    self.push_goal(Goal::Backtrack)?;
                }
            }

            // Unify predicates like unifying heads
            (Value::Call(left), Value::Call(right)) => {
                if left.name == right.name && left.args.len() == right.args.len() {
                    self.append_goals(
                        left.args
                            .iter()
                            .zip(right.args.iter())
                            .map(|(left, right)| Goal::Unify {
                                left: left.clone(),
                                right: right.clone(),
                            })
                            .collect(),
                    )
                } else {
                    self.push_goal(Goal::Backtrack)?
                }
            }

            // external instances can unify if they are exactly the same type, i.e. have
            // the same instance ID
            // this is necessary for the case that an instance appears multiple times
            // in the same rule head, for example
            (Value::ExternalInstance(_), Value::ExternalInstance(_)) => {
                return Err((RuntimeError::TypeError {
                    msg: String::from("Cannot unify two external instances."),
                })
                .into());
            }

            (Value::InstanceLiteral(_), Value::InstanceLiteral(_)) => {
                return Err((RuntimeError::TypeError {
                    msg: String::from("Cannot unify two instance literals."),
                })
                .into());
            }

            (Value::InstanceLiteral(_), Value::ExternalInstance(_))
            | (Value::ExternalInstance(_), Value::InstanceLiteral(_)) => {
                return Err((RuntimeError::TypeError {
                    msg: String::from("Cannot unify instance literal with external instance."),
                })
                .into());
            }

            // Anything else fails.
            (_, _) => self.push_goal(Goal::Backtrack)?,
        }

        Ok(())
    }

    /// Sort a list of rules with respect to a list of arguments
    /// using an explicit-state insertion sort.
    ///
    /// We maintain two indices for the sort, `outer` and `inner`. The `outer` index tracks our
    /// sorting progress. Every rule at or below `outer` is sorted; every rule above it is
    /// unsorted. The `inner` index tracks our search through the sorted sublist for the correct
    /// position of the candidate rule (the rule at the head of the unsorted portion of the
    /// list).
    fn sort_rules(
        &mut self,
        mut rules: Rules,
        args: TermList,
        outer: usize,
        inner: usize,
    ) -> PolarResult<()> {
        if rules.is_empty() {
            return self.push_goal(Goal::Backtrack);
        }

        assert!(outer <= rules.len(), "bad outer index");
        assert!(inner <= rules.len(), "bad inner index");
        assert!(inner <= outer, "bad insertion sort state");

        let next_outer = Goal::SortRules {
            rules: rules.clone(),
            args: args.clone(),
            outer: outer + 1,
            inner: outer + 1,
        };
        // Because `outer` starts as `1`, if there is only one rule in the `Rules`, this check
        // fails and we jump down to the evaluation of that lone rule.
        if outer < rules.len() {
            if inner > 0 {
                let compare = Goal::IsMoreSpecific {
                    left: rules[inner].clone(),
                    right: rules[inner - 1].clone(),
                    args: args.clone(),
                };

                rules.swap(inner - 1, inner);
                let next_inner = Goal::SortRules {
                    rules,
                    outer,
                    inner: inner - 1,
                    args,
                };
                // If the comparison fails, break out of the inner loop.
                // If the comparison succeeds, continue the inner loop with the swapped rules.
                self.choose(vec![vec![compare, Goal::Cut, next_inner], vec![next_outer]]);
            } else {
                assert_eq!(inner, 0);
                self.push_goal(next_outer)?;
            }
        } else {
            // We're done; the rules are sorted.
            // Make alternatives for calling them.
            let mut alternatives = vec![];
            for rule in rules.iter() {
                let Rule { body, params, .. } = self.rename_vars(rule);
                let mut goals = vec![];

                // Unify the arguments with the formal parameters.
                for (arg, param) in args.iter().zip(params.iter()) {
                    if let Some(name) = &param.name {
                        goals.push(Goal::Unify {
                            left: arg.clone(),
                            right: Term::new(Value::Symbol(name.clone())),
                        });
                    }
                    if let Some(specializer) = &param.specializer {
                        goals.push(Goal::Isa {
                            left: arg.clone(),
                            right: specializer.clone(),
                        });
                    }
                }

                // Query for the body clauses.
                goals.push(Goal::Query { term: body.clone() });

                alternatives.push(goals)
            }

            // Choose the first alternative, and push a choice for the rest.
            self.choose(alternatives);
        }
        Ok(())
    }

    /// Succeed if `left` is more specific than `right` with respect to `args`.
    fn is_more_specific(&mut self, left: Rule, right: Rule, args: TermList) -> PolarResult<()> {
        let zipped = left.params.iter().zip(right.params.iter()).zip(args.iter());
        for ((left_param, right_param), arg) in zipped {
            // TODO: Handle the case where one of the params has a specializer and the other does
            // not. The original logic in the python code was that a param with a specializer is
            // always more specific than a param without.
            if let (Some(left_spec), Some(right_spec)) =
                (&left_param.specializer, &right_param.specializer)
            {
                // If you find two non-equal specializers, that comparison determines the relative
                // specificity of the two rules completely. As soon as you have two specializers
                // that aren't the same and you can compare them and ask which one is more specific
                // to the relevant argument, you're done.
                if left_spec != right_spec {
                    let answer = self.kb.gensym("is_subspecializer");
                    // Bind answer to false as a starting point in case is subspecializer doesn't
                    // bind any result.
                    // This is done here for safety to avoid a bug where `answer` is unbound by
                    // `IsSubspecializer` and the `Unify` Goal just assigns it to `true` instead
                    // of checking that is is equal to `true`.
                    self.bind(&answer, &Term::new(Value::Boolean(false)));

                    self.append_goals(vec![
                        Goal::IsSubspecializer {
                            answer: answer.clone(),
                            left: left_spec.clone(),
                            right: right_spec.clone(),
                            arg: arg.clone(),
                        },
                        Goal::Unify {
                            left: Term::new(Value::Symbol(answer)),
                            right: Term::new(Value::Boolean(true)),
                        },
                    ]);
                    return Ok(());
                }
            }
        }
        // If neither rule is more specific, fail!
        self.push_goal(Goal::Backtrack)?;
        Ok(())
    }

    /// Determine if `left` is a more specific specializer ("subspecializer") than `right`
    fn is_subspecializer(
        &mut self,
        answer: Symbol,
        left: Term,
        right: Term,
        arg: Term,
    ) -> PolarResult<QueryEvent> {
        // If the arg is an instance literal, convert it to an external instance
        let (arg, mut goals) = self.instantiate_externals(&arg);
        if !goals.is_empty() {
            goals.push(Goal::IsSubspecializer {
                answer,
                left,
                right,
                arg,
            });
            self.append_goals(goals);
            return Ok(QueryEvent::None);
        }

        match (arg.value, left.value, right.value) {
            (
                Value::ExternalInstance(instance),
                Value::InstanceLiteral(left),
                Value::InstanceLiteral(right),
            ) => {
                let call_id = self.new_call_id(&answer);
                if left.tag == right.tag
                    && !(left.fields.fields.is_empty() && right.fields.fields.is_empty())
                {
                    self.push_goal(Goal::IsSubspecializer {
                        answer,
                        left: Term::new(Value::Dictionary(left.fields)),
                        right: Term::new(Value::Dictionary(right.fields)),
                        arg: Term::new(Value::ExternalInstance(instance.clone())),
                    })?;
                }
                // check ordering based on the classes
                Ok(QueryEvent::ExternalIsSubSpecializer {
                    call_id,
                    instance_id: instance.instance_id,
                    left_class_tag: left.tag,
                    right_class_tag: right.tag,
                })
            }
            (_, Value::Dictionary(left), Value::Dictionary(right)) => {
                let left_fields: HashSet<&Symbol> = left.fields.keys().collect();
                let right_fields: HashSet<&Symbol> = right.fields.keys().collect();

                // The dictionary with more fields is taken as more specific.
                // Assumption here that the rules have already been filtered for applicability,
                // and all fields are applicable.
                // This is a safe assumption because though rules are not currently pre-filtered,
                // inapplicable rules simply don't execute, and therefore their ordering is
                // irrelevant. Thus, the behavior is the same as if the rules were pre-filtered.
                if left_fields.len() != right_fields.len() {
                    self.bind(
                        &answer,
                        &Term::new(Value::Boolean(right_fields.len() < left.fields.len())),
                    );
                }
                Ok(QueryEvent::None)
            }
            (_, Value::InstanceLiteral(_), Value::Dictionary(_)) => {
                self.bind(&answer, &Term::new(Value::Boolean(true)));
                Ok(QueryEvent::None)
            }
            _ => {
                self.bind(&answer, &Term::new(Value::Boolean(false)));
                Ok(QueryEvent::None)
            }
        }
    }

    fn comparison_helper(&mut self, op: Operator, left: Value, right: Value) -> PolarResult<()> {
        let result: bool;
        match (op, left, right) {
            (Operator::Lt, Value::Integer(left), Value::Integer(right)) => {
                result = left < right;
            }
            (Operator::Leq, Value::Integer(left), Value::Integer(right)) => {
                result = left <= right;
            }
            (Operator::Gt, Value::Integer(left), Value::Integer(right)) => {
                result = left > right;
            }
            (Operator::Geq, Value::Integer(left), Value::Integer(right)) => {
                result = left >= right;
            }
            (Operator::Eq, Value::Integer(left), Value::Integer(right)) => {
                result = left == right;
            }
            (Operator::Neq, Value::Integer(left), Value::Integer(right)) => {
                result = left != right;
            }
            (op, left, right) => {
                return Err(RuntimeError::TypeError {
                    msg: format!(
                        "{} expects integers, got: {}, {}",
                        op.to_polar(),
                        left.to_polar(),
                        right.to_polar()
                    ),
                }
                .into());
            }
        }
        if !result {
            self.push_goal(Goal::Backtrack)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use permute::permute;

    /// Shorthand for constructing Goal::Query.
    ///
    /// A one argument invocation assumes the 1st argument is the same
    /// parameters that can be passed to the term! macro.  In this invocation,
    /// typically the form `query!(op!(And, term!(TERM)))` will be used. The
    /// one argument form allows for queries with a top level operator other
    /// than AND.
    ///
    /// Multiple arguments `query!(f1, f2, f3)` result in a query with a root
    /// AND operator term.
    macro_rules! query {
        ($term:expr) => {
            Goal::Query {
                term: term!($term)
            }
        };
        ($($term:expr),+) => {
            Goal::Query {
                term: term!(op!(And, $($term),+))
            }
        };
    }

    /// Macro takes two arguments, the vm and a list-like structure of
    /// QueryEvents to expect.  It will call run() for each event in the second
    /// argument and pattern match to check that the event matches what is
    /// expected.  Then `vm.is_halted()` is checked.
    ///
    /// The QueryEvent list elements can either be:
    ///   - QueryEvent::Result{EXPR} where EXPR is a HashMap<Symbol, Term>.
    ///     This is shorthand for QueryEvent::Result{bindings} if bindings == EXPR.
    ///     Use btreemap! for EXPR from the maplit package to write inline hashmaps
    ///     to assert on.
    ///   - A pattern with optional guard accepted by matches!. (QueryEvent::Result
    ///     cannot be matched on due to the above rule.)
    macro_rules! assert_query_events {
        ($vm:ident, []) => {
            assert!($vm.is_halted());
        };
        ($vm:ident, [QueryEvent::Result{$result:expr}]) => {
            assert!(matches!($vm.run().unwrap(), QueryEvent::Result{bindings} if bindings == $result));
            assert_query_events!($vm, []);
        };
        ($vm:ident, [QueryEvent::Result{$result:expr}, $($tail:tt)*]) => {
            assert!(matches!($vm.run().unwrap(), QueryEvent::Result{bindings} if bindings == $result));
            assert_query_events!($vm, [$($tail)*]);
        };
        ($vm:ident, [$( $pattern:pat )|+ $( if $guard: expr )?]) => {
            assert!(matches!($vm.run().unwrap(), $($pattern)|+ $(if $guard)?));
            assert_query_events!($vm, []);
        };
        ($vm:ident, [$( $pattern:pat )|+ $( if $guard: expr )?, $($tail:tt)*]) => {
            assert!(matches!($vm.run().unwrap(), $($pattern)|+ $(if $guard)?));
            assert_query_events!($vm, [$($tail)*]);
        };
        // TODO (dhatch) Be able to use btreemap! to match on specific bindings.
    }

    #[test]
    fn deref() {
        let mut vm = PolarVirtualMachine::default();
        let value = term!(1);
        let x = sym!("x");
        let y = sym!("y");
        let term_x = term!(x.clone());
        let term_y = term!(y.clone());

        // unbound var
        assert_eq!(vm.deref(&term_x), term_x);

        // unbound var -> unbound var
        vm.bind(&x, &term_y);
        assert_eq!(vm.deref(&term_x), term_y);

        // value
        assert_eq!(vm.deref(&value), value.clone());

        // unbound var -> value
        vm.bind(&x, &value);
        assert_eq!(vm.deref(&term_x), value);

        // unbound var -> unbound var -> value
        vm.bind(&x, &term_y);
        vm.bind(&y, &value);
        assert_eq!(vm.deref(&term_x), value);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn and_expression() {
        let f1 = rule!("f", [1]);
        let f2 = rule!("f", [2]);

        let rule = GenericRule {
            name: sym!("f"),
            rules: vec![f1, f2],
        };

        let mut kb = KnowledgeBase::new();
        kb.rules.insert(rule.name.clone(), rule);

        let goal = query!(op!(And));

        let mut vm = PolarVirtualMachine::new(Arc::new(kb), vec![goal]);
        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!()},
            QueryEvent::Done
        ]);

        assert!(vm.is_halted());

        let f1 = term!(pred!("f", [1]));
        let f2 = term!(pred!("f", [2]));
        let f3 = term!(pred!("f", [3]));

        // Querying for f(1)
        vm.push_goal(query!(op!(And, f1.clone()))).unwrap();

        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{}},
            QueryEvent::Done
        ]);

        // Querying for f(1), f(2)
        vm.push_goal(query!(f1.clone(), f2.clone())).unwrap();
        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{}},
            QueryEvent::Done
        ]);

        // Querying for f(3)
        vm.push_goal(query!(op!(And, f3.clone()))).unwrap();
        assert_query_events!(vm, [QueryEvent::Done]);

        // Querying for f(1), f(2), f(3)
        let parts = vec![f1, f2, f3];
        for permutation in permute(parts) {
            vm.push_goal(Goal::Query {
                term: Term::new(Value::Expression(Operation {
                    operator: Operator::And,
                    args: permutation,
                })),
            })
            .unwrap();
            assert_query_events!(vm, [QueryEvent::Done]);
        }
    }

    #[test]
    fn unify_expression() {
        let mut vm = PolarVirtualMachine::default();
        vm.push_goal(query!(op!(Unify, term!(1), term!(1))))
            .unwrap();

        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{}},
            QueryEvent::Done
        ]);

        let q = op!(Unify, term!(1), term!(2));
        vm.push_goal(query!(q)).unwrap();

        assert_query_events!(vm, [QueryEvent::Done]);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn isa_on_lists() {
        let mut vm = PolarVirtualMachine::default();
        let one = Term::new(Value::Integer(1));
        let two = Term::new(Value::Integer(2));
        let one_list = Term::new(Value::List(vec![one.clone()]));
        let one_two_list = Term::new(Value::List(vec![one.clone(), two.clone()]));
        let two_one_list = Term::new(Value::List(vec![two, one.clone()]));
        let empty_list = Term::new(Value::List(vec![]));

        // [] isa []
        vm.push_goal(Goal::Isa {
            left: empty_list.clone(),
            right: empty_list.clone(),
        })
        .unwrap();
        assert!(matches!(vm.run().unwrap(), QueryEvent::Result{bindings} if bindings.is_empty()));
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // [1,2] isa [1,2]
        vm.push_goal(Goal::Isa {
            left: one_two_list.clone(),
            right: one_two_list.clone(),
        })
        .unwrap();
        assert!(matches!(vm.run().unwrap(), QueryEvent::Result{bindings} if bindings.is_empty()));
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // [1,2] isNOTa [2,1]
        vm.push_goal(Goal::Isa {
            left: one_two_list.clone(),
            right: two_one_list,
        })
        .unwrap();
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // [1] isNOTa [1,2]
        vm.push_goal(Goal::Isa {
            left: one_list.clone(),
            right: one_two_list.clone(),
        })
        .unwrap();
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // [1,2] isNOTa [1]
        vm.push_goal(Goal::Isa {
            left: one_two_list,
            right: one_list.clone(),
        })
        .unwrap();
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // [1] isNOTa []
        vm.push_goal(Goal::Isa {
            left: one_list.clone(),
            right: empty_list.clone(),
        })
        .unwrap();
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // [] isNOTa [1]
        vm.push_goal(Goal::Isa {
            left: empty_list,
            right: one_list.clone(),
        })
        .unwrap();
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // [1] isNOTa 1
        vm.push_goal(Goal::Isa {
            left: one_list.clone(),
            right: one.clone(),
        })
        .unwrap();
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // 1 isNOTa [1]
        vm.push_goal(Goal::Isa {
            left: one,
            right: one_list,
        })
        .unwrap();
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn isa_on_dicts() {
        let mut vm = PolarVirtualMachine::default();
        let left = term!(btreemap! {
            sym!("x") => term!(1),
            sym!("y") => term!(2),
        });
        let right = term!(btreemap! {
            sym!("x") => term!(1),
            sym!("y") => term!(2),
        });
        vm.push_goal(Goal::Isa {
            left: left.clone(),
            right,
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done]);

        // Dicts with identical keys and different values DO NOT isa.
        let right = term!(btreemap! {
            sym!("x") => term!(2),
            sym!("y") => term!(1),
        });
        vm.push_goal(Goal::Isa {
            left: left.clone(),
            right,
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Done]);

        // {} isa {}.
        vm.push_goal(Goal::Isa {
            left: term!(btreemap! {}),
            right: term!(btreemap! {}),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done]);

        // Non-empty dicts should isa against an empty dict.
        vm.push_goal(Goal::Isa {
            left: left.clone(),
            right: term!(btreemap! {}),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done]);

        // Empty dicts should NOT isa against a non-empty dict.
        vm.push_goal(Goal::Isa {
            left: term!(btreemap! {}),
            right: left.clone(),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Done]);

        // Superset dict isa subset dict.
        vm.push_goal(Goal::Isa {
            left: left.clone(),
            right: term!(btreemap! {sym!("x") => term!(1)}),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done]);

        // Subset dict isNOTa superset dict.
        vm.push_goal(Goal::Isa {
            left: term!(btreemap! {sym!("x") => term!(1)}),
            right: left,
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Done]);
    }

    #[test]
    fn unify_dicts() {
        let mut vm = PolarVirtualMachine::default();
        // Dicts with identical keys and values unify.
        let left = term!(btreemap! {
            sym!("x") => term!(1),
            sym!("y") => term!(2),
        });
        let right = term!(btreemap! {
            sym!("x") => term!(1),
            sym!("y") => term!(2),
        });
        vm.push_goal(Goal::Unify {
            left: left.clone(),
            right,
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done]);

        // Dicts with identical keys and different values DO NOT unify.
        let right = term!(btreemap! {
            sym!("x") => term!(2),
            sym!("y") => term!(1),
        });
        vm.push_goal(Goal::Unify {
            left: left.clone(),
            right,
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Done]);

        // Empty dicts unify.
        vm.push_goal(Goal::Unify {
            left: term!(btreemap! {}),
            right: term!(btreemap! {}),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done]);

        // Empty dict should not unify against a non-empty dict.
        vm.push_goal(Goal::Unify {
            left: left.clone(),
            right: term!(btreemap! {}),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Done]);

        // Subset match should fail.
        let right = term!(btreemap! {
            sym!("x") => term!(1),
        });
        vm.push_goal(Goal::Unify { left, right }).unwrap();
        assert_query_events!(vm, [QueryEvent::Done]);
    }

    #[test]
    fn unify_nested_dicts() {
        let mut vm = PolarVirtualMachine::default();

        let left = term!(btreemap! {
            sym!("x") => term!(btreemap!{
                sym!("y") => term!(1)
            })
        });
        let right = term!(btreemap! {
            sym!("x") => term!(btreemap!{
                sym!("y") => term!(sym!("result"))
            })
        });
        vm.push_goal(Goal::Unify { left, right }).unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!{sym!("result") => term!(1)} }, QueryEvent::Done]);
    }

    #[test]
    fn lookup() {
        let mut vm = PolarVirtualMachine::default();

        let fields = btreemap! {
            sym!("x") => term!(1),
        };
        let dict = Dictionary { fields };
        vm.push_goal(Goal::Lookup {
            dict: dict.clone(),
            field: term!(pred!("x", [])),
            value: term!(1),
        })
        .unwrap();

        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{}}
        ]);

        // Lookup with incorrect value
        vm.push_goal(Goal::Lookup {
            dict: dict.clone(),
            field: term!(pred!("x", [])),
            value: term!(2),
        })
        .unwrap();

        assert_query_events!(vm, [QueryEvent::Done]);

        // Lookup with unbound value
        vm.push_goal(Goal::Lookup {
            dict,
            field: term!(pred!("x", [])),
            value: term!(sym!("y")),
        })
        .unwrap();
        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{sym!("y") => term!(1)}}
        ]);
    }

    #[test]
    fn bind() {
        let x = sym!("x");
        let y = sym!("y");
        let zero = term!(0);
        let mut vm = PolarVirtualMachine::default();
        vm.bind(&x, &zero);
        assert_eq!(vm.value(&x), Some(&zero));
        assert_eq!(vm.value(&y), None);
    }

    #[test]
    fn debug() {
        let mut vm = PolarVirtualMachine::new(
            Arc::new(KnowledgeBase::new()),
            vec![Goal::Debug {
                message: "Hello".to_string(),
            }],
        );
        assert!(matches!(
            vm.run().unwrap(),
            QueryEvent::Debug { message } if &message[..] == "Hello"
        ));
    }

    #[test]
    fn halt() {
        let mut vm = PolarVirtualMachine::new(Arc::new(KnowledgeBase::new()), vec![Goal::Halt]);
        let _ = vm.run().unwrap();
        assert_eq!(vm.goals.len(), 0);
        assert_eq!(vm.bindings.len(), 0);
    }

    #[test]
    fn unify() {
        let x = sym!("x");
        let y = sym!("y");
        let vars = term!([x.clone(), y.clone()]);
        let zero = value!(0);
        let one = value!(1);
        let vals = term!([zero.clone(), one.clone()]);
        let mut vm = PolarVirtualMachine::new(
            Arc::new(KnowledgeBase::new()),
            vec![Goal::Unify {
                left: vars,
                right: vals,
            }],
        );
        let _ = vm.run().unwrap();
        assert_eq!(vm.value(&x), Some(&Term::new(zero)));
        assert_eq!(vm.value(&y), Some(&Term::new(one)));
    }

    #[test]
    fn unify_var() {
        let x = sym!("x");
        let y = sym!("y");
        let z = sym!("z");
        let one = term!(1);
        let two = term!(2);

        let mut vm = PolarVirtualMachine::default();

        // Left variable bound to bound right variable.
        vm.bind(&y, &one);
        vm.append_goals(vec![Goal::Unify {
            left: term!(x),
            right: term!(y),
        }]);
        let _ = vm.run().unwrap();
        assert_eq!(vm.value(&sym!("x")), Some(&one));
        vm.backtrack().unwrap();

        // Left variable bound to value.
        vm.bind(&z, &one);
        vm.append_goals(vec![Goal::Unify {
            left: term!(z.clone()),
            right: one.clone(),
        }]);
        let _ = vm.run().unwrap();
        assert_eq!(vm.value(&z), Some(&one));

        // Left variable bound to value
        vm.bind(&z, &one);
        vm.append_goals(vec![Goal::Unify {
            left: term!(z.clone()),
            right: two,
        }]);
        let _ = vm.run().unwrap();
        assert_eq!(vm.value(&z), Some(&one));
    }

    #[test]
    fn test_gen_var() {
        let mut vm = PolarVirtualMachine::default();

        let rule = Rule {
            name: Symbol::new("foo"),
            params: vec![],
            body: Term::new(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![
                    Term::new(Value::Integer(1)),
                    Term::new(Value::Symbol(Symbol("x".to_string()))),
                    Term::new(Value::Symbol(Symbol("x".to_string()))),
                    Term::new(Value::List(vec![Term::new(Value::Symbol(Symbol(
                        "y".to_string(),
                    )))])),
                ],
            })),
        };

        let renamed_rule = vm.rename_vars(&rule);

        let renamed_terms = unwrap_and(renamed_rule.body);
        assert_eq!(renamed_terms[1].value, renamed_terms[2].value);
        let x_value = match &renamed_terms[1].value {
            Value::Symbol(sym) => Some(sym.0.clone()),
            _ => None,
        };
        assert_eq!(x_value.unwrap(), "_x_0");

        let y_value = match &renamed_terms[3].value {
            Value::List(terms) => match &terms[0].value {
                Value::Symbol(sym) => Some(sym.0.clone()),
                _ => None,
            },
            _ => None,
        };
        assert_eq!(y_value.unwrap(), "_y_1");
    }

    #[test]
    fn test_sort_rules() {
        // Test sort rule by mocking ExternalIsSubSpecializer and ExternalIsa.
        let bar_rule = GenericRule::new(
            sym!("bar"),
            vec![
                rule!("bar", [instance!("b"), instance!("a"), value!(3)]),
                rule!("bar", [instance!("a"), instance!("a"), value!(1)]),
                rule!("bar", [instance!("a"), instance!("b"), value!(2)]),
                rule!("bar", [instance!("b"), instance!("b"), value!(4)]),
            ],
        );

        let mut kb = KnowledgeBase::new();
        kb.add_generic_rule(bar_rule);

        let mut vm = PolarVirtualMachine::new(
            Arc::new(kb),
            vec![query!(pred!(
                "bar",
                [instance!("doesn't"), instance!("matter"), sym!("z")]
            ))],
        );

        let mut results = Vec::new();
        loop {
            match vm.run().unwrap() {
                QueryEvent::Done => break,
                QueryEvent::Result { bindings } => results.push(bindings),
                QueryEvent::ExternalIsSubSpecializer {
                    call_id,
                    left_class_tag,
                    right_class_tag,
                    ..
                } => {
                    // For this test we sort classes lexically.
                    vm.external_question_result(call_id, left_class_tag < right_class_tag)
                }
                QueryEvent::MakeExternal { .. } => (),
                QueryEvent::ExternalIsa { call_id, .. } => {
                    // For this test, anything is anything.
                    vm.external_question_result(call_id, true)
                }
                _ => panic!("Unexpected event"),
            }
        }

        assert_eq!(results.len(), 4);
        assert_eq!(
            results,
            vec![
                hashmap! { sym!("z") => term!(1) },
                hashmap! { sym!("z") => term!(2) },
                hashmap! { sym!("z") => term!(3) },
                hashmap! { sym!("z") => term!(4) },
            ]
        );
    }

    #[test]
    fn test_is_subspecializer() {
        let mut vm = PolarVirtualMachine::default();

        // Test `is_subspecializer` case where:
        // - arg: `ExternalInstance`
        // - left: `InstanceLiteral`
        // - right: `Dictionary`
        let arg = term!(Value::ExternalInstance(ExternalInstance {
            instance_id: 1,
            literal: None,
        }));
        let left = term!(value!(InstanceLiteral {
            tag: sym!("Any"),
            fields: Dictionary {
                fields: btreemap! {}
            }
        }));
        let right = term!(Value::Dictionary(Dictionary {
            fields: btreemap! {sym!("a") => term!("a")},
        }));

        let answer = vm.kb.gensym("is_subspecializer");

        match vm
            .is_subspecializer(answer.clone(), left, right, arg)
            .unwrap()
        {
            QueryEvent::None => (),
            event => panic!("Expected None, got {:?}", event),
        }

        assert_eq!(vm.deref(&term!(Value::Symbol(answer))), term!(value!(true)));
    }
}
