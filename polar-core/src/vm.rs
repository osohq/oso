use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::rc::Rc;
use std::string::ToString;
use std::sync::{Arc, RwLock, RwLockReadGuard};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::bindings::{
    Binding, BindingManager, BindingStack, Bindings, Bsp, FollowerId, VariableState,
};
use crate::counter::Counter;
use crate::data_filtering::partition_equivs;
use crate::debugger::{get_binding_for_var, DebugEvent, Debugger};
use crate::error::{invalid_state, unsupported, PolarError, PolarResult, RuntimeError};
use crate::events::*;
use crate::folder::Folder;
use crate::inverter::Inverter;
use crate::kb::*;
use crate::messages::*;
use crate::numerics::*;
use crate::partial::{simplify_bindings_opt, simplify_partial, sub_this, IsaConstraintCheck};
use crate::rewrites::Renamer;
use crate::rules::*;
use crate::runnable::Runnable;
use crate::sources::Context;
use crate::terms::*;
use crate::traces::*;
use crate::visitor::{walk_term, Visitor};

pub const MAX_STACK_SIZE: usize = 10_000;
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
}

impl LogLevel {
    fn should_print_on_level(&self, level: LogLevel) -> bool {
        *self <= level
    }
}

#[derive(Debug, Clone)]
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
    Error {
        error: PolarError,
    },
    Halt,
    Isa {
        left: Term,
        right: Term,
    },
    IsMoreSpecific {
        left: Arc<Rule>,
        right: Arc<Rule>,
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
    IsaExternal {
        instance: Term,
        literal: InstanceLiteral,
    },
    MakeExternal {
        constructor: Term,
        instance_id: u64,
    },
    NextExternal {
        call_id: u64,
        iterable: Term,
    },
    CheckError,
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
    TraceStackPush,
    TraceStackPop,
    Unify {
        left: Term,
        right: Term,
    },

    /// Run the `runnable`.
    Run {
        runnable: Box<dyn Runnable>,
    },

    /// Add a new constraint
    AddConstraint {
        term: Term,
    },

    /// TODO hack.
    /// Add a new constraint
    AddConstraintsBatch {
        add_constraints: Rc<RefCell<Bindings>>,
    },
}

#[derive(Clone, Debug)]
pub struct Choice {
    pub alternatives: Vec<GoalStack>,
    bsp: Bsp,              // binding stack pointer
    pub goals: GoalStack,  // goal stack snapshot
    queries: Queries,      // query stack snapshot
    trace: Vec<Rc<Trace>>, // trace snapshot
    trace_stack: TraceStack,
}

pub type Choices = Vec<Choice>;
/// Shortcut type alias for a list of goals
pub type Goals = Vec<Goal>;
pub type TraceStack = Vec<Rc<Vec<Rc<Trace>>>>;

#[derive(Clone, Debug, Default)]
pub struct GoalStack(Vec<Rc<Goal>>);

impl GoalStack {
    fn new_reversed(goals: Goals) -> Self {
        Self(goals.into_iter().rev().map(Rc::new).collect())
    }
}

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

pub fn compare(
    op: Operator,
    left: &Term,
    right: &Term,
    context: Option<&Term>,
) -> PolarResult<bool> {
    use {Operator::*, Value::*};
    // Coerce booleans to integers.
    // FIXME(gw) why??
    fn to_int(x: bool) -> Numeric {
        Numeric::Integer(i64::from(x))
    }

    fn compare<T: PartialOrd>(op: Operator, left: T, right: T) -> PolarResult<bool> {
        match op {
            Lt => Ok(left < right),
            Leq => Ok(left <= right),
            Gt => Ok(left > right),
            Geq => Ok(left >= right),
            Eq => Ok(left == right),
            Neq => Ok(left != right),
            _ => invalid_state(format!("`{}` is not a comparison operator", op)),
        }
    }

    match (left.value(), right.value()) {
        (Boolean(l), Boolean(r)) => compare(op, &to_int(*l), &to_int(*r)),
        (Boolean(l), Number(r)) => compare(op, &to_int(*l), r),
        (Number(l), Boolean(r)) => compare(op, l, &to_int(*r)),
        (Number(l), Number(r)) => compare(op, l, r),
        (String(l), String(r)) => compare(op, l, r),
        _ => {
            let context = context.expect("should only be None in Grounder, where we unwrap anyway");
            unsupported(context.to_string(), context)
        }
    }
}

#[derive(Clone)]
pub struct PolarVirtualMachine {
    /// Stacks.
    pub goals: GoalStack,
    binding_manager: BindingManager,
    choices: Choices,
    pub queries: Queries,

    pub tracing: bool,
    pub trace_stack: TraceStack, // Stack of traces higher up the tree.
    pub trace: Vec<Rc<Trace>>,   // Traces for the current level of the trace tree.

    // Errors from outside the vm.
    pub external_error: Option<String>,

    #[cfg(not(target_arch = "wasm32"))]
    query_start_time: Option<std::time::Instant>,
    #[cfg(target_arch = "wasm32")]
    query_start_time: Option<f64>,
    query_timeout_ms: u64,

    /// Maximum size of goal stack
    stack_limit: usize,

    /// Binding stack constant below here.
    csp: Bsp,

    /// Interactive debugger.
    pub debugger: Debugger,

    /// Rules and types.
    pub kb: Arc<RwLock<KnowledgeBase>>,

    /// Call ID -> result variable name table.
    call_id_symbols: HashMap<u64, Symbol>,

    /// Logging flag.
    log_level: Option<LogLevel>,

    polar_log_stderr: bool,
    polar_trace_mute: bool,

    // Other flags.
    pub query_contains_partial: bool,
    pub inverting: bool,

    /// Output messages.
    pub messages: MessageQueue,
}

impl Default for PolarVirtualMachine {
    fn default() -> Self {
        PolarVirtualMachine::new(
            Arc::new(RwLock::new(KnowledgeBase::default())),
            false,
            vec![],
            // Messages will not be exposed, only use default() for testing.
            MessageQueue::new(),
        )
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(a: &str);
}

// Methods which aren't goals/instructions.
impl PolarVirtualMachine {
    /// Make a new virtual machine with an initial list of goals.
    /// Reverse the goal list for the sanity of callers.
    pub fn new(
        kb: Arc<RwLock<KnowledgeBase>>,
        tracing: bool,
        goals: Goals,
        messages: MessageQueue,
    ) -> Self {
        let query_timeout_ms = std::env::var("POLAR_TIMEOUT_MS")
            .ok()
            .and_then(|timeout_str| timeout_str.parse::<u64>().ok())
            .unwrap_or(DEFAULT_TIMEOUT_MS);
        let constants = kb
            .read()
            .expect("cannot acquire KB read lock")
            .get_registered_constants()
            .clone();

        let mut vm = Self {
            goals: GoalStack::new_reversed(goals),
            binding_manager: BindingManager::new(),
            query_start_time: None,
            query_timeout_ms,
            stack_limit: MAX_STACK_SIZE,
            csp: Bsp::default(),
            choices: vec![],
            queries: vec![],
            tracing,
            trace_stack: vec![],
            trace: vec![],
            external_error: None,
            debugger: Debugger::default(),
            kb,
            call_id_symbols: HashMap::new(),
            // `log` controls internal VM logging
            log_level: None,
            // `polar_log_stderr` prints things immediately to stderr
            polar_log_stderr: false,
            polar_trace_mute: false,
            query_contains_partial: false,
            inverting: false,
            messages,
        };
        vm.bind_constants(constants);
        vm.query_contains_partial();

        let polar_log = std::env::var("POLAR_LOG");
        vm.set_logging_options(None, polar_log.ok());

        vm
    }

    pub fn set_logging_options(&mut self, rust_log: Option<String>, polar_log: Option<String>) {
        let polar_log = polar_log.unwrap_or_default();
        let polar_log_vars: HashSet<String> = polar_log
            .split(',')
            .filter(|v| !v.is_empty())
            .map(|s| s.to_lowercase())
            .collect();

        self.polar_log_stderr = polar_log_vars.contains(&"now".to_string());

        // TODO: @patrickod remove `RUST_LOG` from node lib & drop this option.
        self.log_level = if rust_log.is_some() {
            Some(LogLevel::Trace)
        } else {
            None
        };

        // The values `off` and `0` mute all logging and take precedence over any other coexisting value.
        // If POLAR_LOG is specified we attempt to match the level requested, other default to INFO
        if !polar_log_vars.is_empty()
            && polar_log_vars.is_disjoint(&HashSet::from(["off".to_string(), "0".to_string()]))
        {
            self.log_level = if polar_log_vars.contains(&LogLevel::Trace.to_string()) {
                Some(LogLevel::Trace)
            } else if polar_log_vars.contains(&LogLevel::Debug.to_string()) {
                Some(LogLevel::Debug)
            } else {
                Some(LogLevel::Info)
            }
        }
    }

    fn query_contains_partial(&mut self) {
        struct VarVisitor<'vm> {
            has_partial: bool,
            vm: &'vm PolarVirtualMachine,
        }

        impl<'vm> Visitor for VarVisitor<'vm> {
            fn visit_variable(&mut self, v: &Symbol) {
                if matches!(self.vm.variable_state(v), VariableState::Partial) {
                    self.has_partial = true;
                }
            }
        }

        let mut visitor = VarVisitor {
            has_partial: false,
            vm: self,
        };
        self.query_contains_partial = self.goals.iter().any(|goal| {
            if let Goal::Query { term } = goal.as_ref() {
                walk_term(&mut visitor, term);
                visitor.has_partial
            } else {
                false
            }
        });
    }

    #[cfg(test)]
    pub fn new_test(kb: Arc<RwLock<KnowledgeBase>>, tracing: bool, goals: Goals) -> Self {
        PolarVirtualMachine::new(kb, tracing, goals, MessageQueue::new())
    }

    /// Clone self, replacing the goal stack and retaining only the current bindings.
    pub fn clone_with_goals(&self, goals: Goals) -> Self {
        let mut vm = Self::new(self.kb.clone(), self.tracing, goals, self.messages.clone());
        vm.binding_manager.clone_from(&self.binding_manager);
        vm.query_contains_partial = self.query_contains_partial;
        vm.debugger = self.debugger.clone();
        vm
    }

    #[cfg(test)]
    fn set_stack_limit(&mut self, limit: usize) {
        self.stack_limit = limit;
    }

    fn kb(&self) -> RwLockReadGuard<KnowledgeBase> {
        self.kb.read().unwrap()
    }

    fn new_id(&self) -> u64 {
        self.kb().new_id()
    }

    pub fn id_counter(&self) -> Counter {
        self.kb().id_counter()
    }

    fn new_call_id(&mut self, symbol: &Symbol) -> u64 {
        let call_id = self.new_id();
        self.call_id_symbols.insert(call_id, symbol.clone());
        call_id
    }

    fn new_call_var(&mut self, var_prefix: &str, initial_value: Value) -> (u64, Term) {
        let sym = self.kb().gensym(var_prefix);
        self.bind(&sym, Term::from(initial_value)).unwrap();
        let call_id = self.new_call_id(&sym);
        (call_id, Term::from(sym))
    }

    fn get_call_sym(&self, call_id: u64) -> &Symbol {
        self.call_id_symbols
            .get(&call_id)
            .expect("unregistered external call ID")
    }

    /// Try to achieve one goal. Return `Some(QueryEvent)` if an external
    /// result is needed to achieve it, or `None` if it can run internally.
    fn next(&mut self, goal: Rc<Goal>) -> PolarResult<QueryEvent> {
        self.log(LogLevel::Trace, || goal.to_string(), &[]);

        self.check_timeout()?;

        match goal.as_ref() {
            Goal::Backtrack => self.backtrack()?,
            Goal::Cut { choice_index } => self.cut(*choice_index),
            Goal::Debug { message } => return Ok(self.debug(message)),
            Goal::Halt => return Ok(self.halt()),
            Goal::Error { error } => return Err(error.clone()),
            Goal::Isa { left, right } => self.isa(left, right)?,
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
            Goal::IsaExternal { instance, literal } => return self.isa_external(instance, literal),
            Goal::MakeExternal {
                constructor,
                instance_id,
            } => return Ok(self.make_external(constructor, *instance_id)),
            Goal::NextExternal { call_id, iterable } => {
                return self.next_external(*call_id, iterable)
            }
            Goal::CheckError => return self.check_error(),
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
            Goal::TraceStackPush => {
                self.trace_stack.push(Rc::new(self.trace.clone()));
                self.trace = vec![];
            }
            Goal::TraceStackPop => {
                let mut children = self.trace.clone();
                self.trace = self.trace_stack.pop().unwrap().as_ref().clone();
                let mut trace = self.trace.pop().unwrap();
                let trace = Rc::make_mut(&mut trace);
                trace.children.append(&mut children);
                self.trace.push(Rc::new(trace.clone()));
                self.maybe_break(DebugEvent::Pop)?;
            }
            Goal::TraceRule { trace } => {
                if let Node::Rule(rule) = &trace.node {
                    self.log(LogLevel::Info, || format!("RULE: {}", rule), &[]);
                }
                self.trace.push(trace.clone());
                self.maybe_break(DebugEvent::Rule)?;
            }
            Goal::Unify { left, right } => self.unify(left, right)?,
            Goal::AddConstraint { term } => self.add_constraint(term)?,
            Goal::AddConstraintsBatch { add_constraints } => {
                add_constraints
                    .borrow_mut()
                    .drain()
                    .try_for_each(|(_, constraint)| self.add_constraint(&constraint))?
            }
            Goal::Run { runnable } => return self.run_runnable(runnable.clone_runnable()),
        }
        Ok(QueryEvent::None)
    }

