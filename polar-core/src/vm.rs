use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::rc::Rc;
use std::string::ToString;
use std::sync::{Arc, RwLock};

use super::debugger::{DebugEvent, Debugger};
use super::error::{self, PolarResult};
use super::events::*;
use super::formatting::ToPolarString;
use super::kb::*;
use super::lexer::loc_to_pos;
use super::messages::*;
use super::numerics::*;
use super::rules::*;
use super::sources::*;
use super::terms::*;
use super::traces::*;

pub const MAX_STACK_SIZE: usize = 10_000;
#[cfg(not(target_arch = "wasm32"))]
pub const QUERY_TIMEOUT_S: std::time::Duration = std::time::Duration::from_secs(30);
#[cfg(target_arch = "wasm32")]
pub const QUERY_TIMEOUT_S: f64 = 30_000.0;

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
        check_errors: bool,
    },
    MakeExternal {
        constructor: Term,
        instance_id: u64,
    },
    IsaExternal {
        instance: Term,
        literal: InstanceLiteral,
    },
    UnifyExternal {
        left_instance_id: u64,
        right_instance_id: u64,
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
    trace_stack: TraceStack,
}

pub type BindingStack = Vec<Binding>;
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

pub struct PolarVirtualMachine {
    /// Stacks.
    pub goals: GoalStack,
    pub bindings: BindingStack,
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
    #[cfg(not(target_arch = "wasm32"))]
    query_timeout: std::time::Duration,
    #[cfg(target_arch = "wasm32")]
    query_timeout: f64,

    /// Maximum size of goal stack
    stack_limit: usize,

    /// Binding stack constant below here.
    csp: usize,

    /// Interactive debugger.
    pub debugger: Debugger,

    /// Rules and types.
    pub kb: Arc<RwLock<KnowledgeBase>>,

    /// Call ID -> result variable name table.
    call_id_symbols: HashMap<u64, Symbol>,

    /// Logging flag.
    log: bool,
    polar_log: bool,
    polar_log_mute: bool,

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

// Methods which aren't goals/instructions.
impl PolarVirtualMachine {
    /// Make a new virtual machine with an initial list of goals.
    /// Reverse the goal list for the sanity of callers.
    pub fn new(
        kb: Arc<RwLock<KnowledgeBase>>,
        trace: bool,
        goals: Goals,
        messages: MessageQueue,
    ) -> Self {
        let constants = kb
            .read()
            .expect("cannot acquire KB read lock")
            .constants
            .clone();
        let mut vm = Self {
            goals: GoalStack::new_reversed(goals),
            bindings: vec![],
            query_start_time: None,
            query_timeout: QUERY_TIMEOUT_S,
            stack_limit: MAX_STACK_SIZE,
            csp: 0,
            choices: vec![],
            queries: vec![],
            tracing: trace,
            trace_stack: vec![],
            trace: vec![],
            external_error: None,
            debugger: Debugger::default(),
            kb,
            call_id_symbols: HashMap::new(),
            log: std::env::var("RUST_LOG").is_ok(),
            polar_log: std::env::var("POLAR_LOG").is_ok(),
            polar_log_mute: false,
            messages,
        };
        vm.bind_constants(constants);
        vm
    }

    pub fn new_test(kb: Arc<RwLock<KnowledgeBase>>, trace: bool, goals: Goals) -> Self {
        PolarVirtualMachine::new(kb, trace, goals, MessageQueue::new())
    }

    #[cfg(test)]
    fn set_stack_limit(&mut self, limit: usize) {
        self.stack_limit = limit;
    }

    #[cfg(test)]
    fn set_query_timeout(&mut self, timeout_s: u64) {
        self.query_timeout = std::time::Duration::from_secs(timeout_s);
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
            self.print(&format!("{}", goal));
        }

