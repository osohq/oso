use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::rc::Rc;
use std::string::ToString;
use std::sync::{Arc, RwLock};

use super::debugger::{DebugEvent, Debugger};
use super::error;
use super::formatting::draw;
use super::lexer::{loc_to_pos, make_context};
use super::types::*;
use super::{PolarResult, ToPolarString};

pub const MAX_CHOICES: usize = 10_000;
pub const MAX_GOALS: usize = 10_000;
pub const MAX_EXECUTED_GOALS: usize = 10_000;

#[derive(Clone, Debug)]
#[must_use = "ignored goals are never accomplished"]
#[allow(clippy::large_enum_variant)]
pub enum Goal {
    Backtrack,
    Cut {
        choice_index: usize, // cuts all choices in range [choice_index..]
    },
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
        call_id: u64,
        instance: Term,
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
    UnifyExternal {
        left_instance_id: u64,
        right_instance_id: u64,
    },
    Noop,
    Query {
        term: Term,
    },
    PopQuery {
        term: Term,
    },
    FilterRules {
        args: TermList,
        applicable_rules: Rules,
        unfiltered_rules: Rules,
    },
    SortRules {
        args: TermList,
        rules: Rules,
        outer: usize,
        inner: usize,
    },
    TraceRule {
        trace: Rc<Trace>,
    },
    TracePush,
    TracePop,
    Unify {
        left: Term,
        right: Term,
    },
}

#[derive(Clone, Debug)]
pub struct Binding(pub Symbol, pub Term);

#[derive(Clone, Debug)]
pub struct Choice {
    pub alternatives: Vec<GoalStack>,
    bsp: usize,            // binding stack pointer
    pub goals: GoalStack,  // goal stack snapshot
    queries: Queries,      // query stack snapshot
    trace: Vec<Rc<Trace>>, // trace snapshot
    trace_stack: Vec<Vec<Rc<Trace>>>,
}

pub type BindingStack = Vec<Binding>;
pub type Choices = Vec<Choice>;
/// Shortcut type alias for a list of goals
pub type Goals = Vec<Goal>;
#[derive(Clone, Debug, Default)]
pub struct GoalStack(Vec<Rc<Goal>>);

impl GoalStack {
    fn new_reversed(goals: Goals) -> Self {
        Self(goals.into_iter().rev().map(Rc::new).collect())
    }
}

// impl From<Vec<Goal>> for GoalStack {
//     fn from(other: Vec<Goal>) -> Self {
//         Self(other.into_iter().map(Rc::new).collect())
//     }
// }

impl std::ops::Deref for GoalStack {
    type Target = Vec<Rc<Goal>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for GoalStack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub type Queries = TermList;

#[derive(Default)]
pub struct PolarVirtualMachine {
    /// Stacks.
    pub goals: GoalStack,
    pub bindings: BindingStack,
    choices: Choices,
    pub queries: Queries,

    pub trace_stack: Vec<Vec<Rc<Trace>>>, // Stack of traces higher up the tree.
    pub trace: Vec<Rc<Trace>>,            // Traces for the current level of the trace tree.

    /// Binding stack constant below here.
    csp: usize,

    /// Executed goal counter.
    goal_counter: usize,

    /// Interactive debugger.
    pub debugger: Debugger,

    /// Rules and types.
    pub kb: Arc<RwLock<KnowledgeBase>>,

    /// Call ID -> result variable name table.
    call_id_symbols: HashMap<u64, Symbol>,

    /// Logging flag.
    log: bool,
}

// Methods which aren't goals/instructions.
impl PolarVirtualMachine {
    /// Make a new virtual machine with an initial list of goals.
    /// Reverse the goal list for the sanity of callers.
    pub fn new(kb: Arc<RwLock<KnowledgeBase>>, goals: Goals) -> Self {
        let constants = kb
            .read()
            .expect("cannot acquire KB read lock")
            .constants
            .clone();
        let mut vm = Self {
            goals: GoalStack::new_reversed(goals),
            bindings: vec![],
            csp: 0,
            choices: vec![],
            goal_counter: 0,
            queries: vec![],
            trace_stack: vec![],
            trace: vec![],
            debugger: Debugger::default(),
            kb,
            call_id_symbols: HashMap::new(),
            log: std::env::var("RUST_LOG").is_ok(),
        };
        vm.bind_constants(constants);
        vm
    }

    pub fn new_id(&self) -> u64 {
        self.kb
            .read()
            .expect("cannot acquire KB read lock")
            .new_id()
    }

    fn new_call_id(&mut self, symbol: &Symbol) -> u64 {
        let call_id = self.new_id();
        self.call_id_symbols.insert(call_id, symbol.clone());
        call_id
    }

    /// Try to achieve one goal. Return `Some(QueryEvent)` if an external
    /// result is needed to achieve it, or `None` if it can run internally.
    fn next(&mut self, goal: Rc<Goal>) -> PolarResult<QueryEvent> {
        if self.log {
            eprintln!("{}", goal);
        }
        self.goal_counter += 1;
        match goal.as_ref() {
            Goal::Backtrack => self.backtrack()?,
            Goal::Cut { choice_index } => self.cut(*choice_index),
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
                instance,
                field,
            } => return self.lookup_external(*call_id, instance, field),
            Goal::IsaExternal {
                instance_id,
                literal,
            } => return self.isa_external(*instance_id, literal),
            Goal::UnifyExternal {
                left_instance_id,
                right_instance_id,
            } => return self.unify_external(*left_instance_id, *right_instance_id),
            Goal::MakeExternal {
                literal,
                instance_id,
            } => return Ok(self.make_external(literal, *instance_id)),
            Goal::Noop => {}
            Goal::Query { term } => {
                let result = self.query(term);
                self.maybe_break(DebugEvent::Query)?;
                return result;
            }
            Goal::PopQuery { .. } => self.pop_query(),
            Goal::FilterRules {
                applicable_rules,
                unfiltered_rules,
                args,
            } => self.filter_rules(applicable_rules, unfiltered_rules, args)?,
            Goal::SortRules {
                rules,
                outer,
                inner,
                args,
            } => self.sort_rules(rules, args, *outer, *inner)?,
            Goal::TracePush => {
                self.trace_stack.push(self.trace.clone());
                self.trace = vec![];
            }
            Goal::TracePop => {
                let mut children = self.trace.clone();
                self.trace = self.trace_stack.pop().unwrap();
                let mut trace = self.trace.pop().unwrap();
                let trace = Rc::make_mut(&mut trace);
                trace.children.append(&mut children);
                self.trace.push(Rc::new(trace.clone()));
            }
            Goal::TraceRule { trace } => {
                self.trace.push(trace.clone());
            }
            Goal::Unify { left, right } => self.unify(&left, &right)?,
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
            self.maybe_break(DebugEvent::Goal(goal.clone()))?;
        }

        if self.log {
            eprintln!("⇒ result");
            for t in &self.trace {
                eprintln!("trace\n{}", draw(t, 0));
            }
        }