    /// Push a goal onto the goal stack.
    pub fn push_goal(&mut self, goal: Goal) -> PolarResult<()> {
        use {Goal::*, VariableState::Unbound};
        if self.goals.len() >= self.stack_limit {
            let msg = format!("Goal stack overflow! MAX_GOALS = {}", self.stack_limit);
            Err(RuntimeError::StackOverflow { msg }.into())
        } else if matches!(goal, LookupExternal { call_id, ..} | NextExternal { call_id, .. } if self.variable_state(self.get_call_sym(call_id)) != Unbound)
        {
            invalid_state("The call_id result variables for LookupExternal and NextExternal goals must be unbound.")
        } else {
            self.goals.push(Rc::new(goal));
            Ok(())
        }
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
    fn push_choice<I>(&mut self, alternatives: I) -> PolarResult<()>
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
        if self.choices.len() >= self.stack_limit {
            let msg = "Too many choices.".to_owned();
            Err(RuntimeError::StackOverflow { msg }.into())
        } else {
            self.choices.push(Choice {
                alternatives,
                bsp: self.bsp(),
                goals: self.goals.clone(),
                queries: self.queries.clone(),
                trace: self.trace.clone(),
                trace_stack: self.trace_stack.clone(),
            });
            Ok(())
        }
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
            self.push_choice(alternatives_iter)?;
            self.append_goals(alternative)
        } else {
            self.backtrack()
        }
    }

    /// If each goal of `conditional` succeeds, execute `consequent`;
    /// otherwise, execute `alternative`. The branches are entered only
    /// by backtracking so that bindings established during the execution
    /// of `conditional` are always unwound.
    fn choose_conditional(
        &mut self,
        mut conditional: Goals,
        consequent: Goals,
        mut alternative: Goals,
    ) -> PolarResult<()> {
        // If the conditional fails, cut the consequent.
        let cut_consequent = Goal::Cut {
            choice_index: self.choices.len(),
        };
        alternative.insert(0, cut_consequent);

        // If the conditional succeeds, cut the alternative and backtrack to this choice point.
        self.push_choice(vec![consequent])?;
        let cut_alternative = Goal::Cut {
            choice_index: self.choices.len(),
        };
        conditional.push(cut_alternative);
        conditional.push(Goal::Backtrack);

        self.choose(vec![conditional, alternative])
    }

    /// Push multiple goals onto the stack in reverse order.
    fn append_goals<I>(&mut self, goals: I) -> PolarResult<()>
    where
        I: IntoIterator<Item = Goal>,
        I::IntoIter: std::iter::DoubleEndedIterator,
    {
        goals.into_iter().rev().try_for_each(|g| self.push_goal(g))
    }

    /// Rebind an external answer variable.
    ///
    /// DO NOT USE THIS TO REBIND ANOTHER VARIABLE (see unsafe_rebind doc string).
    fn rebind_external_answer(&mut self, var: &Symbol, val: Term) {
        self.binding_manager.unsafe_rebind(var, val);
    }

    /// Push a binding onto the binding stack.
    pub fn bind(&mut self, var: &Symbol, val: Term) -> PolarResult<()> {
        self.log(
            LogLevel::Trace,
            || format!("⇒ bind: {} ← {}", var, val),
            &[],
        );
        if let Some(goal) = self.binding_manager.bind(var, val)? {
            self.push_goal(goal)
        } else {
            Ok(())
        }
    }

    pub fn add_binding_follower(&mut self) -> FollowerId {
        self.binding_manager.add_follower(BindingManager::new())
    }

    pub fn remove_binding_follower(&mut self, follower_id: &FollowerId) -> Option<BindingManager> {
        self.binding_manager.remove_follower(follower_id)
    }

    /// Add a single constraint operation to the variables referenced in it.
    /// Precondition: Operation is either binary or ternary (binary + result var),
    /// and at least one of the first two arguments is an unbound variable.
    fn add_constraint(&mut self, term: &Term) -> PolarResult<()> {
        self.log(
            LogLevel::Trace,
            || format!("⇒ add_constraint: {}", term),
            &[],
        );
        self.binding_manager.add_constraint(term)
    }

    /// Augment the bindings stack with constants from a hash map.
    /// There must be no temporaries bound yet.
    fn bind_constants(&mut self, bindings: Bindings) {
        assert_eq!(self.bsp(), self.csp);
        for (var, value) in bindings.iter() {
            self.bind(var, value.clone()).unwrap();
        }
        self.csp = self.bsp();
    }

    /// Retrieve the current non-constant bindings as a hash map.
    pub fn bindings(&self, include_temps: bool) -> Bindings {
        self.binding_manager
            .bindings_after(include_temps, &self.csp)
    }

    /// Retrieve internal binding stack for debugger.
    pub fn bindings_debug(&self) -> &BindingStack {
        self.binding_manager.bindings_debug()
    }

    /// Returns bindings for all vars used by terms in terms.
    pub fn relevant_bindings(&self, terms: &[&Term]) -> Bindings {
        let mut variables = HashSet::new();
        for t in terms {
            t.variables(&mut variables);
        }
        self.binding_manager.variable_bindings(&variables)
    }

    /// Return the current binding stack pointer.
    fn bsp(&self) -> Bsp {
        self.binding_manager.bsp()
    }

    /// Investigate the state of a variable at some point and return a variable state variant.
    pub fn variable_state_at_point(&self, variable: &Symbol, bsp: &Bsp) -> VariableState {
        self.binding_manager.variable_state_at_point(variable, bsp)
    }

    /// Investigate the current state of a variable and return a variable state variant.
    fn variable_state(&self, variable: &Symbol) -> VariableState {
        self.binding_manager.variable_state(variable)
    }

    /// Recursively dereference variables in a term, including subterms, except operations.
    fn deref(&self, term: &Term) -> Term {
        self.binding_manager.deep_deref(term)
    }

    /// Generate a fresh set of variables for a rule.
    fn rename_rule_vars(&self, rule: &Rule) -> Rule {
        let kb = &*self.kb.read().unwrap();
        let mut renamer = Renamer::new(kb);
        renamer.fold_rule(rule.clone())
    }

    /// Push or print a message to the output stream.
    #[cfg(not(target_arch = "wasm32"))]
    fn print<S: Into<String>>(&self, message: S) {
        let message = message.into();
        if self.polar_log_stderr {
            eprintln!("{}", message);
        } else {
            self.messages.push(MessageKind::Print, message);
        }
    }

    /// Push or print a message to the WASM output stream.
    #[cfg(target_arch = "wasm32")]
    fn print<S: Into<String>>(&self, message: S) {
        let message = message.into();
        if self.polar_log_stderr {
            console_error(&message);
        } else {
            self.messages.push(MessageKind::Print, message);
        }
    }

    fn log<F, R>(&self, level: LogLevel, message_fn: F, terms: &[&Term])
    where
        F: FnOnce() -> R,
        R: AsRef<str>,
    {
        if let Some(configured_log_level) = self.log_level {
            // preserve the old `polar_log_mute` behavior which omits parameter
            // specialization checking Unify, IsA and other events from the log
            if level == LogLevel::Trace && self.polar_trace_mute {
                return;
            }
            if configured_log_level.should_print_on_level(level) {
                let mut indent = String::new();
                for _ in 0..=self.queries.len() {
                    indent.push_str("  ");
                }
                let message = message_fn();
                let lines = message.as_ref().split('\n').collect::<Vec<&str>>();
                if let Some(line) = lines.first() {
                    let prefix = format!("[oso][{}] {}", level, &indent);
                    let mut msg = format!("{}{}", prefix, line);

                    // print BINDINGS: { .. } only for TRACE logs
                    if !terms.is_empty() && configured_log_level == LogLevel::Trace {
                        let relevant_bindings = self.relevant_bindings(terms);
                        write!(
                            msg,
                            ", BINDINGS: {{{}}}",
                            relevant_bindings
                                .iter()
                                .map(|(var, val)| format!("{} => {}", var.0, val))
                                .collect::<Vec<String>>()
                                .join(", ")
                        )
                        .unwrap();
                    }
                    self.print(msg);
                    for line in &lines[1..] {
                        self.print(format!("{}{}", prefix, line));
                    }
                }
            }
        }
    }

    /// Get the query stack as a string for printing in error messages.
    pub(crate) fn stack_trace(&self) -> String {
        let mut trace_stack = self.trace_stack.clone();
        let mut trace = self.trace.clone();

        // Build linear stack from trace tree. Not just using query stack because it doesn't
        // know about rules, query stack should really use this too.
        let mut stack = vec![];
        while let Some(t) = trace.last() {
            stack.push(t.clone());
            trace = trace_stack
                .pop()
                .map(|ts| ts.as_ref().clone())
                .unwrap_or_else(Vec::new);
        }

        stack.reverse();

        // Only index queries, not rules. Rule nodes are just used as context for where the query
        // comes from.
        let mut i = stack.iter().filter_map(|t| t.term()).count();

        let mut st = "trace (most recent evaluation last):\n".to_owned();
        let mut rule = None;
        for t in stack {
            match &t.node {
                Node::Rule(r) => {
                    rule = Some(r.clone());
                }
                Node::Term(t) => {
                    if matches!(t.value(), Value::Expression(Operation { operator: Operator::And, args}) if args.len() == 1)
                    {
                        continue;
                    }

                    i -= 1;
                    let _ = writeln!(st, "  {:03}: {}", i, self.term_source(t, false));

                    if let Some(context) = t.parsed_context() {
                        if let Some(rule) = &rule {
                            let _ = write!(st, "    in rule {}", rule.name);
                        } else {
                            let _ = write!(st, "    in query");
                        }
                        let _ = writeln!(st, "{}", context.source_position());
                    };
                }
            }
        }
        st
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn query_duration(&self) -> u64 {
        let now = std::time::Instant::now();
        let start = self.query_start_time.expect("Query start not recorded");
        (now - start).as_millis() as u64
    }

    #[cfg(target_arch = "wasm32")]
    fn query_duration(&self) -> u64 {
        let now: f64 = js_sys::Date::now();
        let start = self.query_start_time.expect("Query start not recorded");
        (now - start) as u64
    }

    fn is_query_timeout_disabled(&self) -> bool {
        self.query_timeout_ms == 0
    }

    fn check_timeout(&self) -> PolarResult<()> {
        if self.is_query_timeout_disabled() {
            // Useful for debugging
            return Ok(());
        }

        let elapsed = self.query_duration();
        let timeout = self.query_timeout_ms;
        if elapsed > timeout {
            return Err(RuntimeError::QueryTimeout { elapsed, timeout }.into());
        }
        Ok(())
    }
}