        self.check_timeout()?;

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
                check_errors,
            } => return self.lookup_external(*call_id, instance, field, *check_errors),
            Goal::IsaExternal { instance, literal } => return self.isa_external(instance, literal),
            Goal::UnifyExternal {
                left_instance_id,
                right_instance_id,
            } => return self.unify_external(*left_instance_id, *right_instance_id),
            Goal::MakeExternal {
                constructor,
                instance_id,
            } => return Ok(self.make_external(constructor, *instance_id)),
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
            Goal::TracePush => {
                self.trace_stack.push(Rc::new(self.trace.clone()));
                self.trace = vec![];
            }
            Goal::TracePop => {
                let mut children = self.trace.clone();
                self.trace = self.trace_stack.pop().unwrap().as_ref().clone();
                let mut trace = self.trace.pop().unwrap();
                let trace = Rc::make_mut(&mut trace);
                trace.children.append(&mut children);
                self.trace.push(Rc::new(trace.clone()));
            }
            Goal::TraceRule { trace } => {
                if let Node::Rule(rule) = &trace.node {
                    self.log_with(
                        || {
                            let source_str = self.rule_source(&rule);
                            format!("RULE: {}", source_str)
                        },
                        &[],
                    );
                }
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
        if self.query_start_time.is_none() {
            #[cfg(not(target_arch = "wasm32"))]
            let query_start_time = Some(std::time::Instant::now());
            #[cfg(target_arch = "wasm32")]
            let query_start_time = Some(js_sys::Date::now());
            self.query_start_time = query_start_time;
        }

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
                event => {
                    self.external_error = None;
                    return Ok(event);
                }
            }
            self.maybe_break(DebugEvent::Goal(goal.clone()))?;
        }

        if self.log {
            self.print("⇒ result");
            if self.tracing {
                for t in &self.trace {
                    self.print(&format!("trace\n{}", t.draw(&self)));
                }
            }
        }

        let trace = if self.tracing {
            let trace = self.trace.first().cloned();
            trace.map(|trace| TraceResult {
                formatted: trace.draw(&self),
                trace,
            })
        } else {
            None
        };

        Ok(QueryEvent::Result {
            bindings: self.bindings(false),
            trace,
        })
    }

    /// Return true if there is nothing left to do.
    pub fn is_halted(&self) -> bool {
        self.goals.is_empty() && self.choices.is_empty()
    }

    /// Push a goal onto the goal stack.
    pub fn push_goal(&mut self, goal: Goal) -> PolarResult<()> {
        if self.goals.len() >= self.stack_limit {
            return Err(error::RuntimeError::StackOverflow {
                msg: format!("Goal stack overflow! MAX_GOALS = {}", self.stack_limit),
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
        assert!(self.choices.len() < self.stack_limit, "too many choices");
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
            Ok(())
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
        self.push_choice(vec![consequent]);
        let cut_alternative = Goal::Cut {
            choice_index: self.choices.len(),
        };
        conditional.push(cut_alternative);
        conditional.push(Goal::Backtrack);

        self.choose(vec![conditional, alternative])?;
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
            self.print(&format!(
                "⇒ bind: {} ← {}",
                var.to_polar(),
                value.to_polar()
            ));
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
            bindings.insert(var.clone(), self.deep_deref(value));
        }
        bindings
    }

    /// Retrieve the current non-constant bindings for symbols in variables.
    pub fn variable_bindings(&self, variables: &HashSet<Symbol>) -> Bindings {
        let mut bindings = HashMap::new();
        for Binding(var, value) in &self.bindings[self.csp..] {
            if !variables.contains(var) {
                continue;
            }
            bindings.insert(var.clone(), self.deep_deref(value));
        }
        bindings
    }

    /// Returns bindings for all vars used by terms in terms.
    pub fn relevant_bindings(&self, terms: &[&Term]) -> Bindings {
        let mut variables = HashSet::new();
        for t in terms {
            t.variables(&mut variables);
        }
        self.variable_bindings(&variables)
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

    pub fn deep_deref(&self, term: &Term) -> Term {
        term.cloned_map_replace(&mut |t| self.deref(t))
    }

    /// Recursively dereference a variable.
    pub fn deref(&self, term: &Term) -> Term {
        match &term.value() {
            Value::List(list) => {
                // Check if last element in list is a rest variable.
                let mut rest = false;
                if let Some(last) = list.last() {
                    if matches!(last.value(), Value::RestVariable(_)) {
                        rest = true;
                    }
                }
                // Deref all elements.
                let mut derefed: Vec<Term> = list.iter().map(|t| self.deref(t)).collect();
                // If last element was a rest variable, append the list it derefed to.
                if rest {
                    if let Some(last_term) = derefed.pop() {
                        if let Value::List(terms) = last_term.value() {
                            derefed.append(&mut terms.clone());
                        } else {
                            derefed.push(last_term);
                        }
                    }
                }
                term.clone_with_value(Value::List(derefed))
            }
            Value::Variable(symbol) | Value::RestVariable(symbol) => {
                self.value(&symbol).map_or(term.clone(), |t| {
                    if t == term {
                        t.clone()
                    } else {
                        self.deref(t)
                    }
                })
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

    /// Print a message to the output stream.
    fn print(&self, message: &str) {
        self.messages.push(MessageKind::Print, message.to_owned());
    }

    fn log(&self, message: &str, terms: &[&Term]) {
        self.log_with(|| message, terms)
    }

    fn log_with<F, R>(&self, message_fn: F, terms: &[&Term])
    where
        F: FnOnce() -> R,
        R: AsRef<str>,
    {
        if self.polar_log && !self.polar_log_mute {
            let mut indent = String::new();
            for _ in 0..=self.queries.len() {
                indent.push_str("  ");
            }
            let message = message_fn();
            let lines = message.as_ref().split('\n').collect::<Vec<&str>>();
            if let Some(line) = lines.first() {
                let mut msg = format!("[debug] {}{}", &indent, line);
                if !terms.is_empty() {
                    let relevant_bindings = self.relevant_bindings(terms);
                    msg.push_str(&format!(
                        ", BINDINGS: {{{}}}",
                        relevant_bindings
                            .iter()
                            .map(|(var, val)| format!("{} = {}", var.0, val.to_polar()))
                            .collect::<Vec<String>>()
                            .join(", ")
                    ));
                }
                self.messages.push(MessageKind::Print, msg);
                for line in &lines[1..] {
                    self.messages
                        .push(MessageKind::Print, format!("[debug] {}{}", &indent, line));
                }
            }
        }
    }

    pub fn source(&self, term: &Term) -> Option<Source> {
        term.get_source_id()
            .and_then(|id| self.kb.read().unwrap().sources.get_source(id))
    }

    /// Get the query stack as a string for printing in error messages.
    pub fn stack_trace(&self) -> String {
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

        let mut st = String::new();
        let _ = write!(st, "trace (most recent evaluation last):");

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
                    let _ = write!(st, "\n  ");

                    if let Some(source) = self.source(t) {
                        if let Some(rule) = &rule {
                            let _ = write!(st, "in rule {} ", rule.name.to_polar());
                        } else {
                            let _ = write!(st, "in query ");
                        }
                        let (row, column) = loc_to_pos(&source.src, t.offset());
                        let _ = write!(st, "at line {}, column {}", row + 1, column + 1);
                        if let Some(filename) = source.filename {
                            let _ = write!(st, " in file {}", filename);
                        }
                        let _ = writeln!(st);
                    };
                    let _ = write!(st, "    {}", self.term_source(t, false));
                }
            }
        }
        st
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn check_timeout(&self) -> PolarResult<()> {
        let now = std::time::Instant::now();
        let start_time = self
            .query_start_time
            .expect("Query start time not recorded");

        if now - start_time > self.query_timeout {
            return Err(error::RuntimeError::QueryTimeout {
                msg: format!(
                    "Query running for {}. Exceeded query timeout of {} seconds",
                    (now - start_time).as_secs(),
                    self.query_timeout.as_secs()
                ),
            }
            .into());
        }

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn check_timeout(&self) -> PolarResult<()> {
        let now = js_sys::Date::now();
        let start_time = self
            .query_start_time
            .expect("Query start time not recorded");

        if now - start_time > self.query_timeout {
            return Err(error::RuntimeError::QueryTimeout {
                msg: format!(
                    "Query running for {}. Exceeded query timeout of {} seconds",
                    (now - start_time) / 1_000.0,
                    self.query_timeout / 1_000.0
                ),
            }
            .into());
        }

        Ok(())
    }
}

/// Implementations of instructions.
impl PolarVirtualMachine {
    /// Remove all bindings after the last choice point, and try the
    /// next available alternative. If no choice is possible, halt.
    fn backtrack(&mut self) -> PolarResult<()> {
        if self.log {
            self.print("⇒ backtrack");
        }
        self.log("BACKTRACK", &[]);

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
                    self.bindings.truncate(bsp);
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
        let _ = self.choices.truncate(index);
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
    pub fn halt(&mut self) -> QueryEvent {
        self.log("HALT", &[]);
        self.goals.clear();
        self.choices.clear();
        assert!(self.is_halted());
        QueryEvent::Done
    }

    /// Comparison operator that essentially performs partial unification.
    pub fn isa(&mut self, left: &Term, right: &Term) -> PolarResult<()> {
        // TODO (dhatch): These errors could potentially be caused by the user.
        // rule(foo) if
        //    x = {a: 1} and
        //    foo matches x
        assert!(
            !matches!(&right.value(), Value::InstanceLiteral(_)),
            "Called isa with bare instance lit!"
        );
        assert!(
            !matches!(&right.value(), Value::Dictionary(_)),
            "Called isa with bare dictionary!"
        );

        self.log_with(
            || format!("MATCHES: {} matches {}", left.to_polar(), right.to_polar()),
            &[left, right],
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
                        field: right_value.clone_with_value(Value::String(field.0.clone())),
                        check_errors: false,
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

            (Value::RestVariable(symbol), _) => {
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

            (_, Value::RestVariable(symbol)) => {
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
                    instance: left.clone(),
                    literal: right_literal.clone(),
                })?;
            }

            (_, Value::Pattern(Pattern::Instance(right_literal))) => {
                self.push_goal(Goal::IsaExternal {
                    instance: left.clone(),
                    literal: right_literal.clone(),
                })?;
            }

            // Default case: x isa y if x = y.
            _ => self.push_goal(Goal::Unify {
                left: left.clone(),
                right: right.clone(),
            })?,
        }
        Ok(())
    }

    pub fn lookup(&mut self, dict: &Dictionary, field: &Term, value: &Term) -> PolarResult<()> {
        // check if field is a variable
        let field = self.deref(&field);
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
                self.choose(alternatives)?;
            }
            Value::String(field) => {
                if let Some(retrieved) = dict.fields.get(&Symbol(field.clone())) {
                    self.push_goal(Goal::Unify {
                        left: retrieved.clone(),
                        right: value.clone(),
                    })?;
                } else {
                    self.push_goal(Goal::Backtrack)?;
                }
            }
            v => {
                return Err(self.type_error(
                    &field,
                    format!("cannot look up field {:?} on a dictionary", v),
                ))
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
        check_errors: bool,
    ) -> PolarResult<QueryEvent> {
        let (field_name, args): (Symbol, Option<Vec<Term>>) = match self.deref(field).value() {
            Value::Call(Call {
                name,
                args,
                kwargs: None,
            }) => (
                name.clone(),
                Some(args.iter().map(|arg| self.deep_deref(arg)).collect()),
            ),
            Value::String(field) => (Symbol(field.clone()), None),
            v => {
                return Err(self.type_error(
                    &field,
                    format!("cannot look up field {:?} on an external instance", v),
                ))
            }
        };

        self.push_choice(vec![vec![Goal::LookupExternal {
            call_id,
            instance: instance.clone(),
            field: field.clone(),
            check_errors,
        }]]);

        if check_errors {
            self.push_goal(Goal::CheckError)?;
        }

        self.log_with(
            || {
                let mut msg = format!("LOOKUP: {}.{}", instance.to_string(), field_name);
                if let Some(arguments) = &args {
                    msg.push('(');
                    msg.push_str(
                        &arguments
                            .iter()
                            .map(|a| a.to_polar())
                            .collect::<Vec<String>>()
                            .join(", "),
                    );
                    msg.push(')');
                }
                msg
            },
            &[],
        );

        Ok(QueryEvent::ExternalCall {
            call_id,
            instance: self.deep_deref(instance),
            attribute: field_name,
            args,
        })
    }

    pub fn isa_external(
        &mut self,
        instance: &Term,
        literal: &InstanceLiteral,
    ) -> PolarResult<QueryEvent> {
        let result = self.kb.read().unwrap().gensym("isa");
        let call_id = self.new_call_id(&result);

        self.bind(&result, Term::new_temporary(Value::Boolean(false)));
        self.push_goal(Goal::Unify {
            left: Term::new_temporary(Value::Variable(result)),
            right: Term::new_temporary(Value::Boolean(true)),
        })?;

        Ok(QueryEvent::ExternalIsa {
            call_id,
            instance: self.deep_deref(instance),
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

        self.bind(&result, Term::new_temporary(Value::Boolean(false)));
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

    pub fn make_external(&self, constructor: &Term, instance_id: u64) -> QueryEvent {
        QueryEvent::MakeExternal {
            instance_id,
            constructor: self.deep_deref(&constructor),
        }
    }

    pub fn check_error(&self) -> PolarResult<QueryEvent> {
        if let Some(error) = &self.external_error {
            let trace = self.trace.last().unwrap();
            let term = match &trace.node {
                Node::Term(t) => Some(t),
                _ => None,
            };
            let stack_trace = self.stack_trace();
            let error = error::RuntimeError::Application {
                msg: error.clone(),
                stack_trace: Some(stack_trace),
            };
            if let Some(term) = term {
                Err(self.set_error_context(term, error))
            } else {
                Err(error.into())
            }
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
        // Don't log if it's just a single element AND like lots of rule bodies tend to be.
        match &term.value() {
            Value::Expression(Operation {
                operator: Operator::And,
                args,
            }) if args.len() < 2 => (),
            _ => {
                self.log_with(|| format!("QUERY: {}", term.to_polar()), &[term]);
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
    fn query_for_predicate(&mut self, predicate: Call) -> PolarResult<()> {
        assert!(predicate.kwargs.is_none());
        let goals = match self.kb.read().unwrap().rules.get(&predicate.name) {
            None => vec![Goal::Backtrack],
            Some(generic_rule) => {
                assert_eq!(generic_rule.name, predicate.name);

                // Pre-filter rules.
                let args = predicate.args.iter().map(|t| self.deep_deref(&t)).collect();
                let pre_filter = generic_rule.get_applicable_rules(&args);

                self.polar_log_mute = true;

                // Filter rules by applicability.
                vec![
                    Goal::TracePush,
                    Goal::FilterRules {
                        applicable_rules: vec![],
                        unfiltered_rules: pre_filter,
                        args: predicate.args,
                    },
                    Goal::TracePop,
                ]
            }
        };
        self.append_goals(goals)
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
            Operator::Assign => {
                assert_eq!(args.len(), 2);
                let right = args.pop().unwrap();
                let left = args.pop().unwrap();
                match (left.value(), right.value()) {
                    (Value::Variable(var), _) => match self.value(var) {
                        None => self.push_goal(Goal::Unify { left, right })?,
                        Some(value) => {
                            return Err(self.type_error( &left, format!("Can only assign to unbound variables, {} is bound to value {}.", var.to_polar(), value.to_polar())));
                        }
                    },
                    _ => {
                        return Err(self.type_error(
                            &left,
                            format!("Cannot assign to type {}.", left.to_polar()),
                        ))
                    }
                }
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
            op @ Operator::Add | op @ Operator::Sub | op @ Operator::Mul | op @ Operator::Div => {
                return self.arithmetic_op_helper(term, op, args);
            }
            Operator::In => {
                assert_eq!(args.len(), 2);
                let item = &args[0];
                let list = self.deref(&args[1]);
                match list.value() {
                    Value::List(list) if list.is_empty() => {
                        // Nothing is in an empty list.
                        self.backtrack()?;
                    }
                    Value::List(terms) => {
                        // Unify item with each element of the list, skipping non-matching ground terms.
                        let x = self.deref(item);
                        let v = x.value();
                        let g = v.is_ground();
                        self.choose(
                            terms
                                .iter()
                                .filter(|term| !g || !term.is_ground() || term.value() == v)
                                .map(|term| {
                                    vec![Goal::Unify {
                                        left: item.clone(),
                                        right: term.clone(),
                                    }]
                                })
                                .collect::<Vec<Goals>>(),
                        )?;
                    }
                    _ => {
                        return Err(self.type_error(
                            item,
                            format!("can only use `in` on a list, this is {:?}", item.value()),
                        ));
                    }
                }
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
            Operator::Print => {
                self.print(
                    &args
                        .iter()
                        .map(|arg| self.deref(arg).to_polar())
                        .collect::<Vec<String>>()
                        .join(", "),
                );
            }
            Operator::New => {
                assert_eq!(args.len(), 2);
                let result = args.pop().unwrap();
                assert!(
                    matches!(result.value(), Value::Variable(_)),
                    "Must have result variable as second arg."
                );
                let constructor = args.pop().unwrap();

                let instance_id = self.new_id();
                let instance =
                    constructor.clone_with_value(Value::ExternalInstance(ExternalInstance {
                        instance_id,
                        constructor: Some(constructor.clone()),
                        repr: Some(constructor.to_polar()),
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
            Err(self.type_error(
                &term,
                format!("can't query for: {}", term.value().to_polar()),
            ))
        }
    }

    /// Push appropriate goals for lookups on Dictionaries, InstanceLiterals, and ExternalInstances
    fn dot_op_helper(&mut self, mut args: Vec<Term>) -> PolarResult<()> {
        assert_eq!(args.len(), 3);
        let object = self.deref(&args[0]);
        let field = self.deref(&args[1]);
        let value = &args[2];

        match object.value() {
            // Push a `Lookup` goal for simple field lookups on dictionaries.
            Value::Dictionary(dict) if matches!(field.value(), Value::String(_) | Value::Variable(_)) => {
                self.push_goal(Goal::Lookup {
                    dict: dict.clone(),
                    field,
                    value: args.remove(2),
                })?
            }
            // Push an `ExternalLookup` goal for external instances and built-ins.
            Value::Dictionary(_)
            | Value::ExternalInstance(_)
            | Value::List(_)
            | Value::Number(_)
            | Value::String(_) => {
                let value = value
                    .value()
                    .clone()
                    .symbol()
                    .map_err(|mut e| {
                        e.add_stack_trace(self);
                        e
                    })
                    .expect("bad lookup value");
                let call_id = self.new_call_id(&value);
                self.push_goal(Goal::LookupExternal {
                    call_id,
                    instance: object.clone(),
                    field,
                    check_errors: true,
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

    /// Evaluate arithmetic operations.
    fn arithmetic_op_helper(
        &mut self,
        term: &Term,
        op: Operator,
        args: Vec<Term>,
    ) -> PolarResult<QueryEvent> {
        assert_eq!(args.len(), 3);
        let left_term = self.deref(&args[0]);
        let right_term = self.deref(&args[1]);
        let result = &args[2];
        assert!(matches!(result.value(), Value::Variable(_)));

        self.log_with(
            || {
                format!(
                    "MATH: {} {} {} = {}",
                    left_term.to_polar(),
                    op.to_polar(),
                    right_term.to_polar(),
                    result.to_polar()
                )
            },
            &[&left_term, &right_term, result],
        );

        match (left_term.value(), right_term.value()) {
            (Value::Number(left), Value::Number(right)) => {
                if let Some(answer) = match op {
                    Operator::Add => *left + *right,
                    Operator::Sub => *left - *right,
                    Operator::Mul => *left * *right,
                    Operator::Div => *left / *right,
                    _ => {
                        return Err(self.set_error_context(
                            term,
                            error::RuntimeError::Unsupported {
                                msg: format!("numeric operation {}", op.to_polar()),
                            },
                        ))
                    }
                } {
                    self.push_goal(Goal::Unify {
                        left: term.clone_with_value(Value::Number(answer)),
                        right: result.clone(),
                    })?;
                } else {
                    return Err(self.set_error_context(
                        term,
                        error::RuntimeError::ArithmeticError {
                            msg: term.to_polar(),
                        },
                    ));
                }
            }
            (_, _) => todo!(),
        }
        Ok(QueryEvent::None)
    }

    /// Evaluate comparisons.
    fn comparison_op_helper(
        &mut self,
        term: &Term,
        op: Operator,
        args: Vec<Term>,
    ) -> PolarResult<QueryEvent> {
        assert_eq!(args.len(), 2);
        let mut left_term = self.deref(&args[0]);
        let mut right_term = self.deref(&args[1]);

        self.log_with(
            || {
                format!(
                    "CMP: {} {} {}",
                    left_term.to_polar(),
                    op.to_polar(),
                    right_term.to_polar(),
                )
            },
            &[&left_term, &right_term],
        );

        // Coerce booleans to integers.
        fn to_int(x: bool) -> i64 {
            if x {
                1
            } else {
                0
            }
        }
        if let Value::Boolean(x) = left_term.value() {
            left_term = left_term.clone_with_value(Value::Number(Numeric::Integer(to_int(*x))));
        }
        if let Value::Boolean(x) = right_term.value() {
            right_term = right_term.clone_with_value(Value::Number(Numeric::Integer(to_int(*x))));
        }

        // Do the comparison.
        match (left_term.value(), right_term.value()) {
            (Value::Number(left), Value::Number(right)) => {
                if !match op {
                    Operator::Lt => left < right,
                    Operator::Leq => left <= right,
                    Operator::Gt => left > right,
                    Operator::Geq => left >= right,
                    Operator::Eq => left == right,
                    Operator::Neq => left != right,
                    _ => unreachable!("{:?} is not a comparison operator", op),
                } {
                    self.push_goal(Goal::Backtrack)?;
                }
                Ok(QueryEvent::None)
            }
            (Value::String(left), Value::String(right)) => {
                if !match op {
                    Operator::Lt => left < right,
                    Operator::Leq => left <= right,
                    Operator::Gt => left > right,
                    Operator::Geq => left >= right,
                    Operator::Eq => left == right,
                    Operator::Neq => left != right,
                    _ => unreachable!("{:?} is not a comparison operator", op),
                } {
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
            self.log_with(|| format!("=> {}", value.to_string()), &[]);

            self.bind(
                &self
                    .call_id_symbols
                    .get(&call_id)
                    .expect("unregistered external call ID")
                    .clone(),
                value,
            );
        } else {
            self.log("=> No more results.", &[]);

            // No more results. Clean up, cut out the retry alternative,
            // and backtrack.
            self.call_id_symbols.remove(&call_id).expect("bad call ID");

            let check_error = if let Some(goal) = self.goals.last() {
                match *(*goal) {
                    Goal::CheckError => true,
                    _ => false,
                }
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

    /// Handle an external response to ExternalIsSubSpecializer and ExternalIsa
    pub fn external_question_result(&mut self, call_id: u64, answer: bool) {
        let var = self.call_id_symbols.remove(&call_id).expect("bad call id");
        self.bind(&var, Term::new_temporary(Value::Boolean(answer)));
    }

    /// Handle an error coming from outside the vm.
    pub fn external_error(&mut self, message: String) {
        self.external_error = Some(message);
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
            (Value::RestVariable(var), _) => self.unify_var(var, right)?,
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
                // Handled in the parser.
                assert!(left.kwargs.is_none());
                assert!(right.kwargs.is_none());
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

            // External instances can unify if they are the same instance, i.e., have the same
            // instance ID. This is necessary for the case where an instance appears multiple times
            // in the same rule head. For example, `f(foo, foo) if ...` or `isa(x, y, x: y) if ...`
            // or `max(x, y, x) if x > y;`.
            (
                Value::ExternalInstance(ExternalInstance {
                    instance_id: left_instance,
                    ..
                }),
                Value::ExternalInstance(ExternalInstance {
                    instance_id: right_instance,
                    ..
                }),
            ) => {
                // If IDs match, they're the same _instance_ (not just the same _value_), so unify.
                if left_instance != right_instance {
                    self.push_goal(Goal::UnifyExternal {
                        left_instance_id: *left_instance,
                        right_instance_id: *right_instance,
                    })?;
                }
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

    /// Unify two list that end with a rest-variable with eachother.
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
            let rest = unify((
                &shorter[n].clone(),
                &Term::new_temporary(Value::List(longer[n..].to_vec())),
            ));
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
            let rest = unify((
                &rest_list[n].clone(),
                &Term::new_temporary(Value::List(list[n..].to_vec())),
            ));
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
                self.choose_conditional(vec![compare], vec![next_inner], vec![next_outer])?;
            } else {
                assert_eq!(inner, 0);
                self.push_goal(next_outer)?;
            }
        } else {
            // We're done; the rules are sorted.
            // Make alternatives for calling them.

            self.polar_log_mute = false;
            self.log_with(
                || {
                    let mut rule_strs = "APPLICABLE_RULES:".to_owned();
                    for rule in rules {
                        rule_strs.push_str(&format!("\n  {}", self.rule_source(&rule)));
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
                goals.push(Goal::TracePush);
                let Rule { body, params, .. } = self.rename_rule_vars(rule);

                // Unify the arguments with the formal parameters.
                for (arg, param) in args.iter().zip(params.iter()) {
                    goals.push(Goal::Unify {
                        left: arg.clone(),
                        right: param.parameter.clone(),
                    });
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

    pub fn term_source(&self, term: &Term, include_info: bool) -> String {
        let source = self.source(term);
        let span = term.span();

        let mut source_string = match (&source, &span) {
            (Some(source), Some((left, right))) => {
                source.src.chars().take(*right).skip(*left).collect()
            }
            _ => term.to_polar(),
        };

        if include_info {
            if let Some(source) = source {
                let offset = term.offset();
                let (row, column) = crate::lexer::loc_to_pos(&source.src, offset);
                source_string.push_str(&format!(" at line {}, column {}", row + 1, column));
                if let Some(filename) = source.filename {
                    source_string.push_str(&format!(" in file {}", filename));
                }
            }
        }

        source_string
    }

    pub fn rule_source(&self, rule: &Rule) -> String {
        let head = format!(
            "{}({})",
            rule.name,
            rule.params.iter().fold(String::new(), |mut acc, p| {
                if acc != "" {
                    acc += ", ";
                }
                acc += &self.term_source(&p.parameter, false);
                if let Some(spec) = &p.specializer {
                    acc += ": ";
                    acc += &self.term_source(&spec, false);
                }
                acc
            })
        );
        match rule.body.value() {
            Value::Expression(Operation {
                operator: Operator::And,
                args,
            }) if !args.is_empty() => head + " if " + &self.term_source(&rule.body, false) + ";",
            _ => head + ";",
        }
    }

    fn set_error_context(
        &self,
        term: &Term,
        error: impl Into<error::PolarError>,
    ) -> error::PolarError {
        let source = self.source(term);
        let error: error::PolarError = error.into();
        error.set_context(source.as_ref(), Some(term))
    }

    fn type_error(&self, term: &Term, msg: String) -> error::PolarError {
        let stack_trace = self.stack_trace();
        let error = error::RuntimeError::TypeError {
            msg,
            stack_trace: Some(stack_trace),
        };
        self.set_error_context(term, error)
    }
}

#[cfg(test)]
mod tests {
    use permute::permute;

    use super::*;
    use crate::rewrites::unwrap_and;

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

        let rule = GenericRule::new(sym!("f"), vec![Arc::new(f1), Arc::new(f2)]);

        let mut kb = KnowledgeBase::new();
        kb.rules.insert(rule.name.clone(), rule);

        let goal = query!(op!(And));

        let mut vm = PolarVirtualMachine::new_test(Arc::new(RwLock::new(kb)), false, vec![goal]);
        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!()},
            QueryEvent::Done
        ]);

        assert!(vm.is_halted());

        let f1 = term!(call!("f", [1]));
        let f2 = term!(call!("f", [2]));
        let f3 = term!(call!("f", [3]));

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
            field: term!(call!("x")),
            value: term!(1),
        })
        .unwrap();

        assert_query_events!(vm, [
            QueryEvent::Result{hashmap!{}}
        ]);

        // Lookup with incorrect value
        vm.push_goal(Goal::Lookup {
            dict: dict.clone(),
            field: term!(call!("x")),
            value: term!(2),
        })
        .unwrap();

        assert_query_events!(vm, [QueryEvent::Done]);

        // Lookup with unbound value
        vm.push_goal(Goal::Lookup {
            dict,
            field: term!(call!("x")),
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
        let mut vm = PolarVirtualMachine::new_test(
            Arc::new(RwLock::new(KnowledgeBase::new())),
            false,
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
        let mut vm = PolarVirtualMachine::new_test(
            Arc::new(RwLock::new(KnowledgeBase::new())),
            false,
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
        let mut vm = PolarVirtualMachine::new_test(
            Arc::new(RwLock::new(KnowledgeBase::new())),
            false,
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
    fn test_filter_rules() {
        let rule_a = Arc::new(rule!("bar", ["_"; instance!("a")]));
        let rule_b = Arc::new(rule!("bar", ["_"; instance!("b")]));
        let rule_1a = Arc::new(rule!("bar", [value!(1)]));
        let rule_1b = Arc::new(rule!("bar", ["_"; value!(1)]));

        let gen_rule = GenericRule::new(sym!("bar"), vec![rule_a, rule_b, rule_1a, rule_1b]);
        let mut kb = KnowledgeBase::new();
        kb.add_generic_rule(gen_rule);

        let external_instance = Value::ExternalInstance(ExternalInstance {
            instance_id: 1,
            constructor: None,
            repr: None,
        });
        let query = query!(call!("bar", [sym!("x")]));
        let mut vm = PolarVirtualMachine::new_test(Arc::new(RwLock::new(kb)), false, vec![query]);
        vm.bind(&sym!("x"), Term::new_from_test(external_instance));

        let mut external_isas = vec![];

        loop {
            match vm.run().unwrap() {
                QueryEvent::Done => break,
                QueryEvent::ExternalIsa {
                    call_id, class_tag, ..
                } => {
                    external_isas.push(class_tag.clone());
                    // Return `true` if the specified `class_tag` is `"a"`.
                    vm.external_question_result(call_id, class_tag.0 == "a")
                }
                QueryEvent::ExternalIsSubSpecializer { .. } | QueryEvent::Result { .. } => (),
                _ => panic!("Unexpected event"),
            }
        }

        let expected = vec![sym!("b"), sym!("a"), sym!("a")];
        assert_eq!(external_isas, expected);

        vm.bind(&sym!("x"), Term::new_from_test(value!(1)));
        let _ = vm
            .query(&Term::new_from_test(Value::Call(call!("bar", [sym!("x")]))))
            .unwrap();

        let mut results = vec![];
        loop {
            match vm.run().unwrap() {
                QueryEvent::Done => break,
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
            constructor: None,
            repr: None,
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
        let mut vm = PolarVirtualMachine::default();
        vm.set_query_timeout(1);
        // Turn this off so we don't hit it.
        vm.set_stack_limit(std::usize::MAX);

        loop {
            vm.push_goal(Goal::Noop).unwrap();
            vm.push_goal(Goal::UnifyExternal {
                left_instance_id: 1,
                right_instance_id: 1,
            })
            .unwrap();
            let result = vm.run();
            match result {
                Ok(event) => assert!(matches!(event, QueryEvent::ExternalUnify { .. })),
                Err(err) => {
                    assert!(matches!(err, error::PolarError {
                        kind: error::ErrorKind::Runtime(error::RuntimeError::QueryTimeout { .. }),
                        ..
                    }));

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
        vm.bind(&sym!("x"), term!(1));
        let _ = vm.run();
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
            QueryEvent::Done
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
            QueryEvent::Done
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
            QueryEvent::Done
        ]);
    }
}