        Ok(QueryEvent::Result {
            bindings: self.bindings(false),
            trace: self.trace.first().cloned(),
        })
    }

    /// Return true if there is nothing left to do.
    pub fn is_halted(&self) -> bool {
        self.goals.is_empty() && self.choices.is_empty()
    }

    /// Push a goal onto the goal stack.
    pub fn push_goal(&mut self, goal: Goal) -> PolarResult<()> {
        if self.goals.len() >= MAX_GOALS {
            return Err(error::RuntimeError::StackOverflow {
                msg: format!("Goal stack overflow! MAX_GOALS = {}", MAX_GOALS),
            }
            .into());
        }
        if self.goal_counter >= MAX_EXECUTED_GOALS {
            return Err(error::RuntimeError::StackOverflow {
                msg: format!(
                    "Goal count exceeded! MAX_EXECUTED_GOALS = {}",
                    MAX_EXECUTED_GOALS
                ),
            }
            .into());
        }
        self.goals.push(Rc::new(goal));
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
    /// ~~Do nothing if there are no alternatives; this saves every caller a
    /// conditional, and maintains the invariant that only choice points with
    /// alternatives are on the choice stack.~~ TODO: this comment is not true any more
    fn push_choice<I>(&mut self, alternatives: I)
    where
        I: IntoIterator<Item = Goals>,
        I::IntoIter: std::iter::DoubleEndedIterator,
    {
        // Make sure that alternatives are executed in order of first to last.
        let alternatives = alternatives
            .into_iter()
            .rev()
            .map(GoalStack::new_reversed)
            .collect();
        assert!(self.choices.len() < MAX_CHOICES, "too many choices");
        self.choices.push(Choice {
            alternatives,
            bsp: self.bsp(),
            goals: self.goals.clone(),
            queries: self.queries.clone(),
            trace: self.trace.clone(),
            trace_stack: self.trace_stack.clone(),
        });
    }

    /// Push a choice onto the choice stack, and execute immediately by
    /// pushing the first alternative onto the goals stack
    ///
    /// Params:
    ///
    /// - `alternatives`: an ordered list of alternatives to try in the choice.
    ///   The first element is the first alternative to try.
    fn choose<I>(&mut self, alternatives: I) -> PolarResult<()>
    where
        I: IntoIterator<Item = Goals>,
        I::IntoIter: std::iter::DoubleEndedIterator,
    {
        let mut alternatives_iter = alternatives.into_iter();
        if let Some(alternative) = alternatives_iter.next() {
            self.push_choice(alternatives_iter);
            self.append_goals(alternative)?;
        }
        Ok(())
    }

    /// Push multiple goals onto the stack in reverse order.
    fn append_goals<I>(&mut self, goals: I) -> PolarResult<()>
    where
        I: IntoIterator<Item = Goal>,
        I::IntoIter: std::iter::DoubleEndedIterator,
    {
        goals
            .into_iter()
            .rev()
            .try_for_each(|goal| self.push_goal(goal))
    }

    /// Push a binding onto the binding stack.
    fn bind(&mut self, var: &Symbol, value: Term) {
        if self.log {
            eprintln!("⇒ bind: {} ← {}", var.to_polar(), value.to_polar());
        }
        self.bindings.push(Binding(var.clone(), value));
    }

    /// Augment the bindings stack with constants from a hash map.
    /// There must be no temporaries bound yet.
    pub fn bind_constants(&mut self, bindings: Bindings) {
        assert_eq!(self.bsp(), self.csp);
        for (var, value) in bindings.iter() {
            self.bind(var, value.clone());
        }
        self.csp += bindings.len();
    }

    /// Retrieve the current non-constant bindings as a hash map.
    pub fn bindings(&self, include_temps: bool) -> Bindings {
        let mut bindings = HashMap::new();
        for Binding(var, value) in &self.bindings[self.csp..] {
            if !include_temps && self.is_temporary_var(&var) {
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

    /// Recursively dereference a variable.
    pub fn deref(&self, term: &Term) -> Term {
        match &term.value() {
            Value::Variable(symbol) | Value::RestVariable(symbol) => {
                self.value(&symbol).map_or(term.clone(), |t| self.deref(t))
            }
            _ => term.clone(),
        }
    }

    /// Return `true` if `var` is a temporary variable.
    fn is_temporary_var(&self, name: &Symbol) -> bool {
        name.0.starts_with('_')
    }

    /// Return `true` if `var` is a constant variable.
    fn is_constant_var(&self, name: &Symbol) -> bool {
        self.bindings
            .iter()
            .take(self.csp)
            .any(|binding| binding.0 == *name)
    }

    /// Generate a fresh set of variables for an argument list.
    fn rename_vars(&self, terms: TermList) -> TermList {
        let mut renames = HashMap::<Symbol, Symbol>::new();
        terms
            .iter()
            .map(|t| {
                t.cloned_map_replace(&mut |t| match t.value() {
                    Value::Variable(sym) | Value::RestVariable(sym)
                        if !self.is_constant_var(sym) =>
                    {
                        if let Some(new) = renames.get(sym) {
                            t.clone_with_value(Value::Variable(new.clone()))
                        } else {
                            let new = self.kb.read().unwrap().gensym(&sym.0);
                            renames.insert(sym.clone(), new.clone());
                            t.clone_with_value(Value::Variable(new))
                        }
                    }
                    _ => t.clone(),
                })
            })
            .collect()
    }

    /// Generate a fresh set of variables for a rule.
    fn rename_rule_vars(&self, rule: &Rule) -> Rule {
        let mut renames = HashMap::<Symbol, Symbol>::new();
        let mut rule = rule.clone();
        rule.map_replace(&mut move |term| match term.value() {
            Value::Variable(sym) if !self.is_constant_var(sym) => {
                if let Some(new) = renames.get(sym) {
                    term.clone_with_value(Value::Variable(new.clone()))
                } else {
                    let new = self.kb.read().unwrap().gensym(&sym.0);
                    renames.insert(sym.clone(), new.clone());
                    term.clone_with_value(Value::Variable(new))
                }
            }
            Value::RestVariable(sym) => {
                if let Some(new) = renames.get(sym) {
                    term.clone_with_value(Value::RestVariable(new.clone()))
                } else {
                    let new = self.kb.read().unwrap().gensym(&sym.0);
                    renames.insert(sym.clone(), new.clone());
                    term.clone_with_value(Value::RestVariable(new))
                }
            }
            _ => term.clone(),
        });
        rule
    }

    /// Get the query stack as a string for printing in error messages.
    fn stack_trace(&self) -> String {
        let mut trace_stack = self.trace_stack.clone();
        let mut trace = self.trace.clone();

        // Build linear stack from trace tree. Not just using query stack because it doesn't
        // know about rules, query stack should really use this too.
        let mut stack = vec![];
        while let Some(t) = trace.last() {
            stack.push(t.clone());
            trace = trace_stack.pop().unwrap_or_else(Vec::new);
        }

        stack.reverse();

        let mut st = String::new();
        write!(st, "trace (most recent evaluation last):").unwrap();

        let mut rule = None;
        for t in stack {
            match &t.node {
                Node::Rule(r) => {
                    rule = Some(r.clone());
                }
                Node::Term(t) => {
                    write!(st, "\n  ").unwrap();
                    let source = { self.kb.read().unwrap().sources.get_source(&t) };
                    if let Some(source) = source {
                        if let Some(rule) = &rule {
                            write!(st, "in rule {} ", rule.name.to_polar()).unwrap();
                        } else {
                            write!(st, "in query ").unwrap();
                        }
                        let (row, column) = loc_to_pos(&source.src, t.offset());
                        write!(st, "at line {}, column {}", row + 1, column + 1).unwrap();
                        if let Some(filename) = source.filename {
                            write!(st, " in file {}", filename).unwrap();
                        }
                        writeln!(st).unwrap();
                    };
                    write!(st, "    {}", t.to_polar()).unwrap();
                }
            }
        }
        st
    }
}

/// Implementations of instructions.
impl PolarVirtualMachine {
    /// Remove all bindings after the last choice point, and try the
    /// next available alternative. If no choice is possible, halt.
    fn backtrack(&mut self) -> PolarResult<()> {
        if self.log {
            eprintln!("⇒ backtrack");
        }
        loop {
            match self.choices.last_mut() {
                None => return self.push_goal(Goal::Halt),
                Some(Choice {
                    alternatives,
                    bsp,
                    goals,
                    queries,
                    trace,
                    trace_stack,
                }) => {
                    self.bindings.drain(*bsp..);
                    if let Some(mut alternative) = alternatives.pop() {
                        self.goals = goals.clone();
                        self.queries = queries.clone();
                        self.trace = trace.clone();
                        self.trace_stack = trace_stack.clone();
                        // Note: here we are directly modifying the
                        // VM goal stack, since what we have is already
                        // a "stored goal stack"
                        self.goals.append(&mut alternative);

                        break; // we have our alternative, end the loop
                    }

                    // falling through means no alternatives found
                    let _ = self.choices.pop();
                }
            }
        }
        Ok(())
    }

    /// Commit to the current choice.
    fn cut(&mut self, index: usize) {
        let _ = self.choices.drain(index..);
    }

    /// Clean up the query stack after completing a query.
    fn pop_query(&mut self) {
        self.queries.pop();
    }

    /// Interact with the debugger.
    fn debug(&self, message: &str) -> QueryEvent {
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
        // TODO (dhatch): These errors could potentially be caused by the user.
        // rule(foo) :=
        //    x = {a: 1},
        //    foo isa x
        assert!(
            !matches!(&right.value(), Value::InstanceLiteral(_)),
            "Called isa with bare instance lit!"
        );
        assert!(
            !matches!(&right.value(), Value::Dictionary(_)),
            "Called isa with bare dictionary!"
        );

        match (&left.value(), &right.value()) {
            (Value::List(left), Value::List(right)) => {
                self.unify_lists(left, right, |(left, right)| Goal::Isa {
                    left: left.clone(),
                    right: right.clone(),
                })?;
            }

            (Value::Dictionary(left), Value::Pattern(Pattern::Dictionary(right))) => {
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
                panic!("How did an instance literal get here???");
            }

            (Value::ExternalInstance(_), Value::Pattern(Pattern::Dictionary(right))) => {
                // For each field in the dict, look up the corresponding field on the instance and
                // then isa them.
                for (field, right_value) in right.fields.iter() {
                    let left_value = self.kb.read().unwrap().gensym("isa_value");
                    let call_id = self.new_call_id(&left_value);
                    let lookup = Goal::LookupExternal {
                        instance: left.clone(),
                        call_id,
                        field: right_value.clone_with_value(Value::Call(Predicate {
                            name: field.clone(),
                            args: vec![],
                        })),
                    };
                    let isa = Goal::Isa {
                        left: left.clone_with_value(Value::Variable(left_value)),
                        right: right_value.clone(),
                    };
                    self.append_goals(vec![lookup, isa])?;
                }
            }

            (Value::Variable(symbol), _) => {
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

            (_, Value::Variable(symbol)) => {
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

            (
                Value::ExternalInstance(left_instance),
                Value::Pattern(Pattern::Instance(right_literal)),
            ) => {
                // Check fields
                self.push_goal(Goal::Isa {
                    left: left.clone_with_value(Value::ExternalInstance(left_instance.clone())),
                    right: right.clone_with_value(Value::Pattern(Pattern::Dictionary(
                        right_literal.fields.clone(),
                    ))),
                })?;
                // Check class
                self.push_goal(Goal::IsaExternal {
                    instance_id: left_instance.instance_id,
                    literal: right_literal.clone(),
                })?;
            }

            _ => self.push_goal(Goal::Unify {
                left: left.clone(),
                right: right.clone(),
            })?,
        }
        Ok(())
    }

    pub fn lookup(&mut self, dict: &Dictionary, field: &Term, value: &Term) -> PolarResult<()> {
        // check if field is a variable
        match &field.value() {
            Value::Variable(_) => {
                let mut alternatives = vec![];
                for (k, v) in &dict.fields {
                    let mut goals: Goals = vec![];
                    // attempt to unify dict key with field
                    // if `field` is bound, unification will only succeed for the matching key
                    // if `field` is unbound, unification will succeed for all keys
                    goals.push(Goal::Unify {
                        left: field.clone_with_value(Value::String(k.clone().0)),
                        right: field.clone(),
                    });
                    // attempt to unify dict value with result
                    goals.push(Goal::Unify {
                        left: v.clone(),
                        right: value.clone(),
                    });
                    alternatives.push(goals);
                }
                self.choose(alternatives)?;
            }
            _ => {
                if let Some(retrieved) = dict.fields.get(&field_name(&field)) {
                    self.push_goal(Goal::Unify {
                        left: retrieved.clone(),
                        right: value.clone(),
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
    pub fn lookup_external(
        &mut self,
        call_id: u64,
        instance: &Term,
        field: &Term,
    ) -> PolarResult<QueryEvent> {
        let (field_name, args) = match &field.value() {
            Value::Call(Predicate { name, args }) => (
                name.clone(),
                args.iter().map(|arg| self.deref(arg)).collect(),
            ),
            _ => unreachable!("call must be a predicate"),
        };
        self.push_choice(vec![vec![Goal::LookupExternal {
            call_id,
            instance: instance.clone(),
            field: field.clone(),
        }]]);

        Ok(QueryEvent::ExternalCall {
            call_id,
            instance: Some(instance.clone()),
            attribute: field_name,
            args,
        })
    }

    pub fn isa_external(
        &mut self,
        instance_id: u64,
        literal: &InstanceLiteral,
    ) -> PolarResult<QueryEvent> {
        let result = self.kb.read().unwrap().gensym("isa");
        let call_id = self.new_call_id(&result);

        self.push_goal(Goal::Unify {
            left: Term::new_temporary(Value::Variable(result)),
            right: Term::new_temporary(Value::Boolean(true)),
        })?;

        Ok(QueryEvent::ExternalIsa {
            call_id,
            instance_id,
            class_tag: literal.tag.clone(),
        })
    }

    pub fn unify_external(
        &mut self,
        left_instance_id: u64,
        right_instance_id: u64,
    ) -> PolarResult<QueryEvent> {
        let result = self.kb.read().unwrap().gensym("unify");
        let call_id = self.new_call_id(&result);

        self.push_goal(Goal::Unify {
            left: Term::new_temporary(Value::Variable(result)),
            right: Term::new_temporary(Value::Boolean(true)),
        })?;

        Ok(QueryEvent::ExternalUnify {
            call_id,
            left_instance_id,
            right_instance_id,
        })
    }

    pub fn make_external(&self, literal: &InstanceLiteral, instance_id: u64) -> QueryEvent {
        QueryEvent::MakeExternal {
            instance_id,
            instance: literal.clone(),
        }
    }

    /// Query for the provided term.
    ///
    /// Uses the knowledge base to get an ordered list of rules.
    /// Creates a choice point over each rule, where each alternative
    /// consists of unifying the rule head with the arguments, then
    /// querying for each body clause.
    fn query(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        self.queries.push(term.clone());
        self.push_goal(Goal::PopQuery { term: term.clone() })?;
        self.trace.push(Rc::new(Trace {
            node: Node::Term(term.clone()),
            children: vec![],
        }));

        match &term.value() {
            Value::Call(predicate) => {
                self.query_for_predicate(predicate.clone())?;
            }
            Value::Expression(Operation { operator, args }) => {
                return self.query_for_operation(&term, *operator, args.clone());
            }
            _ => {
                let term = self.deref(term);
                self.query_for_value(&term)?;
            }
        }
        Ok(QueryEvent::None)
    }

    /// Select applicable rules for predicate.
    /// Sort applicable rules by specificity.
    /// Create a choice over the applicable rules.
    fn query_for_predicate(&mut self, predicate: Predicate) -> PolarResult<()> {
        let generic_rule = {
            let kb = self.kb.read().unwrap();
            kb.rules.get(&predicate.name).cloned()
        };
        match generic_rule {
            None => self.push_goal(Goal::Backtrack)?,
            Some(generic_rule) => {
                assert_eq!(generic_rule.name, predicate.name);
                self.append_goals(vec![
                    Goal::TracePush,
                    Goal::FilterRules {
                        applicable_rules: vec![],
                        unfiltered_rules: generic_rule.rules,
                        args: predicate.args,
                    },
                    Goal::TracePop,
                ])?;
            }
        }

        Ok(())
    }

    fn query_for_operation(
        &mut self,
        term: &Term,
        operator: Operator,
        mut args: Vec<Term>,
    ) -> PolarResult<QueryEvent> {
        match operator {
            Operator::And => {
                // Append a `Query` goal for each term in the args list
                self.push_goal(Goal::TracePop)?;
                self.append_goals(args.into_iter().map(|term| Goal::Query { term }))?;
                self.push_goal(Goal::TracePush)?;
            }
            Operator::Or => {
                // Create a choice point with alternatives to query for each arg, and start on the first alternative
                self.choose(args.into_iter().map(|term| vec![Goal::Query { term }]))?;
            }
            Operator::Not => {
                // Push a choice point that queries for the term; if the query succeeds cut and backtrack
                assert_eq!(args.len(), 1);
                let term = args.pop().unwrap();
                let alternatives = vec![
                    vec![
                        Goal::Query { term },
                        Goal::Cut {
                            choice_index: self.choices.len(),
                        },
                        Goal::Backtrack,
                    ],
                    vec![Goal::Noop],
                ];
                self.choose(alternatives)?;
            }
            Operator::Unify => {
                // Push a `Unify` goal
                assert_eq!(args.len(), 2);
                let right = args.pop().unwrap();
                let left = args.pop().unwrap();
                self.push_goal(Goal::Unify { left, right })?
            }
            Operator::Dot => self.dot_op_helper(args)?,
            op @ Operator::Lt
            | op @ Operator::Gt
            | op @ Operator::Leq
            | op @ Operator::Geq
            | op @ Operator::Eq
            | op @ Operator::Neq => {
                return self.comparison_op_helper(term, op, args);
            }
            Operator::In => {
                assert_eq!(args.len(), 2);
                let item = &args[0];
                let list = self.deref(&args[1]);
                let mut alternatives = vec![];
                if let Value::List(list) = list.value() {
                    for term in list {
                        alternatives.push(vec![Goal::Unify {
                            left: item.clone(),
                            right: term.clone(),
                        }])
                    }
                } else {
                    return Err(self.type_error(
                        item,
                        format!("can only use `in` on a list, this is {:?}", item.value()),
                    ));
                }
                self.choose(alternatives)?;
            }
            Operator::Debug => {
                let mut message = "Welcome to the debugger!".to_string();
                if !args.is_empty() {
                    message += &format!(
                        "\ndebug({})",
                        args.iter()
                            .map(|arg| self.deref(arg).to_polar())
                            .collect::<Vec<String>>()
                            .join(", ")
                    );
                }
                self.push_goal(Goal::Debug { message })?
            }
            Operator::New => {
                assert_eq!(args.len(), 2);
                let result = args.pop().unwrap();
                assert!(
                    matches!(result.value(), Value::Variable(_)),
                    "Must have result as second arg."
                );
                let mut literal_term = args.pop().unwrap();
                literal_term.map_replace(&mut |t| self.deref(t));
                let literal_value = literal_term
                    .value()
                    .clone()
                    .instance_literal()
                    .expect("Arg must be instance literal");

                let instance_id = self.new_id();
                literal_term.replace_value(Value::ExternalInstance(ExternalInstance {
                    instance_id,
                    literal: Some(literal_value.clone()),
                }));

                // A goal is used here in case the result is already bound to some external
                // instance.
                self.append_goals(vec![
                    Goal::Unify {
                        left: result,
                        right: literal_term,
                    },
                    Goal::MakeExternal {
                        instance_id,
                        literal: literal_value,
                    },
                ])?;
            }
            Operator::Cut => {
                // Remove all choices created before this cut that are in the
                // current rule body.
                let mut choice_index = self.choices.len();
                for choice in self.choices.iter().rev() {
                    // Comparison excludes the rule body & cut operator (the last two elements of self.queries)
                    let prefix = &self.queries[..(self.queries.len() - 2)];
                    if choice.queries.starts_with(prefix) {
                        // If the choice has the same query stack as the current
                        // query stack, remove it.
                        choice_index -= 1;
                    } else {
                        break;
                    }
                }

                self.push_goal(Goal::Cut { choice_index })?;
            }
            Operator::Isa => {
                assert_eq!(args.len(), 2);
                let right = args.pop().unwrap();
                let left = args.pop().unwrap();
                self.push_goal(Goal::Isa { left, right })?
            }
            Operator::ForAll => {
                assert_eq!(args.len(), 2);
                let action = args.pop().unwrap();
                let condition = args.pop().unwrap();
                // For all is implemented as !(condition, !action).
                let op = Operation {
                    operator: Operator::Not,
                    args: vec![term.clone_with_value(Value::Expression(Operation {
                        operator: Operator::And,
                        args: vec![
                            condition,
                            term.clone_with_value(Value::Expression(Operation {
                                operator: Operator::Not,
                                args: vec![action],
                            })),
                        ],
                    }))],
                };
                let double_negation = term.clone_with_value(Value::Expression(op));
                self.push_goal(Goal::Query {
                    term: double_negation,
                })?;
            }
            _ => {
                return Err(self.type_error(
                    &term,
                    format!("can't query for: {}", term.value().to_polar()),
                ));
            }
        }
        Ok(QueryEvent::None)
    }

    /// Query for a value.  Succeeds if the value is 'truthy' or backtracks.
    /// Currently only defined for boolean values.
    fn query_for_value(&mut self, term: &Term) -> PolarResult<()> {
        if let Value::Boolean(value) = term.value() {
            if !value {
                // Backtrack if the boolean is false.
                self.push_goal(Goal::Backtrack)?;
            }

            Ok(())
        } else {
            Err(self.type_error(&term, format!("can't query for: {}", term.value().to_polar())))
        }
    }

    /// Push appropriate goals for lookups on Dictionaries, InstanceLiterals, and ExternalInstances
    fn dot_op_helper(&mut self, mut args: Vec<Term>) -> PolarResult<()> {
        assert_eq!(args.len(), 3);
        let object = self.deref(&args[0]);
        let field = &args[1];
        let value = &args[2];

        match object.value() {
            // Push a `Lookup` goal for simple field lookups on dictionaries.
            Value::Dictionary(dict) if !matches!(field.value(), Value::Call(predicate) if !predicate.args.is_empty()) => {
                self.push_goal(Goal::Lookup {
                    dict: dict.clone(),
                    field: field.clone(),
                    value: args.remove(2),
                })?
            }
            // Push an `ExternalLookup` goal for external instances and built-ins.
            Value::Dictionary(_)
            | Value::ExternalInstance(_)
            | Value::List(_)
            | Value::Number(_)
            | Value::String(_) => {
                let value = value.value().clone().symbol().expect("bad lookup value");
                let call_id = self.new_call_id(&value);
                self.push_goal(Goal::LookupExternal {
                    call_id,
                    instance: object.clone(),
                    field: field.clone(),
                })?;
            }
            _ => {
                return Err(self.type_error(
                    &object,
                    format!(
                        "can only perform lookups on dicts and instances, this is {:?}",
                        object.value()
                    ),
                ))
            }
        }
        Ok(())
    }

    /// Evaluate numerical comparisons
    fn comparison_op_helper(
        &mut self,
        term: &Term,
        op: Operator,
        args: Vec<Term>,
    ) -> PolarResult<QueryEvent> {
        assert_eq!(args.len(), 2);
        let left_term = self.deref(&args[0]);
        let right_term = self.deref(&args[1]);

        match (left_term.value(), right_term.value()) {
            (Value::Number(left), Value::Number(right)) => {
                let result = match op {
                    Operator::Lt => left < right,
                    Operator::Leq => left <= right,
                    Operator::Gt => left > right,
                    Operator::Geq => left >= right,
                    Operator::Eq => left == right,
                    Operator::Neq => left != right,
                    _ => unreachable!(
                        "operator: {:?} should not be handled by this method, this is a bug",
                        op
                    ),
                };
                if !result {
                    self.push_goal(Goal::Backtrack)?;
                }
                Ok(QueryEvent::None)
            }
            (Value::ExternalInstance(_), Value::ExternalInstance(_)) => {
                // Generate symbol for external op result and bind to `false` (default)
                let answer = self.kb.read().unwrap().gensym("external_op_result");
                self.bind(&answer, Term::new_temporary(Value::Boolean(false)));

                // append unify goal to be evaluated after external op result is returned & bound
                self.append_goals(vec![Goal::Unify {
                    left: Term::new_temporary(Value::Variable(answer.clone())),
                    right: Term::new_temporary(Value::Boolean(true)),
                }])?;
                let call_id = self.new_call_id(&answer);
                Ok(QueryEvent::ExternalOp {
                    call_id,
                    operator: op,
                    args: vec![left_term, right_term],
                })
            }
            (left, right) => Err(self.type_error(
                term,
                format!(
                    "{} expects comparable arguments, got: {}, {}",
                    op.to_polar(),
                    left.to_polar(),
                    right.to_polar()
                ),
            )),
        }
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
                value,
            );
        } else {
            // No more results. Clean up, cut out the retry alternative,
            // and backtrack.
            self.call_id_symbols.remove(&call_id).expect("bad call ID");
            self.push_goal(Goal::Backtrack)?;
            self.push_goal(Goal::Cut {
                choice_index: self.choices.len() - 1,
            })?;
        }
        Ok(())
    }

    /// Handle an external response to ExternalIsSubSpecializer and ExternalIsa
    pub fn external_question_result(&mut self, call_id: u64, answer: bool) {
        let var = self.call_id_symbols.remove(&call_id).expect("bad call id");
        self.bind(&var, Term::new_temporary(Value::Boolean(answer)));
    }

    /// Unify `left` and `right` terms.
    ///
    /// Outcomes of a unification are:
    ///  - Successful unification => bind zero or more variables to values
    ///  - Recursive unification => more `Unify` goals are pushed onto the stack
    ///  - Failure => backtrack
    fn unify(&mut self, left: &Term, right: &Term) -> PolarResult<()> {
        match (&left.value(), &right.value()) {
            // Unify variables.
            (Value::Variable(var), _) => self.unify_var(var, right)?,
            (_, Value::Variable(var)) => self.unify_var(var, left)?,

            // Unify rest-variables with list tails.
            (Value::RestVariable(var), Value::List(_)) => self.unify_var(var, right)?,
            (Value::List(_), Value::RestVariable(var)) => self.unify_var(var, left)?,

            // Unify lists by recursively unifying their elements.
            (Value::List(left), Value::List(right)) => {
                self.unify_lists(left, right, |(left, right)| Goal::Unify {
                    left: left.clone(),
                    right: right.clone(),
                })?
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
            (Value::Number(left), Value::Number(right)) => {
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
                    self.append_goals(left.args.iter().zip(right.args.iter()).map(
                        |(left, right)| Goal::Unify {
                            left: left.clone(),
                            right: right.clone(),
                        },
                    ))?;
                } else {
                    self.push_goal(Goal::Backtrack)?
                }
            }

            // TODO(gj): Is this case necessary to handle at all? What is an example rule that
            // would lead to this? When would you need to unify an instance with itself?
            //
            // External instances can unify if they are the same instance, i.e., have the same
            // instance ID. This is necessary for the case where an instance appears multiple times
            // in the same rule head. For example, `f(foo, foo) := ...` or `isa(x, y, x: y) := ...`
            // or `max(x, y, x) := x > y;`.
            (
                Value::ExternalInstance(ExternalInstance {
                    instance_id: left_instance,
                    ..
                }),
                Value::ExternalInstance(ExternalInstance {
                    instance_id: right_instance,
                    ..
                }),
            ) if left_instance != right_instance => {
                self.push_goal(Goal::UnifyExternal {
                    left_instance_id: *left_instance,
                    right_instance_id: *right_instance,
                })?;
            }

            (Value::InstanceLiteral(_), Value::InstanceLiteral(_)) => {
                return Err(
                    self.type_error(&left, String::from("Cannot unify two instance literals."))
                );
            }

            (Value::InstanceLiteral(_), Value::ExternalInstance(_))
            | (Value::ExternalInstance(_), Value::InstanceLiteral(_)) => {
                return Err(self.type_error(
                    &left,
                    String::from("Cannot unify instance literal with external instance."),
                ));
            }

            // Anything else fails.
            (_, _) => self.push_goal(Goal::Backtrack)?,
        }

        Ok(())
    }

    /// Unify a symbol `left` with a term `right`.
    /// This is sort of a "sub-goal" of `Unify`.
    fn unify_var(&mut self, left: &Symbol, right: &Term) -> PolarResult<()> {
        let left_value = self.value(&left).cloned();
        let mut right_value = None;
        if let Value::Variable(ref right_sym) | Value::RestVariable(ref right_sym) = right.value() {
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
                self.bind(left, value);
            }
            (None, None) => {
                // Neither is bound, so bind them together.
                // TODO: should theoretically bind the earliest one here?
                self.bind(left, right.clone());
            }
        }
        Ok(())
    }

    /// "Unify" two lists element-wise, respecting rest-variables.
    /// Used by both `unify` and `isa`; hence the third argument,
    /// a closure that builds sub-goals.
    #[allow(clippy::ptr_arg)]
    fn unify_lists<F>(&mut self, left: &TermList, right: &TermList, unify: F) -> PolarResult<()>
    where
        F: FnMut((&Term, &Term)) -> Goal,
    {
        if has_rest_var(left) {
            self.unify_lists_with_rest(left, right, unify)
        } else if has_rest_var(right) {
            self.unify_lists_with_rest(right, left, unify)
        } else if left.len() == right.len() {
            // No rest-variables; unify element-wise.
            self.append_goals(left.iter().zip(right).map(unify))
        } else {
            self.push_goal(Goal::Backtrack)
        }
    }

    /// Unify a list that ends with a rest-variable with another.
    /// We assume that the left list has the rest-variable.
    /// A helper method for `unify_lists`.
    #[allow(clippy::ptr_arg)]
    fn unify_lists_with_rest<F>(
        &mut self,
        left: &TermList,
        right: &TermList,
        mut unify: F,
    ) -> PolarResult<()>
    where
        F: FnMut((&Term, &Term)) -> Goal,
    {
        assert!(has_rest_var(left));
        let n = left.len() - 1;
        if right.len() >= n {
            let rest = unify((
                &left[n].clone(),
                &Term::new_temporary(Value::List(right[n..].to_vec())),
            ));
            self.append_goals(left.iter().take(n).zip(right).map(unify).chain(vec![rest]))
        } else {
            self.push_goal(Goal::Backtrack)
        }
    }

    /// Filter rules to just those applicable to a list of arguments,
    /// then sort them by specificity.
    #[allow(clippy::ptr_arg)]
    fn filter_rules(
        &mut self,
        applicable_rules: &Rules,
        unfiltered_rules: &Rules,
        args: &TermList,
    ) -> PolarResult<()> {
        if unfiltered_rules.is_empty() {
            // The rules have been filtered. Sort them.
            self.push_goal(Goal::SortRules {
                rules: applicable_rules.iter().cloned().rev().collect(),
                args: args.clone(),
                outer: 1,
                inner: 1,
            })
        } else {
            // Check one rule for applicability.
            let mut unfiltered_rules = unfiltered_rules.clone();
            let rule = unfiltered_rules.pop().unwrap();
            let inapplicable = Goal::FilterRules {
                args: args.clone(),
                applicable_rules: applicable_rules.clone(),
                unfiltered_rules: unfiltered_rules.clone(),
            };
            if rule.params.len() != args.len() {
                return self.push_goal(inapplicable); // wrong arity
            }

            let mut applicable_rules = applicable_rules.clone();
            applicable_rules.push(rule.clone());
            let applicable = Goal::FilterRules {
                args: args.clone(),
                applicable_rules,
                unfiltered_rules,
            };

            // Try to unify the arguments with renamed parameters.
            // TODO: Think about using backtrack so that we don't
            // leave temporary bindings around.
            let args = self.rename_vars(args.clone());
            let Rule { params, .. } = self.rename_rule_vars(&rule);
            let mut check_applicability = vec![];
            for (arg, param) in args.iter().zip(params.iter()) {
                if let Some(parameter) = &param.parameter {
                    check_applicability.push(Goal::Unify {
                        left: arg.clone(),
                        right: parameter.clone(),
                    });
                }
                if let Some(specializer) = &param.specializer {
                    check_applicability.push(Goal::Isa {
                        left: arg.clone(),
                        right: specializer.clone(),
                    });
                }
            }
            check_applicability.push(Goal::Cut {
                choice_index: self.choices.len(),
            });
            check_applicability.push(applicable);
            self.choose(vec![check_applicability, vec![inapplicable]])?;
            Ok(())
        }
    }

    /// Sort a list of rules with respect to a list of arguments
    /// using an explicit-state insertion sort.
    ///
    /// We maintain two indices for the sort, `outer` and `inner`. The `outer` index tracks our
    /// sorting progress. Every rule at or below `outer` is sorted; every rule above it is
    /// unsorted. The `inner` index tracks our search through the sorted sublist for the correct
    /// position of the candidate rule (the rule at the head of the unsorted portion of the
    /// list).
    #[allow(clippy::ptr_arg)]
    fn sort_rules(
        &mut self,
        rules: &Rules,
        args: &TermList,
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

                let mut rules = rules.clone();
                rules.swap(inner - 1, inner);
                let next_inner = Goal::SortRules {
                    rules,
                    outer,
                    inner: inner - 1,
                    args: args.clone(),
                };
                // If the comparison fails, break out of the inner loop.
                // If the comparison succeeds, continue the inner loop with the swapped rules.
                self.choose(vec![
                    vec![
                        compare,
                        Goal::Cut {
                            choice_index: self.choices.len(),
                        },
                        next_inner,
                    ],
                    vec![next_outer],
                ])?;
            } else {
                assert_eq!(inner, 0);
                self.push_goal(next_outer)?;
            }
        } else {
            // We're done; the rules are sorted.
            // Make alternatives for calling them.
            let mut alternatives = vec![];
            for rule in rules.iter() {
                let mut goals = vec![];
                goals.push(Goal::TraceRule {
                    trace: Rc::new(Trace {
                        node: Node::Rule(rule.clone()),
                        children: vec![],
                    }),
                });
                goals.push(Goal::TracePush);
                let Rule { body, params, .. } = self.rename_rule_vars(rule);

                // Unify the arguments with the formal parameters.
                for (arg, param) in args.iter().zip(params.iter()) {
                    if let Some(right) = &param.parameter {
                        goals.push(Goal::Unify {
                            left: arg.clone(),
                            right: right.clone(),
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
                goals.push(Goal::TracePop);

                alternatives.push(goals)
            }

            // Choose the first alternative, and push a choice for the rest.
            self.choose(alternatives)?;
        }
        Ok(())
    }

    /// Succeed if `left` is more specific than `right` with respect to `args`.
    #[allow(clippy::ptr_arg)]
    fn is_more_specific(&mut self, left: &Rule, right: &Rule, args: &TermList) -> PolarResult<()> {
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
                    let answer = self.kb.read().unwrap().gensym("is_subspecializer");
                    // Bind answer to false as a starting point in case is subspecializer doesn't
                    // bind any result.
                    // This is done here for safety to avoid a bug where `answer` is unbound by
                    // `IsSubspecializer` and the `Unify` Goal just assigns it to `true` instead
                    // of checking that is is equal to `true`.
                    self.bind(&answer, Term::new_temporary(Value::Boolean(false)));

                    return self.append_goals(vec![
                        Goal::IsSubspecializer {
                            answer: answer.clone(),
                            left: left_spec.clone(),
                            right: right_spec.clone(),
                            arg: arg.clone(),
                        },
                        Goal::Unify {
                            left: Term::new_temporary(Value::Variable(answer)),
                            right: Term::new_temporary(Value::Boolean(true)),
                        },
                    ]);
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
        answer: &Symbol,
        left: &Term,
        right: &Term,
        arg: &Term,
    ) -> PolarResult<QueryEvent> {
        assert!(!matches!(left.value(), Value::InstanceLiteral(_)));
        assert!(!matches!(right.value(), Value::InstanceLiteral(_)));

        let arg = self.deref(&arg);
        match (arg.value(), left.value(), right.value()) {
            (
                Value::ExternalInstance(instance),
                Value::Pattern(Pattern::Instance(left_lit)),
                Value::Pattern(Pattern::Instance(right_lit)),
            ) => {
                let call_id = self.new_call_id(&answer);
                let instance_id = instance.instance_id;
                if left_lit.tag == right_lit.tag
                    && !(left_lit.fields.fields.is_empty() && right_lit.fields.fields.is_empty())
                {
                    self.push_goal(Goal::IsSubspecializer {
                        answer: answer.clone(),
                        left: left.clone_with_value(Value::Pattern(Pattern::Dictionary(
                            left_lit.fields.clone(),
                        ))),
                        right: right.clone_with_value(Value::Pattern(Pattern::Dictionary(
                            right_lit.fields.clone(),
                        ))),
                        arg,
                    })?;
                }
                // check ordering based on the classes
                Ok(QueryEvent::ExternalIsSubSpecializer {
                    call_id,
                    instance_id,
                    left_class_tag: left_lit.tag.clone(),
                    right_class_tag: right_lit.tag.clone(),
                })
            }
            (
                _,
                Value::Pattern(Pattern::Dictionary(left)),
                Value::Pattern(Pattern::Dictionary(right)),
            ) => {
                let left_fields: HashSet<&Symbol> = left.fields.keys().collect();
                let right_fields: HashSet<&Symbol> = right.fields.keys().collect();

                // The dictionary with more fields is taken as more specific.
                // The assumption here is that rules have already been filtered
                // for applicability.
                if left_fields.len() != right_fields.len() {
                    self.bind(
                        &answer,
                        Term::new_temporary(Value::Boolean(right_fields.len() < left.fields.len())),
                    );
                }
                Ok(QueryEvent::None)
            }
            (_, Value::Pattern(Pattern::Instance(_)), Value::Pattern(Pattern::Dictionary(_))) => {
                self.bind(&answer, Term::new_temporary(Value::Boolean(true)));
                Ok(QueryEvent::None)
            }
            _ => {
                self.bind(&answer, Term::new_temporary(Value::Boolean(false)));
                Ok(QueryEvent::None)
            }
        }
    }

    fn type_error(&self, term: &Term, msg: String) -> error::PolarError {
        let source = { self.kb.read().unwrap().sources.get_source(&term) };
        let context = if let Some(source) = source {
            make_context(&source, term.offset())
        } else {
            None
        };
        let stack_trace = self.stack_trace();
        let error = error::RuntimeError::TypeError {
            msg,
            loc: term.offset(),
            context,
            stack_trace: Some(stack_trace),
        };
        error.into()
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
            assert!(matches!($vm.run().unwrap(), QueryEvent::Result{bindings, ..} if bindings == $result));
            assert_query_events!($vm, []);
        };
        ($vm:ident, [QueryEvent::Result{$result:expr}, $($tail:tt)*]) => {
            assert!(matches!($vm.run().unwrap(), QueryEvent::Result{bindings, ..} if bindings == $result));
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
        vm.bind(&x, term_y.clone());
        assert_eq!(vm.deref(&term_x), term_y);

        // value
        assert_eq!(vm.deref(&value), value.clone());

        // unbound var -> value
        vm.bind(&x, value.clone());
        assert_eq!(vm.deref(&term_x), value);

        // unbound var -> unbound var -> value
        vm.bind(&x, term_y);
        vm.bind(&y, value.clone());
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

        let mut vm = PolarVirtualMachine::new(Arc::new(RwLock::new(kb)), vec![goal]);
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
                term: Term::new_from_test(Value::Expression(Operation {
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
        let one = term!(1);
        let one_list = term!([1]);
        let one_two_list = term!([1, 2]);
        let two_one_list = term!([2, 1]);
        let empty_list = term!([]);

        // [] isa []
        vm.push_goal(Goal::Isa {
            left: empty_list.clone(),
            right: empty_list.clone(),
        })
        .unwrap();
        assert!(
            matches!(vm.run().unwrap(), QueryEvent::Result{bindings, ..} if bindings.is_empty())
        );
        assert!(matches!(vm.run().unwrap(), QueryEvent::Done));
        assert!(vm.is_halted());

        // [1,2] isa [1,2]
        vm.push_goal(Goal::Isa {
            left: one_two_list.clone(),
            right: one_two_list.clone(),
        })
        .unwrap();
        assert!(
            matches!(vm.run().unwrap(), QueryEvent::Result{bindings, ..} if bindings.is_empty())
        );
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
            left: one_two_list.clone(),
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

        // [1,2] isa [1, *rest]
        vm.push_goal(Goal::Isa {
            left: one_two_list,
            right: term!([1, Value::RestVariable(sym!("rest"))]),
        })
        .unwrap();
        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{sym!("rest") => term!([2])}},
            QueryEvent::Done
        ]);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn isa_on_dicts() {
        let mut vm = PolarVirtualMachine::default();
        let left = term!(btreemap! {
            sym!("x") => term!(1),
            sym!("y") => term!(2),
        });
        let right = Pattern::term_as_pattern(&term!(btreemap! {
            sym!("x") => term!(1),
            sym!("y") => term!(2),
        }));
        vm.push_goal(Goal::Isa {
            left: left.clone(),
            right,
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done]);

        // Dicts with identical keys and different values DO NOT isa.
        let right = Pattern::term_as_pattern(&term!(btreemap! {
            sym!("x") => term!(2),
            sym!("y") => term!(1),
        }));
        vm.push_goal(Goal::Isa {
            left: left.clone(),
            right,
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Done]);

        // {} isa {}.
        vm.push_goal(Goal::Isa {
            left: term!(btreemap! {}),
            right: Pattern::term_as_pattern(&term!(btreemap! {})),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done]);

        // Non-empty dicts should isa against an empty dict.
        vm.push_goal(Goal::Isa {
            left: left.clone(),
            right: Pattern::term_as_pattern(&term!(btreemap! {})),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done]);

        // Empty dicts should NOT isa against a non-empty dict.
        vm.push_goal(Goal::Isa {
            left: term!(btreemap! {}),
            right: Pattern::term_as_pattern(&left),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Done]);

        // Superset dict isa subset dict.
        vm.push_goal(Goal::Isa {
            left: left.clone(),
            right: Pattern::term_as_pattern(&term!(btreemap! {sym!("x") => term!(1)})),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done]);

        // Subset dict isNOTa superset dict.
        vm.push_goal(Goal::Isa {
            left: term!(btreemap! {sym!("x") => term!(1)}),
            right: Pattern::term_as_pattern(&left),
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
        vm.bind(&x, zero.clone());
        assert_eq!(vm.value(&x), Some(&zero));
        assert_eq!(vm.value(&y), None);
    }

    #[test]
    fn debug() {
        let mut vm = PolarVirtualMachine::new(
            Arc::new(RwLock::new(KnowledgeBase::new())),
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
        let mut vm = PolarVirtualMachine::new(
            Arc::new(RwLock::new(KnowledgeBase::new())),
            vec![Goal::Halt],
        );
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
            Arc::new(RwLock::new(KnowledgeBase::new())),
            vec![Goal::Unify {
                left: vars,
                right: vals,
            }],
        );
        let _ = vm.run().unwrap();
        assert_eq!(vm.value(&x), Some(&Term::new_from_test(zero)));
        assert_eq!(vm.value(&y), Some(&Term::new_from_test(one)));
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
        vm.bind(&y, one.clone());
        vm.append_goals(vec![Goal::Unify {
            left: term!(x),
            right: term!(y),
        }])
        .unwrap();
        let _ = vm.run().unwrap();
        assert_eq!(vm.value(&sym!("x")), Some(&one));
        vm.backtrack().unwrap();

        // Left variable bound to value.
        vm.bind(&z, one.clone());
        vm.append_goals(vec![Goal::Unify {
            left: term!(z.clone()),
            right: one.clone(),
        }])
        .unwrap();
        let _ = vm.run().unwrap();
        assert_eq!(vm.value(&z), Some(&one));

        // Left variable bound to value
        vm.bind(&z, one.clone());
        vm.append_goals(vec![Goal::Unify {
            left: term!(z.clone()),
            right: two,
        }])
        .unwrap();
        let _ = vm.run().unwrap();
        assert_eq!(vm.value(&z), Some(&one));
    }

    #[test]
    fn test_gen_var() {
        let vm = PolarVirtualMachine::default();

        let rule = Rule {
            name: Symbol::new("foo"),
            params: vec![],
            body: Term::new_from_test(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![
                    term!(1),
                    Term::new_from_test(Value::Variable(Symbol("x".to_string()))),
                    Term::new_from_test(Value::Variable(Symbol("x".to_string()))),
                    Term::new_from_test(Value::List(vec![Term::new_from_test(Value::Variable(
                        Symbol("y".to_string()),
                    ))])),
                ],
            })),
        };

        let renamed_rule = vm.rename_rule_vars(&rule);
        let renamed_terms = unwrap_and(renamed_rule.body);
        assert_eq!(renamed_terms[1].value(), renamed_terms[2].value());
        let x_value = match &renamed_terms[1].value() {
            Value::Variable(sym) => Some(sym.0.clone()),
            _ => None,
        };
        assert_eq!(x_value.unwrap(), "_x_0");

        let y_value = match &renamed_terms[3].value() {
            Value::List(terms) => match &terms[0].value() {
                Value::Variable(sym) => Some(sym.0.clone()),
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
                rule!("bar", ["_"; instance!("b"), "__"; instance!("a"), value!(3)]),
                rule!("bar", ["_"; instance!("a"), "__"; instance!("a"), value!(1)]),
                rule!("bar", ["_"; instance!("a"), "__"; instance!("b"), value!(2)]),
                rule!("bar", ["_"; instance!("b"), "__"; instance!("b"), value!(4)]),
            ],
        );

        let mut kb = KnowledgeBase::new();
        kb.add_generic_rule(bar_rule);

        let external_instance = Value::ExternalInstance(ExternalInstance {
            literal: None,
            instance_id: 1,
        });

        let mut vm = PolarVirtualMachine::new(
            Arc::new(RwLock::new(kb)),
            vec![query!(pred!(
                "bar",
                [external_instance.clone(), external_instance, sym!("z")]
            ))],
        );

        let mut results = Vec::new();
        loop {
            match vm.run().unwrap() {
                QueryEvent::Done => break,
                QueryEvent::Result { bindings, .. } => results.push(bindings),
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
                hashmap! {sym!("z") => term!(1)},
                hashmap! {sym!("z") => term!(2)},
                hashmap! {sym!("z") => term!(3)},
                hashmap! {sym!("z") => term!(4)},
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
        let left = term!(value!(Pattern::Instance(InstanceLiteral {
            tag: sym!("Any"),
            fields: Dictionary {
                fields: btreemap! {}
            }
        })));
        let right = term!(Value::Pattern(Pattern::Dictionary(Dictionary {
            fields: btreemap! {sym!("a") => term!("a")},
        })));

        let answer = vm.kb.read().unwrap().gensym("is_subspecializer");

        match vm.is_subspecializer(&answer, &left, &right, &arg).unwrap() {
            QueryEvent::None => (),
            event => panic!("Expected None, got {:?}", event),
        }

        assert_eq!(
            vm.deref(&term!(Value::Variable(answer))),
            term!(value!(true))
        );
    }
}