/// Implementations of instructions.
impl PolarVirtualMachine {
    /// Remove all bindings after the last choice point, and try the
    /// next available alternative. If no choice is possible, halt.
    fn backtrack(&mut self) -> PolarResult<()> {
        self.log(LogLevel::Trace, || "BACKTRACK", &[]);

        loop {
            match self.choices.pop() {
                None => return self.push_goal(Goal::Halt),
                Some(Choice {
                    mut alternatives,
                    bsp,
                    goals,
                    queries,
                    trace,
                    trace_stack,
                }) => {
                    self.binding_manager.backtrack(&bsp);
                    if let Some(mut alternative) = alternatives.pop() {
                        if alternatives.is_empty() {
                            self.goals = goals;
                            self.queries = queries;
                            self.trace = trace;
                            self.trace_stack = trace_stack;
                        } else {
                            self.goals.clone_from(&goals);
                            self.queries.clone_from(&queries);
                            self.trace.clone_from(&trace);
                            self.trace_stack.clone_from(&trace_stack);
                            self.choices.push(Choice {
                                alternatives,
                                bsp,
                                goals,
                                queries,
                                trace,
                                trace_stack,
                            })
                        }
                        self.goals.append(&mut alternative);
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Commit to the current choice.
    fn cut(&mut self, index: usize) {
        self.choices.truncate(index);
    }

    /// Clean up the query stack after completing a query.
    fn pop_query(&mut self) {
        self.queries.pop();
    }

    /// Interact with the debugger.
    fn debug(&mut self, message: &str) -> QueryEvent {
        // Query start time is reset when a debug event occurs.
        self.query_start_time.take();

        QueryEvent::Debug {
            message: message.to_string(),
        }
    }

    /// Halt the VM by clearing all goals and choices.
    fn halt(&mut self) -> QueryEvent {
        self.log(LogLevel::Trace, || "HALT", &[]);
        self.goals.clear();
        self.choices.clear();
        QueryEvent::Done { result: true }
    }

    /// Comparison operator that essentially performs partial unification.
    #[allow(clippy::many_single_char_names)]
    fn isa(&mut self, left: &Term, right: &Term) -> PolarResult<()> {
        self.log(
            LogLevel::Trace,
            || format!("MATCHES: {} matches {}", left, right),
            &[left, right],
        );

        match (left.value(), right.value()) {
            (_, Value::Dictionary(_)) => todo!("make this case unreachable"),
            (Value::Expression(_), _) | (_, Value::Expression(_)) => {
                unreachable!("encountered bare expression")
            }

            _ if self.kb.read().unwrap().is_union(left) => {
                // A union (currently) only matches itself.
                //
                // TODO(gj): when we have unions beyond `Actor` and `Resource`, we'll need to be
                // smarter about this check since UnionA is more specific than UnionB if UnionA is
                // a member of UnionB.
                let unions_match = (left.is_actor_union() && right.is_actor_union())
                    || (left.is_resource_union() && right.is_resource_union());
                if !unions_match {
                    return self.push_goal(Goal::Backtrack);
                }
            }
            _ if self.kb.read().unwrap().is_union(right) => self.isa_union(left, right)?,

            // TODO(gj): (Var, Rest) + (Rest, Var) cases might be unreachable.
            (Value::Variable(l), Value::Variable(r))
            | (Value::Variable(l), Value::RestVariable(r))
            | (Value::RestVariable(l), Value::Variable(r))
            | (Value::RestVariable(l), Value::RestVariable(r)) => {
                // Two variables.
                match (self.variable_state(l), self.variable_state(r)) {
                    (VariableState::Bound(x), _) => self.push_goal(Goal::Isa {
                        left: x,
                        right: right.clone(),
                    })?,
                    (_, VariableState::Bound(y)) => self.push_goal(Goal::Isa {
                        left: left.clone(),
                        right: y,
                    })?,
                    (_, _) => self.add_constraint(&term!(op!(Isa, left.clone(), right.clone())))?,
                }
            }
            (Value::Variable(l), _) | (Value::RestVariable(l), _) => match self.variable_state(l) {
                VariableState::Bound(x) => self.push_goal(Goal::Isa {
                    left: x,
                    right: right.clone(),
                })?,
                _ => self.isa_expr(left, right)?,
            },
            (_, Value::Variable(r)) | (_, Value::RestVariable(r)) => match self.variable_state(r) {
                VariableState::Bound(y) => self.push_goal(Goal::Isa {
                    left: left.clone(),
                    right: y,
                })?,
                _ => self.push_goal(Goal::Unify {
                    left: left.clone(),
                    right: right.clone(),
                })?,
            },

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
                        .get(k)
                        .expect("left fields should be a superset of right fields")
                        .clone();
                    self.push_goal(Goal::Isa {
                        left,
                        right: v.clone(),
                    })?;
                }
            }

            (_, Value::Pattern(Pattern::Dictionary(right))) => {
                // For each field in the dict, look up the corresponding field on the instance and
                // then isa them.
                for (field, right_value) in right.fields.iter() {
                    // Generate symbol for the lookup result and leave the variable unbound, so that unification with the result does not fail.
                    // Unification with the lookup result happens in `fn external_call_result()`.
                    let answer = self.kb.read().unwrap().gensym("isa_value");
                    let call_id = self.new_call_id(&answer);

                    let lookup = Goal::LookupExternal {
                        instance: left.clone(),
                        call_id,
                        field: right_value.clone_with_value(Value::String(field.0.clone())),
                    };
                    let isa = Goal::Isa {
                        left: Term::from(answer),
                        right: right_value.clone(),
                    };
                    self.append_goals(vec![lookup, isa])?;
                }
            }

            (_, Value::Pattern(Pattern::Instance(right_literal))) => {
                // Check fields
                self.push_goal(Goal::Isa {
                    left: left.clone(),
                    right: right.clone_with_value(Value::Pattern(Pattern::Dictionary(
                        right_literal.fields.clone(),
                    ))),
                })?;

                // attempt an in-core IsA check if we have the necessary
                // class_id information
                if let Value::ExternalInstance(ExternalInstance {
                    class_id: Some(class_id),
                    ..
                }) = *left.value()
                {
                    let isa = {
                        let kb = self.kb.read().unwrap();
                        let right_id = kb
                            .get_class_id_for_symbol(&right_literal.tag)
                            .expect("no class ID for symbol");
                        let left_symbol = kb
                            .get_symbol_for_class_id(&class_id)
                            .expect("no symbol for class ID");
                        if let Some(mro) = kb.mro.get(left_symbol) {
                            mro.contains(right_id)
                        } else {
                            false
                        }
                    };
                    if !isa {
                        self.push_goal(Goal::Backtrack)?;
                    }
                // default to IsaExternal when no `class_id` information is available
                } else {
                    // Check class
                    self.push_goal(Goal::IsaExternal {
                        instance: left.clone(),
                        literal: right_literal.clone(),
                    })?;
                }
            }

            // Default case: x isa y if x = y.
            _ => self.push_goal(Goal::Unify {
                left: left.clone(),
                right: right.clone(),
            })?,
        }
        Ok(())
    }

    fn get_names(&self, s: &Symbol) -> HashSet<Symbol> {
        let cycles = self
            .binding_manager
            .get_constraints(s)
            .constraints()
            .into_iter()
            .filter_map(|con| match con.operator {
                Operator::Unify | Operator::Eq => {
                    if let (Ok(l), Ok(r)) = (con.args[0].as_symbol(), con.args[1].as_symbol()) {
                        Some((l.clone(), r.clone()))
                    } else {
                        None
                    }
                }
                _ => None,
            });

        partition_equivs(cycles)
            .into_iter()
            .find(|c| c.contains(s))
            .unwrap_or_else(|| {
                let mut hs = HashSet::with_capacity(1);
                hs.insert(s.clone());
                hs
            })
    }

    fn isa_expr(&mut self, left: &Term, right: &Term) -> PolarResult<()> {
        match right.value() {
            Value::Pattern(Pattern::Dictionary(fields)) => {
                // Produce a constraint like left.field = value
                let to_unify = |(field, value): (&Symbol, &Term)| -> Term {
                    let value = self.deref(value);
                    let field = right.clone_with_value(value!(field.0.as_ref()));
                    let left = left.clone_with_value(value!(op!(Dot, left.clone(), field)));
                    term!(op!(Unify, left, value))
                };

                let constraints = fields.fields.iter().rev().map(to_unify).collect::<Vec<_>>();
                for op in constraints {
                    self.add_constraint(&op)?;
                }
            }
            Value::Pattern(Pattern::Instance(InstanceLiteral { fields, tag })) => {
                // TODO(gj): assert that a simplified expression contains at most 1 unification
                // involving a particular variable.
                // TODO(gj): Ensure `op!(And) matches X{}` doesn't die after these changes.
                let var = left.as_symbol()?;

                // Get the existing partial on the LHS variable.
                let partial = self.binding_manager.get_constraints(var);

                let names = self.get_names(var);
                let output = names.clone();

                let partial = partial.into();
                let (simplified, _) = simplify_partial(var, partial, output, false);

                let simplified = simplified.as_expression()?;

                // TODO (dhatch): what if there is more than one var = dot_op constraint?
                // What if the one there is is in a not, or an or, or something
                let lhss_of_matches = simplified
                    .constraints()
                    .into_iter()
                    .filter_map(|c| {
                        // If the simplified partial includes a constraint of form:
                        // `v = dot_op`, `dot_op = v`, or `v in dot_op`
                        // and the receiver of the dot operation is either
                        // `var` or an alias thereof, use the dot op as the LHS of the matches.
                        if c.operator != Operator::Unify && c.operator != Operator::In {
                            None
                        } else if matches!(c.args[0].as_symbol(), Ok(s) if names.contains(s)) &&
                            matches!(c.args[1].as_expression(), Ok(o) if o.operator == Operator::Dot) {
                            Some(c.args[1].clone())
                        } else if c.operator == Operator::Unify && matches!(c.args[1].as_symbol(), Ok(s) if names.contains(s)) &&
                            // only look for var on the RHS of a unfication (i.e. not on the RHS of an `in`)
                            matches!(c.args[0].as_expression(), Ok(o) if o.operator == Operator::Dot) {
                            Some(c.args[0].clone())
                        } else {
                            None
                        }
                    })
                    .chain(std::iter::once(left.clone()));

                // Construct field-less matches operation.
                let tag_pattern = right.clone_with_value(value!(pattern!(instance!(tag.clone()))));
                let type_constraint = op!(Isa, left.clone(), tag_pattern);

                let new_matcheses =
                    lhss_of_matches.map(|lhs_of_matches| op!(Isa, lhs_of_matches, right.clone()));

                let runnables = new_matcheses
                    .map(|new_matches| {
                        let runnable = Box::new(IsaConstraintCheck::new(
                            simplified.constraints(),
                            new_matches,
                            names.clone(),
                        ));
                        Goal::Run { runnable }
                    })
                    .collect();

                // Construct field constraints.
                let field_constraints = fields.fields.iter().rev().map(|(f, v)| {
                    let v = self.deref(v);
                    let field = right.clone_with_value(value!(f.0.as_ref()));
                    let left = left.clone_with_value(value!(op!(Dot, left.clone(), field)));
                    op!(Unify, left, v)
                });

                let mut add_constraints = vec![type_constraint];
                add_constraints.extend(field_constraints.into_iter());

                // Run compatibility check.
                self.choose_conditional(
                    runnables,
                    add_constraints
                        .into_iter()
                        .map(|op| Goal::AddConstraint { term: op.into() })
                        .collect(),
                    vec![Goal::CheckError, Goal::Backtrack],
                )?;
            }
            // if the RHS isn't a pattern or a dictionary, we'll fall back to unifying
            // this is not the _best_ behaviour, but it's what we've been doing
            // previously
            _ => self.push_goal(Goal::Unify {
                left: left.clone(),
                right: right.clone(),
            })?,
        }
        Ok(())
    }

    /// To evaluate `left matches Union`, look up `Union`'s member classes and create a choicepoint
    /// to check if `left` matches any of them.
    fn isa_union(&mut self, left: &Term, union: &Term) -> PolarResult<()> {
        let member_isas = {
            let kb = self.kb.read().unwrap();
            let members = kb.get_union_members(union).iter();
            members
                .map(|member| {
                    let tag = member.as_symbol().unwrap().0.as_str();
                    member.clone_with_value(value!(pattern!(instance!(tag))))
                })
                .map(|pattern| {
                    vec![Goal::Isa {
                        left: left.clone(),
                        right: pattern,
                    }]
                })
                .collect::<Vec<Goals>>()
        };
        self.choose(member_isas)
    }

    fn lookup(&mut self, dict: &Dictionary, field: &Term, value: &Term) -> PolarResult<()> {
        let field = self.deref(field);
        match field.value() {
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
                self.choose(alternatives)
            }
            Value::String(field) => {
                if let Some(retrieved) = dict.fields.get(&Symbol(field.clone())) {
                    self.push_goal(Goal::Unify {
                        left: retrieved.clone(),
                        right: value.clone(),
                    })
                } else {
                    self.push_goal(Goal::Backtrack)
                }
            }
            v => self.type_error(
                &field,
                format!("cannot look up field {:?} on a dictionary", v),
            ),
        }
    }

    /// Return an external call event to look up a field's value
    /// in an external instance. Push a `Goal::LookupExternal` as
    /// an alternative on the last choice point to poll for results.
    fn lookup_external(
        &mut self,
        call_id: u64,
        instance: &Term,
        field: &Term,
    ) -> PolarResult<QueryEvent> {
        let (field_name, args, kwargs): (
            Symbol,
            Option<Vec<Term>>,
            Option<BTreeMap<Symbol, Term>>,
        ) = match self.deref(field).value() {
            Value::Call(Call { name, args, kwargs }) => (
                name.clone(),
                Some(args.iter().map(|arg| self.deref(arg)).collect()),
                kwargs.as_ref().map(|unwrapped| {
                    unwrapped
                        .iter()
                        .map(|(k, v)| (k.to_owned(), self.deref(v)))
                        .collect()
                }),
            ),
            Value::String(field) => (Symbol(field.clone()), None, None),
            v => {
                return self.type_error(
                    field,
                    format!("cannot look up field {:?} on an external instance", v),
                )
            }
        };

        // add an empty choice point; lookups return only one value
        // but we'll want to cut if we get back nothing
        self.push_choice(vec![])?;

        self.log(
            LogLevel::Trace,
            || {
                let mut msg = format!("LOOKUP: {}.{}", instance, field_name);
                msg.push('(');
                let args = args
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|a| a.to_string());
                let kwargs = kwargs
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(k, v)| format!("{}: {}", k, v));
                msg.push_str(&args.chain(kwargs).collect::<Vec<_>>().join(", "));
                msg.push(')');
                msg
            },
            &[],
        );

        Ok(QueryEvent::ExternalCall {
            call_id,
            instance: self.deref(instance),
            attribute: field_name,
            args,
            kwargs,
        })
    }

    fn isa_external(
        &mut self,
        instance: &Term,
        literal: &InstanceLiteral,
    ) -> PolarResult<QueryEvent> {
        let (call_id, answer) = self.new_call_var("isa", false.into());
        self.push_goal(Goal::Unify {
            left: answer,
            right: Term::from(true),
        })?;

        Ok(QueryEvent::ExternalIsa {
            call_id,
            instance: self.deref(instance),
            class_tag: literal.tag.clone(),
        })
    }

    fn next_external(&mut self, call_id: u64, iterable: &Term) -> PolarResult<QueryEvent> {
        // add another choice point for the next result
        self.push_choice(vec![vec![Goal::NextExternal {
            call_id,
            iterable: iterable.clone(),
        }]])?;

        Ok(QueryEvent::NextExternal {
            call_id,
            iterable: iterable.clone(),
        })
    }

    fn make_external(&self, constructor: &Term, instance_id: u64) -> QueryEvent {
        QueryEvent::MakeExternal {
            instance_id,
            constructor: self.deref(constructor),
        }
    }

    fn check_error(&mut self) -> PolarResult<QueryEvent> {
        if let Some(msg) = self.external_error.take() {
            let term = match self.trace.last().map(|t| t.node.clone()) {
                Some(Node::Term(t)) => Some(t),
                _ => None,
            };
            let stack_trace = self.stack_trace();
            Err(RuntimeError::Application {
                msg,
                stack_trace,
                term,
            }
            .into())
        } else {
            Ok(QueryEvent::None)
        }
    }

    /// Query for the provided term.
    ///
    /// Uses the knowledge base to get an ordered list of rules.
    /// Creates a choice point over each rule, where each alternative
    /// consists of unifying the rule head with the arguments, then
    /// querying for each body clause.
    fn query(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        // - Print INFO event for queries for rules.
        // - Print TRACE (a superset of INFO) event for all other queries.
        // - We filter out single-element ANDs, which many rule bodies take the form of, to instead
        //   log only their inner operations for readability|brevity reasons.
        match &term.value() {
            Value::Call(predicate) => {
                self.log(
                    LogLevel::Info,
                    || format!("QUERY RULE: {}", predicate),
                    &[term],
                );
            }
            Value::Expression(Operation {
                operator: Operator::And,
                args,
            }) if args.len() < 2 => (),
            _ => {
                self.log(LogLevel::Trace, || format!("QUERY: {}", term), &[term]);
            }
        };

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
            Value::Expression(_) => {
                return self.query_for_operation(term);
            }
            Value::Variable(sym) => {
                self.push_goal(
                    if let VariableState::Bound(val) = self.variable_state(sym) {
                        Goal::Query { term: val }
                    } else {
                        // variable was unbound
                        // apply a constraint to variable that it must be truthy
                        Goal::Unify {
                            left: term.clone(),
                            right: term!(true),
                        }
                    },
                )?
            }
            Value::Boolean(value) => {
                if !value {
                    // Backtrack if the boolean is false.
                    self.push_goal(Goal::Backtrack)?;
                }

                return Ok(QueryEvent::None);
            }
            _ => {
                // everything else dies horribly and in pain
                return self.type_error(
                    term,
                    format!(
                        "{} isn't something that is true or false so can't be a condition",
                        term
                    ),
                );
            }
        }
        Ok(QueryEvent::None)
    }

    /// Select applicable rules for predicate.
    /// Sort applicable rules by specificity.
    /// Create a choice over the applicable rules.
    fn query_for_predicate(&mut self, predicate: Call) -> PolarResult<()> {
        if predicate.kwargs.is_some() {
            return invalid_state(format!(
                "query_for_predicate: unexpected kwargs: {}",
                predicate
            ));
        }
        let goals = match self.kb.read().unwrap().get_generic_rule(&predicate.name) {
            None => {
                return Err(RuntimeError::QueryForUndefinedRule {
                    name: predicate.name.0.clone(),
                }
                .into())
            }
            Some(generic_rule) => {
                if generic_rule.name != predicate.name {
                    return invalid_state(format!(
                        "query_for_predicate: different rule names: {} != {}",
                        generic_rule.name, predicate.name
                    ));
                }

                // Pre-filter rules.
                let args = predicate.args.iter().map(|t| self.deref(t)).collect();
                let pre_filter = generic_rule.get_applicable_rules(&args);

                self.polar_trace_mute = true;

                // Filter rules by applicability.
                vec![
                    Goal::TraceStackPush,
                    Goal::FilterRules {
                        applicable_rules: vec![],
                        unfiltered_rules: pre_filter,
                        args: predicate.args,
                    },
                    Goal::TraceStackPop,
                ]
            }
        };
        self.append_goals(goals)
    }

    fn query_for_operation(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        let operation = term.as_expression().unwrap();
        let mut args = operation.args.clone();
        let wrong_arity = || invalid_state(format!("query_for_operation: wrong arity: {}", term));
        match operation.operator {
            Operator::And => {
                // Query for each conjunct.
                self.push_goal(Goal::TraceStackPop)?;
                self.append_goals(args.into_iter().map(|term| Goal::Query { term }))?;
                self.push_goal(Goal::TraceStackPush)?;
            }
            Operator::Or => {
                // Make an alternative Query for each disjunct.
                self.choose(args.into_iter().map(|term| vec![Goal::Query { term }]))?;
            }
            Operator::Not => {
                // Query in a sub-VM and invert the results.
                if args.len() != 1 {
                    return wrong_arity();
                }

                let term = args.pop().unwrap();
                let add_constraints = Rc::new(RefCell::new(Bindings::new()));
                let inverter = Box::new(Inverter::new(
                    self,
                    vec![Goal::Query { term }],
                    add_constraints.clone(),
                    self.bsp(),
                ));
                self.choose_conditional(
                    vec![Goal::Run { runnable: inverter }],
                    vec![Goal::AddConstraintsBatch { add_constraints }],
                    vec![Goal::Backtrack],
                )?;
            }
            Operator::Assign => {
                if args.len() != 2 {
                    return wrong_arity();
                }
                let right = args.pop().unwrap();
                let left = args.pop().unwrap();
                match (left.value(), right.value()) {
                    (Value::Variable(var), _) => match self.variable_state(var) {
                        VariableState::Unbound => {
                            self.push_goal(Goal::Unify { left, right })?;
                        }
                        _ => {
                            return self.type_error(
                                &left,
                                format!(
                                    "Can only assign to unbound variables, {} is not unbound.",
                                    var
                                ),
                            );
                        }
                    },
                    _ => return self.type_error(&left, format!("Cannot assign to type {}.", left)),
                }
            }

            Operator::Unify => {
                // Push a `Unify` goal
                if args.len() != 2 {
                    return wrong_arity();
                }
                let right = args.pop().unwrap();
                let left = args.pop().unwrap();
                self.push_goal(Goal::Unify { left, right })?
            }
            Operator::Dot => {
                return self.query_op_helper(term, Self::dot_op_helper, false, false);
            }

            Operator::Lt
            | Operator::Gt
            | Operator::Leq
            | Operator::Geq
            | Operator::Eq
            | Operator::Neq => {
                return self.query_op_helper(term, Self::comparison_op_helper, true, true);
            }

            Operator::Add
            | Operator::Sub
            | Operator::Mul
            | Operator::Div
            | Operator::Mod
            | Operator::Rem => {
                return self.query_op_helper(term, Self::arithmetic_op_helper, true, true);
            }

            Operator::In => {
                return self.query_op_helper(term, Self::in_op_helper, false, true);
            }

            Operator::Debug => {
                let message = self.debugger.break_msg(self).unwrap_or_else(|| {
                    format!(
                        "debug({})",
                        args.iter()
                            .map(|arg| self.deref(arg).to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                });
                self.push_goal(Goal::Debug { message })?;
            }
            Operator::Print => {
                self.print(
                    args.iter()
                        .map(|arg| self.deref(arg).to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                );
            }
            Operator::New => {
                if args.len() != 2 {
                    return wrong_arity();
                }
                let result = args.pop().unwrap();
                result.as_symbol()?; // Ensure `result` is a variable.
                let constructor = args.pop().unwrap();

                let instance_id = self.new_id();

                let class = &constructor.as_call()?.name;
                let class_repr = if self.kb().is_constant(class) {
                    Some(class.0.clone())
                } else {
                    None
                };
                let instance =
                    constructor.clone_with_value(Value::ExternalInstance(ExternalInstance {
                        instance_id,
                        constructor: Some(constructor.clone()),
                        repr: Some(constructor.to_string()),
                        class_repr,
                        class_id: None,
                    }));

                // A goal is used here in case the result is already bound to some external
                // instance.
                self.append_goals(vec![
                    Goal::Unify {
                        left: result,
                        right: instance,
                    },
                    Goal::MakeExternal {
                        instance_id,
                        constructor,
                    },
                ])?;
            }
            Operator::Cut => {
                if self.query_contains_partial {
                    return unsupported("cannot use cut with partial evaluation", term);
                }

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
                // TODO (dhatch): Use query op helper.
                if args.len() != 2 {
                    return wrong_arity();
                }
                let right = args.pop().unwrap();
                let left = args.pop().unwrap();
                self.push_goal(Goal::Isa { left, right })?
            }
            Operator::ForAll => {
                if args.len() != 2 {
                    return wrong_arity();
                }
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
        }
        Ok(QueryEvent::None)
    }

    /// Handle variables & constraints as arguments to various operations.
    /// Calls the `eval` method to handle ground terms.
    ///
    /// Arguments:
    ///
    /// - handle_unbound_left_var: If set to `false`, allow `eval` to handle
    ///   operations with an unbound left variable, instead of adding a constraint.
    ///   Some operations, like `In`, emit new goals or choice points when the left
    ///   operand is a variable.
    /// - handle_unbound_right_var: Same as above but for the RHS. `Dot` uses this.
    #[allow(clippy::many_single_char_names)]
    fn query_op_helper<F>(
        &mut self,
        term: &Term,
        eval: F,
        handle_unbound_left_var: bool,
        handle_unbound_right_var: bool,
    ) -> PolarResult<QueryEvent>
    where
        F: Fn(&mut Self, &Term) -> PolarResult<QueryEvent>,
    {
        let Operation { operator: op, args } = term.as_expression().unwrap();

        let mut args = args.clone();
        if args.len() < 2 {
            return invalid_state(format!("query_op_helper: wrong arity: {}", term));
        }
        let left = &args[0];
        let right = &args[1];

        match (left.value(), right.value()) {
            // We may be querying a partial from the simplifier, which can contain
            // embedded binary (as opposed to ternary) dot operations. In that case
            // we introduce a new variable, unify it with the dot lookup, then query
            // against the variable instead.
            //
            // TODO(gw) take these out after the simplifier/inverter work better ...
            //
            // dot on the left
            (
                Value::Expression(Operation {
                    operator: Operator::Dot,
                    args,
                }),
                _,
            ) if args.len() == 2 => {
                let var = term!(self.kb().gensym("rwdot"));
                let val = Value::Expression(Operation {
                    operator: *op,
                    args: vec![var.clone(), right.clone()],
                });
                let term = term.clone_with_value(val);
                self.push_goal(Goal::Query { term })?;
                self.push_goal(Goal::Unify {
                    left: left.clone(),
                    right: var,
                })?;
                return Ok(QueryEvent::None);
            }

            // dot on the right
            (
                _,
                Value::Expression(Operation {
                    operator: Operator::Dot,
                    args,
                }),
            ) if args.len() == 2 => {
                let var = term!(self.kb().gensym("rwdot"));
                let val = Value::Expression(Operation {
                    operator: *op,
                    args: vec![left.clone(), var.clone()],
                });
                let term = term.clone_with_value(val);
                self.push_goal(Goal::Query { term })?;
                self.push_goal(Goal::Unify {
                    left: var,
                    right: right.clone(),
                })?;
                return Ok(QueryEvent::None);
            }

            // otherwise this isn't allowed.
            (Value::Expression(_), _)
            | (_, Value::Expression(_))
            | (Value::RestVariable(_), _)
            | (_, Value::RestVariable(_)) => {
                return invalid_state(format!("invalid query: {}", term));
            }
            _ => {}
        };

        if let Value::Variable(r) = right.value() {
            if let VariableState::Bound(x) = self.variable_state(r) {
                args[1] = x;
                self.push_goal(Goal::Query {
                    term: term.clone_with_value(Value::Expression(Operation {
                        operator: *op,
                        args,
                    })),
                })?;
                return Ok(QueryEvent::None);
            } else if !handle_unbound_right_var && left.as_symbol().is_err() {
                return eval(self, term);
            }
        }

        if let Value::Variable(l) = left.value() {
            if let VariableState::Bound(x) = self.variable_state(l) {
                args[0] = x;
                self.push_goal(Goal::Query {
                    term: term.clone_with_value(Value::Expression(Operation {
                        operator: *op,
                        args,
                    })),
                })?;
                return Ok(QueryEvent::None);
            } else if !handle_unbound_left_var && right.as_symbol().is_err() {
                return eval(self, term);
            }
        }

        if left.as_symbol().is_ok() || right.as_symbol().is_ok() {
            self.add_constraint(term)?;
            return Ok(QueryEvent::None);
        }

        eval(self, term)
    }

    /// Evaluate comparison operations.
    fn comparison_op_helper(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        let Operation { operator: op, args } = term.as_expression().unwrap();

        if args.len() != 2 {
            return invalid_state(format!("comparison_op_helper: wrong arity: {}", term));
        }
        let left = &args[0];
        let right = &args[1];

        match (left.value(), right.value()) {
            (Value::ExternalInstance(_), _) | (_, Value::ExternalInstance(_)) => {
                // Generate a symbol for the external result and bind to `false` (default).
                let (call_id, answer) = self.new_call_var("external_op_result", false.into());

                // Check that the external result is `true` when we return.
                self.push_goal(Goal::Unify {
                    left: answer,
                    right: Term::from(true),
                })?;

                // Emit an event for the external operation.
                Ok(QueryEvent::ExternalOp {
                    call_id,
                    operator: *op,
                    args: vec![left.clone(), right.clone()],
                })
            }
            _ => {
                if !compare(*op, left, right, Some(term))? {
                    self.push_goal(Goal::Backtrack)?;
                }
                Ok(QueryEvent::None)
            }
        }
    }

    // TODO(ap, dhatch): Rewrite 3-arg arithmetic ops as 2-arg + unify,
    // like we do for dots; e.g., `+(a, b, c)` → `c = +(a, b)`.
    /// Evaluate arithmetic operations.
    fn arithmetic_op_helper(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        let Operation { operator: op, args } = term.as_expression().unwrap();

        if args.len() != 3 {
            return invalid_state(format!("arithmetic_op_helper: wrong arity: {}", term));
        }
        let left = &args[0];
        let right = &args[1];
        let result = &args[2];
        result.as_symbol()?; // Ensure `result` is a variable.

        match (left.value(), right.value()) {
            (Value::Number(left), Value::Number(right)) => {
                if let Some(answer) = match op {
                    Operator::Add => *left + *right,
                    Operator::Sub => *left - *right,
                    Operator::Mul => *left * *right,
                    Operator::Div => *left / *right,
                    Operator::Mod => (*left).modulo(*right),
                    Operator::Rem => *left % *right,
                    _ => return unsupported(format!("numeric operation {}", op), term),
                } {
                    self.push_goal(Goal::Unify {
                        left: term.clone_with_value(Value::Number(answer)),
                        right: result.clone(),
                    })?;
                    Ok(QueryEvent::None)
                } else {
                    Err(RuntimeError::ArithmeticError { term: term.clone() }.into())
                }
            }
            (_, _) => unsupported(format!("unsupported arithmetic operands: {}", term), term),
        }
    }

    /// Push appropriate goals for lookups on dictionaries and instances.
    fn dot_op_helper(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        let Operation { args, .. } = term.as_expression().unwrap();

        if args.len() != 3 {
            return invalid_state(format!("dot_op_helper: wrong arity: {}", term));
        }
        let mut args = args.clone();
        let object = &args[0];
        let field = &args[1];
        let value = &args[2];

        match object.value() {
            // Push a `Lookup` goal for simple field lookups on dictionaries.
            Value::Dictionary(dict)
                if matches!(field.value(), Value::String(_) | Value::Variable(_)) =>
            {
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
                let answer = self.kb.read().unwrap().gensym("lookup_value");
                let call_id = self.new_call_id(&answer);
                self.append_goals(vec![
                    Goal::LookupExternal {
                        call_id,
                        field: field.clone(),
                        instance: object.clone(),
                    },
                    Goal::CheckError,
                    Goal::Unify {
                        left: value.clone(),
                        right: Term::from(answer),
                    },
                ])?;
            }
            Value::Variable(v) => {
                if matches!(field.value(), Value::Call(_)) {
                    return unsupported(
                        format!("cannot call method on unbound variable {}", v),
                        object,
                    );
                }

                // Translate `.(object, field, value)` → `value = .(object, field)`.
                let dot2 = op!(Dot, object.clone(), field.clone());
                let value = self.deref(value);
                let term = Term::from(op!(Unify, value, dot2.into()));
                self.add_constraint(&term)?;
            }
            _ => {
                return self.type_error(
                    object,
                    format!(
                        "can only perform lookups on dicts and instances, this is {}",
                        object
                    ),
                )
            }
        }
        Ok(QueryEvent::None)
    }

    fn in_op_helper(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        let Operation { args, .. } = term.as_expression().unwrap();

        if args.len() != 2 {
            return invalid_state(format!("in_op_helper: wrong arity: {}", term));
        }
        let item = &args[0];
        let iterable = &args[1];
        let item_is_ground = item.is_ground();

        match iterable.value() {
            // Unify item with each element of the list, skipping non-matching ground terms.
            Value::List(terms) => self.choose(
                terms
                    .iter()
                    .filter(|term| {
                        !item_is_ground || !term.is_ground() || term.value() == item.value()
                    })
                    .map(|term| match term.value() {
                        Value::RestVariable(v) => {
                            let term = op!(In, item.clone(), Term::from(v.clone())).into();
                            vec![Goal::Query { term }]
                        }
                        _ => vec![Goal::Unify {
                            left: item.clone(),
                            right: term.clone(),
                        }],
                    })
                    .collect::<Vec<Goals>>(),
            )?,
            // Unify item with each (k, v) pair of the dict, skipping non-matching ground terms.
            Value::Dictionary(dict) => self.choose(
                dict.fields
                    .iter()
                    .map(|(k, v)| {
                        iterable.clone_with_value(Value::List(vec![
                            v.clone_with_value(Value::String(k.0.clone())),
                            v.clone(),
                        ]))
                    })
                    .filter(|term| {
                        !item_is_ground || !term.is_ground() || term.value() == item.value()
                    })
                    .map(|term| {
                        vec![Goal::Unify {
                            left: item.clone(),
                            right: term,
                        }]
                    })
                    .collect::<Vec<Goals>>(),
            )?,
            // Unify item with each element of the string
            // FIXME (gw): this seems strange, wouldn't a substring search make more sense?
            Value::String(s) => self.choose(
                s.chars()
                    .map(|c| c.to_string())
                    .map(Value::String)
                    .filter(|c| !item_is_ground || c == item.value())
                    .map(|c| {
                        vec![Goal::Unify {
                            left: item.clone(),
                            right: iterable.clone_with_value(c),
                        }]
                    })
                    .collect::<Vec<Goals>>(),
            )?,
            // Push an `ExternalLookup` goal for external instances
            Value::ExternalInstance(_) => {
                // Generate symbol for next result and leave the variable unbound, so that unification with the result does not fail
                // Unification of the `next_sym` variable with the result of `NextExternal` happens in `fn external_call_result()`
                // `external_call_result` is the handler for results from both `LookupExternal` and `NextExternal`, so neither can bind the
                // call ID variable to `false`.
                let next_sym = self.kb.read().unwrap().gensym("next_value");
                let call_id = self.new_call_id(&next_sym);

                // append unify goal to be evaluated after
                // next result is fetched
                self.append_goals(vec![
                    Goal::NextExternal {
                        call_id,
                        iterable: self.deref(iterable),
                    },
                    Goal::Unify {
                        left: item.clone(),
                        right: Term::from(next_sym),
                    },
                ])?;
            }
            _ => {
                return self.type_error(
                    iterable,
                    format!(
                        "can only use `in` on an iterable value, this is {:?}",
                        iterable.value()
                    ),
                );
            }
        }
        Ok(QueryEvent::None)
    }

    /// Unify `left` and `right` terms.
    ///
    /// Outcomes of a unification are:
    ///  - Successful unification => bind zero or more variables to values
    ///  - Recursive unification => more `Unify` goals are pushed onto the stack
    ///  - Failure => backtrack
    fn unify(&mut self, left: &Term, right: &Term) -> PolarResult<()> {
        match (left.value(), right.value()) {
            (Value::Expression(op), other) | (other, Value::Expression(op)) => {
                match op {
                    // this branch handles dot ops that were rewritten for inclusion
                    // in a partial by Vm::dot_op_helper(), but then queried again after
                    // the partial was bound by Vm::bind().
                    Operation {
                        operator: Operator::Dot,
                        args,
                    } if args.len() == 2 => {
                        let term = Term::from(op!(
                            Dot,
                            args[0].clone(),
                            args[1].clone(),
                            Term::from(other.clone())
                        ));
                        self.push_goal(Goal::Query { term })?
                    }
                    // otherwise this should never happen.
                    _ => {
                        return self.type_error(
                            left,
                            format!("cannot unify expressions directly `{}` = `{}`", left, right),
                        )
                    }
                }
            }
            (Value::Pattern(_), _) | (_, Value::Pattern(_)) => {
                return self.type_error(
                    left,
                    format!("cannot unify patterns directly `{}` = `{}`", left, right),
                );
            }

            // Unify two variables.
            // TODO(gj): (Var, Rest) + (Rest, Var) cases might be unreachable.
            (Value::Variable(l), Value::Variable(r))
            | (Value::Variable(l), Value::RestVariable(r))
            | (Value::RestVariable(l), Value::Variable(r))
            | (Value::RestVariable(l), Value::RestVariable(r)) => {
                // FIXME(gw):
                // if the variables are the same the unification succeeds, so
                // we don't need to do anything. but this causes an inconsistency
                // with NaN where `nan = nan` is false but `x = nan and x = x` is
                // true. if we really want to keep the NaN equality semantics
                // maybe we can have `nan = nan` but not `nan == nan`?
                if l != r {
                    match (self.variable_state(l), self.variable_state(r)) {
                        (VariableState::Bound(x), VariableState::Bound(y)) => {
                            // Both variables are bound. Unify their values.
                            self.push_goal(Goal::Unify { left: x, right: y })?;
                        }
                        _ => {
                            // At least one variable is unbound. Bind it.
                            if self.bind(l, right.clone()).is_err() {
                                self.push_goal(Goal::Backtrack)?;
                            }
                        }
                    }
                }
            }

            // FIXME(gw): i think we might actually want this, see the comment
            // above about unifying variables.
            // (Value::Number(Numeric::Float(a)),
            //  Value::Number(Numeric::Float(b)))
            //     if a.is_nan() && b.is_nan() => (),

            // Unify/bind a variable on the left with/to the term on the right.
            (Value::Variable(var), _) | (Value::RestVariable(var), _) => {
                let right = right.clone();
                match self.variable_state(var) {
                    VariableState::Bound(value) => {
                        self.push_goal(Goal::Unify { left: value, right })?;
                    }
                    _ => {
                        if self.bind(var, right).is_err() {
                            self.push_goal(Goal::Backtrack)?;
                        }
                    }
                }
            }

            // Unify/bind a variable on the right with/to the term on the left.
            (_, Value::Variable(var)) | (_, Value::RestVariable(var)) => {
                let left = left.clone();
                match self.variable_state(var) {
                    VariableState::Bound(value) => {
                        self.push_goal(Goal::Unify { left, right: value })?;
                    }
                    _ => {
                        if self.bind(var, left).is_err() {
                            self.push_goal(Goal::Backtrack)?;
                        }
                    }
                }
            }

            // Unify predicates like unifying heads
            (Value::Call(left), Value::Call(right)) => {
                if left.kwargs.is_some() || right.kwargs.is_some() {
                    // Handled in the parser.
                    return invalid_state("unify: unexpected kwargs");
                }
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

            // Unify lists by recursively unifying their elements.
            (Value::List(l), Value::List(r)) => self.unify_lists(l, r, |(l, r)| Goal::Unify {
                left: l.clone(),
                right: r.clone(),
            })?,

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
                    let right = right.fields.get(k).expect("fields should be equal").clone();
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

            (
                Value::ExternalInstance(ExternalInstance {
                    instance_id: left, ..
                }),
                Value::ExternalInstance(ExternalInstance {
                    instance_id: right, ..
                }),
            ) if left == right => (),

            // If either operand is an external instance, let the host
            // compare them for equality. This handles unification between
            // "equivalent" host and native types transparently.
            (Value::ExternalInstance(_), _) | (_, Value::ExternalInstance(_)) => {
                self.push_goal(Goal::Query {
                    term: Term::from(Operation {
                        operator: Operator::Eq,
                        args: vec![left.clone(), right.clone()],
                    }),
                })?
            }

            // Anything else fails.
            (_, _) => self.push_goal(Goal::Backtrack)?,
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
        if has_rest_var(left) && has_rest_var(right) {
            self.unify_two_lists_with_rest(left, right, unify)
        } else if has_rest_var(left) {
            self.unify_rest_list_with_list(left, right, unify)
        } else if has_rest_var(right) {
            self.unify_rest_list_with_list(right, left, unify)
        } else if left.len() == right.len() {
            // No rest-variables; unify element-wise.
            self.append_goals(left.iter().zip(right).map(unify))
        } else {
            self.push_goal(Goal::Backtrack)
        }
    }

    /// Unify two list that end with a rest-variable with each other.
    /// A helper method for `unify_lists`.
    #[allow(clippy::ptr_arg)]
    fn unify_two_lists_with_rest<F>(
        &mut self,
        rest_list_a: &TermList,
        rest_list_b: &TermList,
        mut unify: F,
    ) -> PolarResult<()>
    where
        F: FnMut((&Term, &Term)) -> Goal,
    {
        if rest_list_a.len() == rest_list_b.len() {
            let n = rest_list_b.len() - 1;
            let rest = unify((&rest_list_b[n].clone(), &rest_list_a[n].clone()));
            self.append_goals(
                rest_list_b
                    .iter()
                    .take(n)
                    .zip(rest_list_a)
                    .map(unify)
                    .chain(vec![rest]),
            )
        } else {
            let (shorter, longer) = {
                if rest_list_a.len() < rest_list_b.len() {
                    (rest_list_a, rest_list_b)
                } else {
                    (rest_list_b, rest_list_a)
                }
            };
            let n = shorter.len() - 1;
            let rest = unify((&shorter[n].clone(), &Term::from(longer[n..].to_vec())));
            self.append_goals(
                shorter
                    .iter()
                    .take(n)
                    .zip(longer)
                    .map(unify)
                    .chain(vec![rest]),
            )
        }
    }

    /// Unify a list that ends with a rest-variable with another that doesn't.
    /// A helper method for `unify_lists`.
    #[allow(clippy::ptr_arg)]
    fn unify_rest_list_with_list<F>(
        &mut self,
        rest_list: &TermList,
        list: &TermList,
        mut unify: F,
    ) -> PolarResult<()>
    where
        F: FnMut((&Term, &Term)) -> Goal,
    {
        let n = rest_list.len() - 1;
        if list.len() >= n {
            let rest = unify((&rest_list[n].clone(), &Term::from(list[n..].to_vec())));
            self.append_goals(
                rest_list
                    .iter()
                    .take(n)
                    .zip(list)
                    .map(unify)
                    .chain(vec![rest]),
            )
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

            if applicable_rules.is_empty() {
                self.log(LogLevel::Info, || "No matching rules found", &[]);
            }

            self.push_goal(Goal::SortRules {
                rules: applicable_rules.iter().rev().cloned().collect(),
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

            // The prefilter already checks applicability for ground rules.
            if rule.is_ground() {
                return self.push_goal(applicable);
            }

            // Rename the variables in the rule (but not the args).
            // This avoids clashes between arg vars and rule vars.
            let Rule { params, .. } = self.rename_rule_vars(&rule);
            let mut check_applicability = vec![];
            for (arg, param) in args.iter().zip(params.iter()) {
                check_applicability.push(Goal::Unify {
                    left: arg.clone(),
                    right: param.parameter.clone(),
                });
                if let Some(specializer) = &param.specializer {
                    check_applicability.push(Goal::Isa {
                        left: arg.clone(),
                        right: specializer.clone(),
                    });
                }
            }
            self.choose_conditional(check_applicability, vec![applicable], vec![inapplicable])?;
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
        } else if outer > rules.len() {
            return invalid_state("bad outer index");
        } else if inner > rules.len() {
            return invalid_state("bad inner index");
        } else if inner > outer {
            return invalid_state("bad insertion sort state");
        }

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
                self.choose_conditional(vec![compare], vec![next_inner], vec![next_outer])?;
            } else {
                if inner != 0 {
                    return invalid_state("inner == 0");
                }
                self.push_goal(next_outer)?;
            }
        } else {
            // We're done; the rules are sorted.
            // Make alternatives for calling them.

            self.polar_trace_mute = false;
            self.log(
                LogLevel::Info,
                || {
                    let mut rule_strs = "APPLICABLE_RULES:".to_owned();
                    for rule in rules {
                        let context = rule
                            .parsed_context()
                            .map_or_else(|| "".into(), Context::source_position);

                        write!(rule_strs, "\n  {}{}", rule.head_as_string(), context).unwrap();
                    }
                    rule_strs
                },
                &[],
            );

            let mut alternatives = Vec::with_capacity(rules.len());
            for rule in rules.iter() {
                let mut goals = Vec::with_capacity(2 * args.len() + 4);
                goals.push(Goal::TraceRule {
                    trace: Rc::new(Trace {
                        node: Node::Rule(rule.clone()),
                        children: vec![],
                    }),
                });
                goals.push(Goal::TraceStackPush);
                let Rule { body, params, .. } = self.rename_rule_vars(rule);

                // Unify the arguments with the formal parameters.
                for (arg, param) in args.iter().zip(params.iter()) {
                    goals.push(Goal::Unify {
                        left: arg.clone(),
                        right: param.parameter.clone(),
                    });
                    if let Some(specializer) = &param.specializer {
                        goals.push(Goal::Isa {
                            left: param.parameter.clone(),
                            right: specializer.clone(),
                        });
                    }
                }

                // Query for the body clauses.
                goals.push(Goal::Query { term: body.clone() });
                goals.push(Goal::TraceStackPop);

                alternatives.push(goals)
            }

            // Choose the first alternative, and push a choice for the rest.
            self.choose(alternatives)?;
        }
        Ok(())
    }

    /// Succeed if `left` is more specific than `right` with respect to `args`.
    #[allow(clippy::ptr_arg, clippy::wrong_self_convention)]
    fn is_more_specific(&mut self, left: &Rule, right: &Rule, args: &TermList) -> PolarResult<()> {
        let zipped = left.params.iter().zip(right.params.iter()).zip(args.iter());
        for ((left_param, right_param), arg) in zipped {
            match (&left_param.specializer, &right_param.specializer) {
                // If both specs are unions, they have the same specificity regardless of whether
                // they're the same or different unions.
                //
                // TODO(gj): when we have unions beyond `Actor` and `Resource`, we'll need to be
                // smarter about this check since UnionA is more specific than UnionB if UnionA is
                // a member of UnionB.
                (Some(left_spec), Some(right_spec))
                    if self.kb.read().unwrap().is_union(left_spec)
                        && self.kb.read().unwrap().is_union(right_spec) => {}
                // If left is a union and right is not, left cannot be more specific, so we
                // backtrack.
                (Some(left_spec), Some(_)) if self.kb.read().unwrap().is_union(left_spec) => {
                    return self.push_goal(Goal::Backtrack)
                }
                // If right is a union and left is not, left IS more specific, so we return.
                (Some(_), Some(right_spec)) if self.kb.read().unwrap().is_union(right_spec) => {
                    return Ok(())
                }

                (Some(left_spec), Some(right_spec)) => {
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
                        self.bind(&answer, Term::from(false)).unwrap();

                        return self.append_goals(vec![
                            Goal::IsSubspecializer {
                                answer: answer.clone(),
                                left: left_spec.clone(),
                                right: right_spec.clone(),
                                arg: arg.clone(),
                            },
                            Goal::Unify {
                                left: Term::from(answer),
                                right: Term::from(true),
                            },
                        ]);
                    }
                }
                // If the left rule has no specializer and the right does, it is NOT more specific,
                // so we Backtrack (fail)
                (None, Some(_)) => return self.push_goal(Goal::Backtrack),
                // If the left rule has a specializer and the right does not, the left IS more specific,
                // so we return
                (Some(_), None) => return Ok(()),
                // If neither has a specializer, neither is more specific, so we continue to the next argument.
                (None, None) => (),
            }
        }
        // Fail on any of the above branches that do not return
        self.push_goal(Goal::Backtrack)
    }

    /// Determine if `left` is a more specific specializer ("subspecializer") than `right`
    #[allow(clippy::wrong_self_convention)]
    fn is_subspecializer(
        &mut self,
        answer: &Symbol,
        left: &Term,
        right: &Term,
        arg: &Term,
    ) -> PolarResult<QueryEvent> {
        let arg = self.deref(arg);
        match (arg.value(), left.value(), right.value()) {
            (
                Value::ExternalInstance(instance),
                Value::Pattern(Pattern::Instance(left_lit)),
                Value::Pattern(Pattern::Instance(right_lit)),
            ) => {
                let call_id = self.new_call_id(answer);
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
                    self.rebind_external_answer(
                        answer,
                        Term::from(right_fields.len() < left.fields.len()),
                    );
                }
                Ok(QueryEvent::None)
            }
            (_, Value::Pattern(Pattern::Instance(_)), Value::Pattern(Pattern::Dictionary(_))) => {
                self.rebind_external_answer(answer, Term::from(true));
                Ok(QueryEvent::None)
            }
            _ => {
                self.rebind_external_answer(answer, Term::from(false));
                Ok(QueryEvent::None)
            }
        }
    }

    pub fn term_source(&self, term: &Term, include_info: bool) -> String {
        let source_info = term.parsed_context();

        let mut source_string = if let Some(context) = source_info {
            let chars = context.source.src.chars();
            chars.take(context.right).skip(context.left).collect()
        } else {
            term.to_string()
        };

        if include_info {
            if let Some(context) = source_info {
                source_string += &context.source_position();
            }
        }

        source_string
    }

    fn type_error<T>(&self, term: &Term, msg: String) -> PolarResult<T> {
        Err(RuntimeError::TypeError {
            msg,
            stack_trace: self.stack_trace(),
            term: term.clone(),
        }
        .into())
    }

    fn run_runnable(&mut self, runnable: Box<dyn Runnable>) -> PolarResult<QueryEvent> {
        let (call_id, answer) = self.new_call_var("runnable_result", Value::Boolean(false));
        self.push_goal(Goal::Unify {
            left: answer,
            right: Term::from(true),
        })?;

        Ok(QueryEvent::Run { runnable, call_id })
    }

    /// Handle an error coming from outside the vm.
    pub fn external_error(&mut self, message: String) -> PolarResult<()> {
        self.external_error = Some(message);
        Ok(())
    }
}

impl Runnable for PolarVirtualMachine {
    /// Run the virtual machine. While there are goals on the stack,
    /// pop them off and execute them one at a time until we have a
    /// `QueryEvent` to return. May be called multiple times to restart
    /// the machine.
    fn run(&mut self, _: Option<&mut Counter>) -> PolarResult<QueryEvent> {
        if self.query_start_time.is_none() {
            #[cfg(not(target_arch = "wasm32"))]
            let query_start_time = Some(std::time::Instant::now());
            #[cfg(target_arch = "wasm32")]
            let query_start_time = Some(js_sys::Date::now());
            self.query_start_time = query_start_time;
        }

        if self.goals.is_empty() {
            if self.choices.is_empty() {
                return Ok(QueryEvent::Done { result: true });
            } else {
                self.backtrack()?;
            }
        }

        while let Some(goal) = self.goals.pop() {
            match self.next(goal.clone())? {
                QueryEvent::None => (),
                event => {
                    self.external_error = None;
                    return Ok(event);
                }
            }
            self.maybe_break(DebugEvent::Goal(goal.clone()))?;
        }

        if self.tracing {
            for t in &self.trace {
                self.log(LogLevel::Trace, || format!("trace\n{}", t.draw(self)), &[]);
            }
        }

        let trace = if self.tracing {
            let trace = self.trace.first().cloned();
            trace.map(|trace| TraceResult {
                formatted: trace.draw(self),
                trace,
            })
        } else {
            None
        };

        let mut bindings = self.bindings(true);
        if !self.inverting {
            match simplify_bindings_opt(bindings, false) {
                Ok(Some(bs)) => {
                    // simplification succeeds
                    bindings = bs;
                }
                Ok(None) => {
                    // incompatible bindings; simplification fails
                    // do not return result
                    return Ok(QueryEvent::None);
                }

                Err(RuntimeError::UnhandledPartial { term, ref var }) => {
                    // use the debugger to get the nicest possible version of this binding
                    let Binding(original_var_name, simplified) = get_binding_for_var(&var.0, self);

                    // TODO(gj): `t` is a partial constructed in the VM, so we don't have any
                    // source context for it. We make a best effort to track down some relevant
                    // context by walking `t` in search of the first piece of source context we
                    // find.
                    //
                    // For a future refactor, we might consider using the `Term::clone_with_value`
                    // API to preserve source context when initially binding a variable to an
                    // `Expression`.
                    fn try_to_add_context(t: &Term, simplified: Term) -> Term {
                        /// `GetSource` walks a term & returns the _1st_ piece of source info it finds.
                        struct GetSource {
                            term: Option<Term>,
                        }

                        impl Visitor for GetSource {
                            fn visit_term(&mut self, t: &Term) {
                                if self.term.is_none() {
                                    if t.parsed_context().is_none() {
                                        walk_term(self, t)
                                    } else {
                                        self.term = Some(t.clone())
                                    }
                                }
                            }
                        }

                        let mut source_getter = GetSource { term: None };
                        source_getter.visit_term(t);
                        if let Some(term_with_context) = source_getter.term {
                            term_with_context.clone_with_value(simplified.value().clone())
                        } else {
                            simplified
                        }
                    }

                    // there was an unhandled partial in the bindings
                    // grab the context from the variable that was defined and
                    // set the context before returning
                    return Err(RuntimeError::UnhandledPartial {
                        term: try_to_add_context(&term, simplified),
                        var: original_var_name,
                    }
                    .into());
                }
                Err(e) => unreachable!("unexpected error: {}", e.to_string()),
            }

            bindings = bindings
                .clone()
                .into_iter()
                .filter(|(var, _)| !var.is_temporary_var())
                .map(|(var, value)| (var.clone(), sub_this(var, value)))
                .collect();
        }

        self.log(
            LogLevel::Info,
            || {
                if bindings.is_empty() {
                    "RESULT: SUCCESS".to_string()
                } else {
                    let mut out = "RESULT: {\n".to_string(); // open curly & newline
                    for (key, value) in &bindings {
                        writeln!(out, "  {}: {}", key, value).unwrap(); // key-value pairs spaced w/ newlines
                    }
                    out.push('}'); // closing curly
                    out
                }
            },
            &[],
        );

        Ok(QueryEvent::Result { bindings, trace })
    }

    fn handle_error(&mut self, error: PolarError) -> PolarResult<QueryEvent> {
        // if we pushed a debug goal, push an error goal underneath it.
        if self.maybe_break(DebugEvent::Error(error.clone()))? {
            let g = self.goals.pop().unwrap();
            self.push_goal(Goal::Error { error })?;
            self.goals.push(g);
            Ok(QueryEvent::None)
        } else {
            Err(error)
        }
    }

    /// Handle response to a predicate posed to the application, e.g., `ExternalIsa`.
    fn external_question_result(&mut self, call_id: u64, answer: bool) -> PolarResult<()> {
        let var = self.call_id_symbols.remove(&call_id).expect("bad call id");
        self.rebind_external_answer(&var, Term::from(answer));
        Ok(())
    }

    /// Handle an external result provided by the application.
    ///
    /// If the value is `Some(_)` then we have a result, and unify the
    /// symbol associated with the call ID to the result value. If the
    /// value is `None` then the external has no (more) results, so we
    /// backtrack to the choice point left by `Goal::LookupExternal`.
    fn external_call_result(&mut self, call_id: u64, term: Option<Term>) -> PolarResult<()> {
        // TODO: Open question if we need to pass errors back down to rust.
        // For example what happens if the call asked for a field that doesn't exist?

        if let Some(value) = term {
            self.log(LogLevel::Trace, || format!("=> {}", value), &[]);

            // Fetch variable to unify with call result.
            let sym = self.get_call_sym(call_id).to_owned();

            self.push_goal(Goal::Unify {
                left: Term::from(sym),
                right: value,
            })?;
        } else {
            self.log(LogLevel::Trace, || "=> No more results.", &[]);

            // No more results. Clean up, cut out the retry alternative,
            // and backtrack.
            self.call_id_symbols.remove(&call_id).expect("bad call ID");

            let check_error = if let Some(goal) = self.goals.last() {
                matches!(*(*goal), Goal::CheckError)
            } else {
                false
            };

            self.push_goal(Goal::Backtrack)?;
            self.push_goal(Goal::Cut {
                choice_index: self.choices.len() - 1,
            })?;

            if check_error {
                self.push_goal(Goal::CheckError)?;
            }
        }
        Ok(())
    }

    /// Drive debugger.
    fn debug_command(&mut self, command: &str) -> PolarResult<()> {
        let mut debugger = self.debugger.clone();
        let maybe_goal = debugger.debug_command(command, self);
        if let Some(goal) = maybe_goal {
            self.push_goal(goal)?;
        }
        self.debugger = debugger;
        Ok(())
    }

    fn clone_runnable(&self) -> Box<dyn Runnable> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {

    use permutohedron::Heap;

    use super::*;
    use crate::error::ErrorKind;
    use crate::rewrites::unwrap_and;

    impl PolarVirtualMachine {
        /// Return true if there is nothing left to do.
        fn is_halted(&self) -> bool {
            self.goals.is_empty() && self.choices.is_empty()
        }
    }

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
            assert!(matches!($vm.run(None).unwrap(), QueryEvent::Result{bindings, ..} if bindings == $result));
            assert_query_events!($vm, []);
        };
        ($vm:ident, [QueryEvent::Result{$result:expr}, $($tail:tt)*]) => {
            assert!(matches!($vm.run(None).unwrap(), QueryEvent::Result{bindings, ..} if bindings == $result));
            assert_query_events!($vm, [$($tail)*]);
        };
        ($vm:ident, [$( $pattern:pat_param )|+ $( if $guard: expr )?]) => {
            assert!(matches!($vm.run(None).unwrap(), $($pattern)|+ $(if $guard)?));
            assert_query_events!($vm, []);
        };
        ($vm:ident, [$( $pattern:pat_param )|+ $( if $guard: expr )?, $($tail:tt)*]) => {
            assert!(matches!($vm.run(None).unwrap(), $($pattern)|+ $(if $guard)?));
            assert_query_events!($vm, [$($tail)*]);
        };
        // TODO (dhatch) Be able to use btreemap! to match on specific bindings.
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn and_expression() {
        let f1 = rule!("f", [1]);
        let f2 = rule!("f", [2]);

        let rule = GenericRule::new(sym!("f"), vec![Arc::new(f1), Arc::new(f2)]);

        let mut kb = KnowledgeBase::new();
        kb.add_generic_rule(rule);

        let goal = query!(op!(And));

        let mut vm = PolarVirtualMachine::new_test(Arc::new(RwLock::new(kb)), false, vec![goal]);
        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!()},
            QueryEvent::Done { result: true }
        ]);

        assert!(vm.is_halted());

        let f1 = term!(call!("f", [1]));
        let f2 = term!(call!("f", [2]));
        let f3 = term!(call!("f", [3]));

        // Querying for f(1)
        vm.push_goal(query!(op!(And, f1.clone()))).unwrap();

        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{}},
            QueryEvent::Done { result: true }
        ]);

        // Querying for f(1), f(2)
        vm.push_goal(query!(f1.clone(), f2.clone())).unwrap();
        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{}},
            QueryEvent::Done { result: true }
        ]);

        // Querying for f(3)
        vm.push_goal(query!(op!(And, f3.clone()))).unwrap();
        assert_query_events!(vm, [QueryEvent::Done { result: true }]);

        // Querying for f(1), f(2), f(3)
        let mut parts = vec![f1, f2, f3];
        for permutation in Heap::new(&mut parts) {
            vm.push_goal(Goal::Query {
                term: Term::new_from_test(Value::Expression(Operation {
                    operator: Operator::And,
                    args: permutation,
                })),
            })
            .unwrap();
            assert_query_events!(vm, [QueryEvent::Done { result: true }]);
        }
    }

    #[test]
    fn unify_expression() {
        let mut vm = PolarVirtualMachine::default();
        vm.push_goal(query!(op!(Unify, term!(1), term!(1))))
            .unwrap();

        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{}},
            QueryEvent::Done { result: true }
        ]);

        let q = op!(Unify, term!(1), term!(2));
        vm.push_goal(query!(q)).unwrap();

        assert_query_events!(vm, [QueryEvent::Done { result: true }]);
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
            matches!(vm.run(None).unwrap(), QueryEvent::Result{bindings, ..} if bindings.is_empty())
        );
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [1,2] isa [1,2]
        vm.push_goal(Goal::Isa {
            left: one_two_list.clone(),
            right: one_two_list.clone(),
        })
        .unwrap();
        assert!(
            matches!(vm.run(None).unwrap(), QueryEvent::Result{bindings, ..} if bindings.is_empty())
        );
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [1,2] isNOTa [2,1]
        vm.push_goal(Goal::Isa {
            left: one_two_list.clone(),
            right: two_one_list,
        })
        .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [1] isNOTa [1,2]
        vm.push_goal(Goal::Isa {
            left: one_list.clone(),
            right: one_two_list.clone(),
        })
        .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [1,2] isNOTa [1]
        vm.push_goal(Goal::Isa {
            left: one_two_list.clone(),
            right: one_list.clone(),
        })
        .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [1] isNOTa []
        vm.push_goal(Goal::Isa {
            left: one_list.clone(),
            right: empty_list.clone(),
        })
        .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [] isNOTa [1]
        vm.push_goal(Goal::Isa {
            left: empty_list,
            right: one_list.clone(),
        })
        .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [1] isNOTa 1
        vm.push_goal(Goal::Isa {
            left: one_list.clone(),
            right: one.clone(),
        })
        .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // 1 isNOTa [1]
        vm.push_goal(Goal::Isa {
            left: one,
            right: one_list,
        })
        .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [1,2] isa [1, *rest]
        vm.push_goal(Goal::Isa {
            left: one_two_list,
            right: term!([1, Value::RestVariable(sym!("rest"))]),
        })
        .unwrap();
        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{sym!("rest") => term!([2])}},
            QueryEvent::Done { result: true }
        ]);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn isa_on_dicts() {
        let mut vm = PolarVirtualMachine::default();
        let dict = term!(btreemap! {
            sym!("x") => term!(1),
            sym!("y") => term!(2),
        });
        let dict_pattern = term!(pattern!(btreemap! {
            sym!("x") => term!(1),
            sym!("y") => term!(2),
        }));
        vm.push_goal(Goal::Isa {
            left: dict.clone(),
            right: dict_pattern.clone(),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done { result: true }]);

        // Dicts with identical keys and different values DO NOT isa.
        let different_dict_pattern = term!(pattern!(btreemap! {
            sym!("x") => term!(2),
            sym!("y") => term!(1),
        }));
        vm.push_goal(Goal::Isa {
            left: dict.clone(),
            right: different_dict_pattern,
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Done { result: true }]);

        let empty_dict = term!(btreemap! {});
        let empty_dict_pattern = term!(pattern!(btreemap! {}));
        // {} isa {}.
        vm.push_goal(Goal::Isa {
            left: empty_dict.clone(),
            right: empty_dict_pattern.clone(),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done { result: true }]);

        // Non-empty dicts should isa against an empty dict.
        vm.push_goal(Goal::Isa {
            left: dict.clone(),
            right: empty_dict_pattern,
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done { result: true }]);

        // Empty dicts should NOT isa against a non-empty dict.
        vm.push_goal(Goal::Isa {
            left: empty_dict,
            right: dict_pattern.clone(),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Done { result: true }]);

        let subset_dict_pattern = term!(pattern!(btreemap! {sym!("x") => term!(1)}));
        // Superset dict isa subset dict.
        vm.push_goal(Goal::Isa {
            left: dict,
            right: subset_dict_pattern,
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done { result: true }]);

        // Subset dict isNOTa superset dict.
        let subset_dict = term!(btreemap! {sym!("x") => term!(1)});
        vm.push_goal(Goal::Isa {
            left: subset_dict,
            right: dict_pattern,
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Done { result: true }]);
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
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done { result: true }]);

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
        assert_query_events!(vm, [QueryEvent::Done { result: true }]);

        // Empty dicts unify.
        vm.push_goal(Goal::Unify {
            left: term!(btreemap! {}),
            right: term!(btreemap! {}),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done { result: true }]);

        // Empty dict should not unify against a non-empty dict.
        vm.push_goal(Goal::Unify {
            left: left.clone(),
            right: term!(btreemap! {}),
        })
        .unwrap();
        assert_query_events!(vm, [QueryEvent::Done { result: true }]);

        // Subset match should fail.
        let right = term!(btreemap! {
            sym!("x") => term!(1),
        });
        vm.push_goal(Goal::Unify { left, right }).unwrap();
        assert_query_events!(vm, [QueryEvent::Done { result: true }]);
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
        assert_query_events!(vm, [QueryEvent::Result { hashmap!{sym!("result") => term!(1)} }, QueryEvent::Done { result: true }]);
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
            field: term!(string!("x")),
            value: term!(1),
        })
        .unwrap();

        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{}}
        ]);

        // Lookup with incorrect value
        vm.push_goal(Goal::Lookup {
            dict: dict.clone(),
            field: term!(string!("x")),
            value: term!(2),
        })
        .unwrap();

        assert_query_events!(vm, [QueryEvent::Done { result: true }]);

        // Lookup with unbound value
        vm.push_goal(Goal::Lookup {
            dict,
            field: term!(string!("x")),
            value: term!(sym!("y")),
        })
        .unwrap();
        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{sym!("y") => term!(1)}}
        ]);
    }

    #[test]
    fn debug() {
        let mut vm = PolarVirtualMachine::new_test(
            Arc::new(RwLock::new(KnowledgeBase::new())),
            false,
            vec![Goal::Debug {
                message: "Hello".to_string(),
            }],
        );
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Debug { message } if &message[..] == "Hello"
        ));
    }

    #[test]
    fn halt() {
        let mut vm = PolarVirtualMachine::new_test(
            Arc::new(RwLock::new(KnowledgeBase::new())),
            false,
            vec![Goal::Halt],
        );
        let _ = vm.run(None).unwrap();
        assert_eq!(vm.goals.len(), 0);
        assert_eq!(vm.bindings(true).len(), 0);
    }

    #[test]
    fn unify() {
        let x = sym!("x");
        let y = sym!("y");
        let vars = term!([x.clone(), y.clone()]);
        let zero = value!(0);
        let one = value!(1);
        let vals = term!([zero.clone(), one.clone()]);
        let mut vm = PolarVirtualMachine::new_test(
            Arc::new(RwLock::new(KnowledgeBase::new())),
            false,
            vec![Goal::Unify {
                left: vars,
                right: vals,
            }],
        );
        let _ = vm.run(None).unwrap();
        assert_eq!(vm.variable_state(&x), VariableState::Bound(term!(zero)));
        assert_eq!(vm.variable_state(&y), VariableState::Bound(term!(one)));
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
        vm.bind(&y, one.clone()).unwrap();
        vm.append_goals(vec![Goal::Unify {
            left: term!(x.clone()),
            right: term!(y),
        }])
        .unwrap();
        let _ = vm.run(None).unwrap();
        assert_eq!(vm.deref(&term!(x)), one);
        vm.backtrack().unwrap();

        // Left variable bound to value.
        vm.bind(&z, one.clone()).unwrap();
        vm.append_goals(vec![Goal::Unify {
            left: term!(z.clone()),
            right: one.clone(),
        }])
        .unwrap();
        let _ = vm.run(None).unwrap();
        assert_eq!(vm.deref(&term!(z.clone())), one);

        // Left variable bound to value, unify with something else, backtrack.
        vm.append_goals(vec![Goal::Unify {
            left: term!(z.clone()),
            right: two,
        }])
        .unwrap();
        let _ = vm.run(None).unwrap();
        assert_eq!(vm.deref(&term!(z)), one);
    }

    #[test]
    fn test_gen_var() {
        let vm = PolarVirtualMachine::default();

        let rule = Rule::new_from_test(
            Symbol::new("foo"),
            vec![],
            Term::new_from_test(Value::Expression(Operation {
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
        );

        let renamed_rule = vm.rename_rule_vars(&rule);
        let renamed_terms = unwrap_and(&renamed_rule.body);
        assert_eq!(renamed_terms[1].value(), renamed_terms[2].value());
        let x_value = match &renamed_terms[1].value() {
            Value::Variable(sym) => Some(sym.0.clone()),
            _ => None,
        };
        assert_eq!(x_value.unwrap(), "_x_1");

        let y_value = match &renamed_terms[3].value() {
            Value::List(terms) => match &terms[0].value() {
                Value::Variable(sym) => Some(sym.0.clone()),
                _ => None,
            },
            _ => None,
        };
        assert_eq!(y_value.unwrap(), "_y_2");
    }

    #[test]
    fn test_filter_rules() {
        let rule_a = Arc::new(rule!("bar", ["_"; instance!("a")]));
        let rule_b = Arc::new(rule!("bar", ["_"; instance!("b")]));
        let rule_1a = Arc::new(rule!("bar", [value!(1)]));
        let rule_1b = Arc::new(rule!("bar", ["_"; value!(1)]));

        let gen_rule = GenericRule::new(sym!("bar"), vec![rule_a, rule_b, rule_1a, rule_1b]);
        let mut kb = KnowledgeBase::new();
        kb.add_generic_rule(gen_rule);

        let kb = Arc::new(RwLock::new(kb));

        let external_instance = Value::ExternalInstance(ExternalInstance {
            instance_id: 1,
            constructor: None,
            repr: None,
            class_repr: None,
            class_id: None,
        });
        let query = query!(call!("bar", [sym!("x")]));
        let mut vm = PolarVirtualMachine::new_test(kb.clone(), false, vec![query]);
        vm.bind(&sym!("x"), Term::new_from_test(external_instance))
            .unwrap();

        let mut external_isas = vec![];

        loop {
            match vm.run(None).unwrap() {
                QueryEvent::Done { .. } => break,
                QueryEvent::ExternalIsa {
                    call_id, class_tag, ..
                } => {
                    external_isas.push(class_tag.clone());
                    // Return `true` if the specified `class_tag` is `"a"`.
                    vm.external_question_result(call_id, class_tag.0 == "a")
                        .unwrap()
                }
                QueryEvent::ExternalOp { .. }
                | QueryEvent::ExternalIsSubSpecializer { .. }
                | QueryEvent::Result { .. } => (),
                e => panic!("Unexpected event: {:?}", e),
            }
        }

        let expected = vec![sym!("b"), sym!("a"), sym!("a")];
        assert_eq!(external_isas, expected);

        let query = query!(call!("bar", [sym!("x")]));
        let mut vm = PolarVirtualMachine::new_test(kb, false, vec![query]);
        vm.bind(&sym!("x"), Term::new_from_test(value!(1))).unwrap();

        let mut results = vec![];
        loop {
            match vm.run(None).unwrap() {
                QueryEvent::Done { .. } => break,
                QueryEvent::ExternalIsa { .. } => (),
                QueryEvent::Result { bindings, .. } => results.push(bindings),
                _ => panic!("Unexpected event"),
            }
        }

        assert_eq!(results.len(), 2);
        assert_eq!(
            results,
            vec![
                hashmap! {sym!("x") => term!(1)},
                hashmap! {sym!("x") => term!(1)},
            ]
        );
    }

    #[test]
    fn test_sort_rules() {
        // Test sort rule by mocking ExternalIsSubSpecializer and ExternalIsa.
        let bar_rule = GenericRule::new(
            sym!("bar"),
            vec![
                Arc::new(rule!("bar", ["_"; instance!("b"), "_"; instance!("a"), value!(3)])),
                Arc::new(rule!("bar", ["_"; instance!("a"), "_"; instance!("a"), value!(1)])),
                Arc::new(rule!("bar", ["_"; instance!("a"), "_"; instance!("b"), value!(2)])),
                Arc::new(rule!("bar", ["_"; instance!("b"), "_"; instance!("b"), value!(4)])),
            ],
        );

        let mut kb = KnowledgeBase::new();
        kb.add_generic_rule(bar_rule);

        let external_instance = Value::ExternalInstance(ExternalInstance {
            instance_id: 1,
            constructor: None,
            repr: None,
            class_repr: None,
            class_id: None,
        });

        let mut vm = PolarVirtualMachine::new_test(
            Arc::new(RwLock::new(kb)),
            false,
            vec![query!(call!(
                "bar",
                [external_instance.clone(), external_instance, sym!("z")]
            ))],
        );

        let mut results = Vec::new();
        loop {
            match vm.run(None).unwrap() {
                QueryEvent::Done { .. } => break,
                QueryEvent::Result { bindings, .. } => results.push(bindings),
                QueryEvent::ExternalIsSubSpecializer {
                    call_id,
                    left_class_tag,
                    right_class_tag,
                    ..
                } => {
                    // For this test we sort classes lexically.
                    vm.external_question_result(call_id, left_class_tag < right_class_tag)
                        .unwrap()
                }
                QueryEvent::MakeExternal { .. } => (),

                QueryEvent::ExternalOp {
                    operator: Operator::Eq,
                    call_id,
                    ..
                } => vm.external_question_result(call_id, true).unwrap(),

                QueryEvent::ExternalIsa { call_id, .. } => {
                    // For this test, anything is anything.
                    vm.external_question_result(call_id, true).unwrap()
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
            constructor: None,
            repr: None,
            class_repr: None,
            class_id: None,
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

    #[test]
    fn test_timeout() {
        let vm = PolarVirtualMachine::default();
        assert!(vm.query_timeout_ms == DEFAULT_TIMEOUT_MS);

        std::env::set_var("POLAR_TIMEOUT_MS", "0");
        let vm = PolarVirtualMachine::default();
        std::env::remove_var("POLAR_TIMEOUT_MS");
        assert!(vm.is_query_timeout_disabled());

        std::env::set_var("POLAR_TIMEOUT_MS", "500");
        let mut vm = PolarVirtualMachine::default();
        std::env::remove_var("POLAR_TIMEOUT_MS");
        // Turn this off so we don't hit it.
        vm.set_stack_limit(std::usize::MAX);

        loop {
            vm.push_goal(Goal::Noop).unwrap();
            vm.push_goal(Goal::MakeExternal {
                constructor: Term::from(true),
                instance_id: 1,
            })
            .unwrap();
            let result = vm.run(None);
            match result {
                Ok(event) => assert!(matches!(event, QueryEvent::MakeExternal { .. })),
                Err(err) => {
                    assert!(matches!(
                        err.0,
                        ErrorKind::Runtime(RuntimeError::QueryTimeout { .. })
                    ));

                    // End test.
                    break;
                }
            }
        }
    }

    #[test]
    fn test_prefiltering() {
        let bar_rule = GenericRule::new(
            sym!("bar"),
            vec![
                Arc::new(rule!("bar", [value!([1])])),
                Arc::new(rule!("bar", [value!([2])])),
            ],
        );

        let mut kb = KnowledgeBase::new();
        kb.add_generic_rule(bar_rule);

        let mut vm = PolarVirtualMachine::new_test(Arc::new(RwLock::new(kb)), false, vec![]);
        vm.bind(&sym!("x"), term!(1)).unwrap();
        let _ = vm.run(None);
        let _ = vm.next(Rc::new(query!(call!("bar", [value!([sym!("x")])]))));
        // After calling the query goal we should be left with the
        // prefiltered rules
        let next_goal = vm
            .goals
            .iter()
            .find(|g| matches!(g.as_ref(), Goal::FilterRules { .. }))
            .unwrap();
        let goal_debug = format!("{:#?}", next_goal);
        assert!(
            matches!(next_goal.as_ref(), Goal::FilterRules {
            ref applicable_rules, ref unfiltered_rules, ..
        } if unfiltered_rules.len() == 1 && applicable_rules.is_empty()),
            "Goal should contain just one prefiltered rule: {}",
            goal_debug
        );
    }

    #[test]
    fn choose_conditional() {
        let mut vm = PolarVirtualMachine::new_test(
            Arc::new(RwLock::new(KnowledgeBase::new())),
            false,
            vec![],
        );
        let consequent = Goal::Debug {
            message: "consequent".to_string(),
        };
        let alternative = Goal::Debug {
            message: "alternative".to_string(),
        };

        // Check consequent path when conditional succeeds.
        vm.choose_conditional(
            vec![Goal::Noop],
            vec![consequent.clone()],
            vec![alternative.clone()],
        )
        .unwrap();
        assert_query_events!(vm, [
            QueryEvent::Debug { message } if &message[..] == "consequent" && vm.is_halted(),
            QueryEvent::Done { result: true }
        ]);

        // Check alternative path when conditional fails.
        vm.choose_conditional(
            vec![Goal::Backtrack],
            vec![consequent.clone()],
            vec![alternative.clone()],
        )
        .unwrap();
        assert_query_events!(vm, [
            QueryEvent::Debug { message } if &message[..] == "alternative" && vm.is_halted(),
            QueryEvent::Done { result: true }
        ]);

        // Ensure bindings are cleaned up after conditional.
        vm.choose_conditional(
            vec![
                Goal::Unify {
                    left: term!(sym!("x")),
                    right: term!(true),
                },
                query!(sym!("x")),
            ],
            vec![consequent],
            vec![alternative],
        )
        .unwrap();
        assert_query_events!(vm, [
            QueryEvent::Debug { message } if &message[..] == "consequent" && vm.bindings(true).is_empty() && vm.is_halted(),
            QueryEvent::Done { result: true }
        ]);
    }

    #[test]
    fn test_log_level_should_print_for_level() {
        use LogLevel::*;

        // TRACE
        assert!(Trace.should_print_on_level(Trace));
        assert!(Trace.should_print_on_level(Debug));
        assert!(Trace.should_print_on_level(Info));

        // DEBUG
        assert!(!Debug.should_print_on_level(Trace));
        assert!(Debug.should_print_on_level(Debug));
        assert!(Debug.should_print_on_level(Info));

        // INFO
        assert!(!Info.should_print_on_level(Trace));
        assert!(!Info.should_print_on_level(Debug));
        assert!(Info.should_print_on_level(Info));
    }
}
