use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Write,
    rc::Rc,
    string::ToString,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{
    bindings::{BindingManager, BindingStack, Bindings, Bsp, FollowerId, VariableState},
    counter::Counter,
    debugger::{DebugEvent, Debugger},
    error::{self, PolarError, PolarResult},
    events::*,
    formatting::ToPolarString,
    kb::*,
    messages::*,
    numerics::*,
    rules::*,
    runnable::Runnable,
    sources::*,
    terms::*,
    traces::*,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub const MAX_STACK_SIZE: usize = 10_000;
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Clone)]
#[must_use = "ignored goals are never accomplished"]
#[allow(clippy::large_enum_variant)]
pub enum Goal {
    Backtrack,
    Cut(usize),
    Debug(String),
    Error(PolarError),
    Halt,
    Isa(Term, Term),
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
    Query(Term),
    PopQuery(Term),
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
    TraceRule(Rc<Trace>),
    TraceStackPush,
    TraceStackPop,
    Unify(Term, Term),

    /// Run the `runnable`.
    Run(Box<dyn Runnable>),

    /// Add a new constraint
    AddConstraint(Term),

    /// TODO hack.
    /// Add a new constraint
    AddConstraints(Rc<RefCell<Bindings>>),
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

// TODO(ap): don't panic.
pub fn compare(op: Operator, left: &Term, right: &Term) -> PolarResult<bool> {
    use Operator::*;
    use Value::*;
    // Coerce booleans to integers.
    fn to_int(x: bool) -> Numeric {
        Numeric::Integer(if x { 1 } else { 0 })
    }

    fn compare<T: PartialOrd>(op: Operator, left: T, right: T) -> bool {
        match op {
            Lt => left < right,
            Leq => left <= right,
            Gt => left > right,
            Geq => left >= right,
            Eq => left == right,
            Neq => left != right,
            _ => panic!("`{}` is not a comparison operator", op.to_polar()),
        }
    }

    match (left.value(), right.value()) {
        (Boolean(l), Boolean(r)) => Ok(compare(op, &to_int(*l), &to_int(*r))),
        (Boolean(l), Number(r)) => Ok(compare(op, &to_int(*l), r)),
        (Number(l), Boolean(r)) => Ok(compare(op, l, &to_int(*r))),
        (Number(l), Number(r)) => Ok(compare(op, l, r)),
        (String(l), String(r)) => Ok(compare(op, l, r)),
        _ => Err(error::RuntimeError::Unsupported {
            msg: format!("{} {} {}", left.to_polar(), op.to_polar(), right.to_polar()),
        }
        .into()),
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
    log: bool,
    polar_log: bool,
    polar_log_stderr: bool,
    polar_log_mute: bool,

    // Other flags.
    pub query_contains_partial: bool,
    pub inverting: bool,

    /// Output messages.
    pub messages: MessageQueue,
}

impl Default for PolarVirtualMachine {
    fn default() -> Self {
        PolarVirtualMachine::new(
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
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
            .constants
            .clone();
        let stack_limit = MAX_STACK_SIZE;
        // logging options ... there's a lot :O
        let (log, polar_log, polar_log_stderr) = {
            // get all comma-delimited POLAR_LOG variables
            let polar_log = std::env::var("POLAR_LOG");
            let polar_log_vars = polar_log
                .iter()
                .flat_map(|pl| pl.split(','))
                .collect::<Vec<&str>>();
            (
                // `log` controls internal VM logging
                polar_log_vars.iter().any(|var| var == &"trace"),
                // `polar_log` for tracing policy evaluation
                !polar_log_vars.is_empty()
                    && !polar_log_vars.iter().any(|var| ["0", "off"].contains(var)),
                // `polar_log_stderr` prints things immediately to stderr
                polar_log_vars.iter().any(|var| var == &"now"),
            )
        };
        let mut vm = Self {
            query_timeout_ms,
            tracing,
            kb,
            messages,
            log,
            polar_log,
            polar_log_stderr,
            stack_limit,
            goals: GoalStack::new_reversed(goals),
            binding_manager: Default::default(),
            query_start_time: Default::default(),
            csp: Default::default(),
            choices: Default::default(),
            queries: Default::default(),
            trace_stack: Default::default(),
            trace: Default::default(),
            external_error: Default::default(),
            debugger: Default::default(),
            call_id_symbols: Default::default(),
            polar_log_mute: Default::default(),
            query_contains_partial: Default::default(),
            inverting: Default::default(),
        };
        vm.bind_constants(constants);
        vm.query_contains_partial();
        vm
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_logging_options(&mut self, rust_log: Option<String>, polar_log: Option<String>) {
        self.log = rust_log.is_some();
        if let Some(pl) = polar_log {
            if &pl == "now" {
                self.polar_log_stderr = true;
            }
            self.polar_log = match Some(pl).as_deref() {
                None | Some("0") | Some("off") => false,
                _ => true,
            }
        }
    }

    fn kb(&self) -> RwLockReadGuard<KnowledgeBase> {
        self.kb.read().unwrap()
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
            if let Goal::Query(term) = goal.as_ref() {
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

    pub fn id_counter(&self) -> Counter {
        self.kb().id_counter()
    }

    fn new_call_id(&mut self, symbol: &Symbol) -> u64 {
        let call_id = self.kb().new_id();
        self.call_id_symbols.insert(call_id, symbol.clone());
        call_id
    }

    fn new_call_var(&mut self, var_prefix: &str, initial_value: Value) -> (u64, Term) {
        let sym = self.kb().gensym(var_prefix);
        self.bind(&sym, term!(initial_value)).unwrap();
        let call_id = self.new_call_id(&sym);
        (call_id, Term::from(sym))
    }

    fn get_call_sym(&self, call_id: u64) -> &Symbol {
        self.call_id_symbols
            .get(&call_id)
            .expect("unregistered external call ID")
    }

    fn cut(&mut self, i: usize) -> PolarResult<QueryEvent> {
        self.choices.truncate(i);
        self.query_event_none()
    }

    /// Try to achieve one goal. Return `Some(QueryEvent)` if an external
    /// result is needed to achieve it, or `None` if it can run internally.
    fn do_goal(&mut self, goal: Rc<Goal>) -> PolarResult<QueryEvent> {
        if self.log {
            self.print(&format!("{}", goal));
        }

        self.check_timeout()?;

        use Goal::*;
        match goal.as_ref() {
            Backtrack => self.backtrack()?.query_event_none(),
            Cut(i) => self.cut(*i),
            Debug(msg) => self.query_event_debug(msg),
            Halt => {
                self.log("HALT", &[]);
                self.goals.clear();
                self.choices.clear();
                self.query_event_done(true)
            }

            Error(error) => Err(error.clone()),
            Isa(left, right) => self.isa(left, right)?.query_event_none(),
            IsMoreSpecific { left, right, args } => {
                self.is_more_specific(left, right, args)?.query_event_none()
            }
            IsSubspecializer {
                answer,
                left,
                right,
                arg,
            } => self.is_subspecializer(answer, left, right, arg),
            Lookup { dict, field, value } => self.lookup(dict, field, value),
            LookupExternal {
                call_id,
                instance,
                field,
            } => self.lookup_external(*call_id, instance, field),
            IsaExternal { instance, literal } => self.query_event_isa_external(instance, literal),
            MakeExternal {
                constructor,
                instance_id,
            } => self.query_event_make_external(constructor, *instance_id),
            NextExternal { call_id, iterable } => {
                self.query_event_next_external(*call_id, iterable)
            }
            CheckError => self.check_error()?.query_event_none(),
            Noop => self.query_event_none(),
            Query(term) => {
                let result = self.query(term);
                self.maybe_break(DebugEvent::Query)?;
                result
            }
            PopQuery { .. } => {
                self.queries.pop();
                self.query_event_none()
            }
            FilterRules {
                applicable_rules,
                unfiltered_rules,
                args,
            } => self
                .filter_rules(applicable_rules.clone(), unfiltered_rules.clone(), args)?
                .query_event_none(),
            SortRules {
                rules,
                outer,
                inner,
                args,
            } => self
                .sort_rules(rules.clone(), args, *outer, *inner)?
                .query_event_none(),
            TraceStackPush => {
                self.trace_stack.push(Rc::new(self.trace.clone()));
                self.trace = vec![];
                self.query_event_none()
            }
            TraceStackPop => {
                let mut children = self.trace.clone();
                self.trace = self.trace_stack.pop().unwrap().as_ref().clone();
                let mut trace = self.trace.pop().unwrap();
                let trace = Rc::make_mut(&mut trace);
                trace.children.append(&mut children);
                self.trace.push(Rc::new(trace.clone()));
                self.maybe_break(DebugEvent::Pop)?;
                self.query_event_none()
            }
            TraceRule(trace) => {
                if let Node::Rule(rule) = &trace.node {
                    self.log_with(
                        || {
                            let source_str = rule.to_polar();
                            format!("RULE: {}", source_str)
                        },
                        &[],
                    );
                }
                self.trace.push(trace.clone());
                self.maybe_break(DebugEvent::Rule)?;
                self.query_event_none()
            }
            Unify(left, right) => self.unify(left, right)?.query_event_none(),
            AddConstraint(term) => self.add_constraint(term)?.query_event_none(),
            AddConstraints(constraints) => constraints
                .borrow_mut()
                .drain()
                .fold(Ok(self), |this, (_, con)| this?.add_constraint(&con))?
                .query_event_none(),
            Run(r) => self.run_runnable(r.clone_runnable()),
        }
    }

    /// Push a goal onto the goal stack.
    pub fn push_goal(&mut self, goal: Goal) -> PolarResult<&mut Self> {
        match goal {
            _ if self.goals.len() >= self.stack_limit =>
                Err(error::RuntimeError::StackOverflow {
                    msg: format!("Goal stack overflow! MAX_GOALS = {}", self.stack_limit),
                }
                .into()),
            Goal::LookupExternal { call_id, .. } | Goal::NextExternal { call_id, .. }
                if !matches!(
                        self.variable_state(self.get_call_sym(call_id)),
                        VariableState::Unbound) =>
                panic!( "The call_id result variables for LookupExternal and NextExternal goals must be unbound."),
            _ => {
                self.goals.push(Rc::new(goal));
                Ok(self)
            }
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
    fn push_choice<I>(&mut self, alternatives: I) -> &mut Self
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
        self
    }

    /// Push a choice onto the choice stack, and execute immediately by
    /// pushing the first alternative onto the goals stack
    ///
    /// Params:
    ///
    /// - `alternatives`: an ordered list of alternatives to try in the choice.
    ///   The first element is the first alternative to try.
    fn choose<I>(&mut self, alternatives: I) -> PolarResult<&mut Self>
    where
        I: IntoIterator<Item = Goals>,
        I::IntoIter: std::iter::DoubleEndedIterator,
    {
        let mut alternatives_iter = alternatives.into_iter();
        if let Some(alternative) = alternatives_iter.next() {
            self.push_choice(alternatives_iter)
                .append_goals(alternative)
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
    ) -> PolarResult<&mut Self> {
        // If the conditional fails, cut the consequent.
        let cut_consequent = Goal::Cut(self.choices.len());
        alternative.insert(0, cut_consequent);

        // If the conditional succeeds, cut the alternative and backtrack to this choice point.
        self.push_choice(vec![consequent]);
        let cut_alternative = Goal::Cut(self.choices.len());
        conditional.push(cut_alternative);
        conditional.push(Goal::Backtrack);

        self.choose(vec![conditional, alternative])
    }

    /// Push multiple goals onto the stack in reverse order.
    fn append_goals<I>(&mut self, goals: I) -> PolarResult<&mut Self>
    where
        I: IntoIterator<Item = Goal>,
        I::IntoIter: std::iter::DoubleEndedIterator,
    {
        goals
            .into_iter()
            .rev()
            .fold(Ok(self), |this, g| this?.push_goal(g))
    }

    /// Rebind an external answer variable.
    ///
    /// DO NOT USE THIS TO REBIND ANOTHER VARIABLE (see unsafe_rebind doc string).
    fn rebind_external_answer(&mut self, var: &Symbol, val: Term) {
        self.binding_manager.unsafe_rebind(var, val);
    }

    /// Push a binding onto the binding stack.
    pub fn bind(&mut self, var: &Symbol, val: Term) -> PolarResult<&mut Self> {
        if self.log {
            self.print(&format!("⇒ bind: {} ← {}", var.to_polar(), val.to_polar()));
        }
        if let Some(goal) = self.binding_manager.bind(var, val)? {
            self.push_goal(goal)
        } else {
            Ok(self)
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
    fn add_constraint(&mut self, term: &Term) -> PolarResult<&mut Self> {
        if self.log {
            self.print(&format!("⇒ add_constraint: {}", term.to_polar()));
        }
        self.binding_manager.add_constraint(term)?;
        Ok(self)
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

    /// Retrive internal binding stack for debugger.
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
        use crate::{folder::Folder, rewrites::Renamer};
        Renamer::new(&self.kb()).fold_rule(rule.clone())
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
                self.print(msg);
                for line in &lines[1..] {
                    self.print(format!("[debug] {}{}", &indent, line));
                }
            }
        }
    }

    pub fn source(&self, term: &Term) -> Option<Source> {
        term.get_source_id()
            .and_then(|id| self.kb().sources.get_source(id))
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
                        let (row, column) = crate::lexer::loc_to_pos(&source.src, t.offset());
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

    fn check_timeout(&mut self) -> PolarResult<&mut Self> {
        let elapsed = self.query_duration();
        if self.is_query_timeout_disabled() || elapsed <= self.query_timeout_ms {
            Ok(self)
        } else {
            Err(error::RuntimeError::QueryTimeout {
                msg: format!(
                    "Query running for {}ms, which exceeds the timeout of {}ms. To disable timeouts, set the POLAR_TIMEOUT_MS environment variable to 0.",
                    elapsed, self.query_timeout_ms
                ),
            }
            .into())
        }
    }

    /// Remove all bindings after the last choice point, and try the
    /// next available alternative. If no choice is possible, halt.
    fn backtrack(&mut self) -> PolarResult<&mut Self> {
        if self.log {
            self.print("⇒ backtrack");
        }
        self.log("BACKTRACK", &[]);
        self.backtrack_loop()
    }

    fn backtrack_loop(&mut self) -> PolarResult<&mut Self> {
        match self.choices.pop() {
            None => self.push_goal(Goal::Halt),
            Some(mut ch) => {
                self.binding_manager.backtrack(&ch.bsp);
                match ch.alternatives.pop() {
                    None => self.backtrack_loop(),
                    Some(mut alternative) => {
                        if ch.alternatives.is_empty() {
                            self.goals = ch.goals;
                            self.queries = ch.queries;
                            self.trace = ch.trace;
                            self.trace_stack = ch.trace_stack;
                        } else {
                            self.goals.clone_from(&ch.goals);
                            self.queries.clone_from(&ch.queries);
                            self.trace.clone_from(&ch.trace);
                            self.trace_stack.clone_from(&ch.trace_stack);
                            self.choices.push(ch)
                        }
                        self.goals.append(&mut alternative);
                        Ok(self)
                    }
                }
            }
        }
    }

    /// Interact with the debugger.
    fn query_event_debug(&mut self, message: &str) -> PolarResult<QueryEvent> {
        // Query start time is reset when a debug event occurs.
        self.query_start_time.take();
        Ok(QueryEvent::Debug {
            message: message.to_string(),
        })
    }

    fn query_event_none(&self) -> PolarResult<QueryEvent> {
        Ok(QueryEvent::None)
    }

    fn query_event_done(&self, result: bool) -> PolarResult<QueryEvent> {
        Ok(QueryEvent::Done { result })
    }

    /// Comparison operator that essentially performs partial unification.
    pub fn isa(&mut self, left: &Term, right: &Term) -> PolarResult<&mut Self> {
        self.log_with(
            || format!("MATCHES: {} matches {}", left.to_polar(), right.to_polar()),
            &[left, right],
        );

        match (left.value(), right.value()) {
            (_, Value::Dictionary(_)) => todo!("make this case unreachable"),
            (Value::Expression(_), _) | (_, Value::Expression(_)) => {
                unreachable!("encountered bare expression")
            }

            _ if left.is_union() => {
                // A union (currently) only matches itself.
                //
                // TODO(gj): when we have unions beyond `Actor` and `Resource`, we'll need to be
                // smarter about this check since UnionA is more specific than UnionB if UnionA is
                // a member of UnionB.
                let unions_match = (left.is_actor_union() && right.is_actor_union())
                    || (left.is_resource_union() && right.is_resource_union());
                if !unions_match {
                    self.backtrack()
                } else {
                    Ok(self)
                }
            }
            _ if right.is_union() => self.isa_union(left, right),

            // TODO(gj): (Var, Rest) + (Rest, Var) cases might be unreachable.
            (Value::Variable(l), Value::Variable(r))
            | (Value::Variable(l), Value::RestVariable(r))
            | (Value::RestVariable(l), Value::Variable(r))
            | (Value::RestVariable(l), Value::RestVariable(r)) => {
                // Two variables.
                match (self.variable_state(l), self.variable_state(r)) {
                    (VariableState::Bound(x), _) => self.isa(&x, right),
                    (_, VariableState::Bound(y)) => self.isa(left, &y),
                    (_, _) => self.add_constraint(&term!(op!(Isa, left.clone(), right.clone()))),
                }
            }
            (Value::Variable(l), _) | (Value::RestVariable(l), _) => match self.variable_state(l) {
                VariableState::Bound(x) => self.isa(&x, right),
                _ => self.isa_expr(left, right),
            },
            (_, Value::Variable(r)) | (_, Value::RestVariable(r)) => match self.variable_state(r) {
                VariableState::Bound(y) => self.isa(left, &y),
                _ => self.unify(left, right),
            },

            (Value::List(left), Value::List(right)) => self.unify_lists(Goal::Isa, left, right),

            // FIXME(gw/ss) recursive isa could be expressed better
            (Value::Dictionary(left), Value::Pattern(Pattern::Dictionary(right))) => {
                // if both sides are plain dictionaries, the isa falls through to a unify.
                // if the right side is a pattern, then the isa succeeds if
                // - each key in the pattern is present in the dict and
                // - for each key in pattern, dict[key] `isa` pattern[key]
                let left_fields: HashSet<&Symbol> = left.fields.keys().collect();
                let right_fields: HashSet<&Symbol> = right.fields.keys().collect();
                if !right_fields.is_subset(&left_fields) {
                    self.backtrack()
                } else {
                    self.append_goals(
                        right.fields.iter().map(|(k, v)| {
                            Goal::Isa(left.fields.get(k).unwrap().clone(), v.clone())
                        }),
                    )
                }
            }

            (_, Value::Pattern(Pattern::Dictionary(right))) => {
                right
                    .fields
                    .iter()
                    .fold(Ok(self), |this, (field, right_value)| {
                        let this = this?;
                        // Generate symbol for the lookup result and leave the variable unbound, so that unification with the result does not fail.
                        // Unification with the lookup result happens in `fn external_call_result()`.
                        let answer = this.kb().gensym("isa_value");
                        let call_id = this.new_call_id(&answer);

                        let lookup = Goal::LookupExternal {
                            instance: left.clone(),
                            call_id,
                            field: right_value.clone_with_value(Value::String(field.0.clone())),
                        };
                        let isa = Goal::Isa(term!(answer), right_value.clone());
                        this.append_goals(vec![lookup, isa])
                    })
            }

            (_, Value::Pattern(Pattern::Instance(right_literal))) => self.append_goals(vec![
                Goal::IsaExternal {
                    instance: left.clone(),
                    literal: right_literal.clone(),
                },
                Goal::Isa(
                    left.clone(),
                    right.clone_with_value(Value::Pattern(Pattern::Dictionary(
                        right_literal.fields.clone(),
                    ))),
                ),
            ]),

            // Default case: x isa y if x = y.
            _ => self.unify(left, right),
        }
    }

    fn get_names(&self, s: &Symbol) -> HashSet<Symbol> {
        let cycles = self
            .binding_manager
            .get_constraints(s)
            .constraints()
            .into_iter()
            .filter_map(|con| match con.operator {
                Operator::Unify | Operator::Eq => {
                    if let (Ok(l), Ok(r)) = (
                        con.args[0].value().as_symbol(),
                        con.args[1].value().as_symbol(),
                    ) {
                        Some((l.clone(), r.clone()))
                    } else {
                        None
                    }
                }
                _ => None,
            });

        crate::data_filtering::partition_equivs(cycles)
            .into_iter()
            .find(|c| c.contains(s))
            .unwrap_or_else(|| {
                let mut hs = HashSet::with_capacity(1);
                hs.insert(s.clone());
                hs
            })
    }

    fn isa_expr(&mut self, left: &Term, right: &Term) -> PolarResult<&mut Self> {
        use crate::partial::{simplify_partial, IsaConstraintCheck};
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
                Ok(self)
            }
            Value::Pattern(Pattern::Instance(InstanceLiteral { fields, tag })) => {
                // TODO(gj): assert that a simplified expression contains at most 1 unification
                // involving a particular variable.
                // TODO(gj): Ensure `op!(And) matches X{}` doesn't die after these changes.
                // Get the existing partial on the LHS variable.
                let var = left.value().as_symbol()?;
                let partial = term!(self.binding_manager.get_constraints(var));

                // get the aliases for this variable
                let names = self.get_names(var);
                let simplified = simplify_partial(partial, names.clone());
                if simplified.is_none() {
                    return self.backtrack();
                }
                let simplified = simplified.unwrap();
                let simplified = simplified.value().as_expression()?;

                // TODO (dhatch): what if there is more than one var = dot_op constraint?
                // What if the one there is is in a not, or an or, or something
                let lhs_of_matches = simplified
                    .constraints()
                    .into_iter()
                    .find_map(|c| {
                        // If the simplified partial includes a `var = dot_op` constraint, use the
                        // dot op as the LHS of the matches.
                        if c.operator != Operator::Unify {
                            None
                        } else if &c.args[0] == left &&
                            matches!(c.args[1].value().as_expression(), Ok(o) if o.operator == Operator::Dot) {
                            Some(c.args[1].clone())
                        } else if &c.args[1] == left &&
                            matches!(c.args[0].value().as_expression(), Ok(o) if o.operator == Operator::Dot) {
                            Some(c.args[0].clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| left.clone());

                // Construct field-less matches operation.
                let tag_pattern = right.clone_with_value(value!(pattern!(instance!(tag.clone()))));
                let type_constraint = op!(Isa, left.clone(), tag_pattern);

                let new_matches = op!(Isa, lhs_of_matches, right.clone());

                let runnable = Box::new(IsaConstraintCheck::new(
                    simplified.constraints(),
                    new_matches,
                    names,
                ));

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
                    vec![Goal::Run(runnable)],
                    add_constraints
                        .into_iter()
                        .map(|op| Goal::AddConstraint(op.into()))
                        .collect(),
                    vec![Goal::CheckError, Goal::Backtrack],
                )
            }
            // if the RHS isn't a pattern or a dictionary, we'll fall back to unifying
            // this is not the _best_ behaviour, but it's what we've been doing
            // previously
            _ => self.unify(left, right),
        }
    }

    /// To evaluate `left matches Union`, look up `Union`'s member classes and create a choicepoint
    /// to check if `left` matches any of them.
    fn isa_union(&mut self, left: &Term, union: &Term) -> PolarResult<&mut Self> {
        let member_isas: Vec<_> = self
            .kb()
            .get_union_members(union)
            .iter()
            .map(|member| {
                let tag = member.value().as_symbol().unwrap().0.as_str();
                member.clone_with_value(value!(pattern!(instance!(tag))))
            })
            .map(|pattern| vec![Goal::Isa(left.clone(), pattern)])
            .collect();
        self.choose(member_isas)
    }

    fn lookup(&mut self, dict: &Dictionary, field: &Term, value: &Term) -> PolarResult<QueryEvent> {
        let field = self.deref(field);
        match field.value() {
            Value::Variable(_) => {
                let mut alternatives = vec![];
                for (k, v) in &dict.fields {
                    let mut goals: Goals = vec![];
                    // attempt to unify dict key with field
                    // if `field` is bound, unification will only succeed for the matching key
                    // if `field` is unbound, unification will succeed for all keys
                    goals.push(Goal::Unify(
                        field.clone_with_value(Value::String(k.clone().0)),
                        field.clone(),
                    ));
                    // attempt to unify dict value with result
                    goals.push(Goal::Unify(v.clone(), value.clone()));
                    alternatives.push(goals);
                }
                self.choose(alternatives)
            }
            Value::String(field) => {
                if let Some(retrieved) = dict.fields.get(&Symbol(field.clone())) {
                    self.unify(retrieved, value)
                } else {
                    self.backtrack()
                }
            }
            v => self.type_error(
                &field,
                format!("cannot look up field {:?} on a dictionary", v),
            ),
        }?
        .query_event_none()
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
        self.push_choice(vec![]).log_with(
            || {
                let mut msg = format!("LOOKUP: {}.{}", instance.to_string(), field_name);
                msg.push('(');
                let args = args
                    .clone()
                    .unwrap_or_else(Vec::new)
                    .into_iter()
                    .map(|a| a.to_polar());
                let kwargs = kwargs
                    .clone()
                    .unwrap_or_else(BTreeMap::new)
                    .into_iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_polar()));
                msg.push_str(&args.chain(kwargs).collect::<Vec<String>>().join(", "));
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

    fn query_event_isa_external(
        &mut self,
        instance: &Term,
        literal: &InstanceLiteral,
    ) -> PolarResult<QueryEvent> {
        let (call_id, answer) = self.new_call_var("isa", false.into());
        self.push_goal(Goal::Unify(answer, Term::from(true)))?;

        Ok(QueryEvent::ExternalIsa {
            call_id,
            instance: self.deref(instance),
            class_tag: literal.tag.clone(),
        })
    }

    fn query_event_next_external(
        &mut self,
        call_id: u64,
        iterable: &Term,
    ) -> PolarResult<QueryEvent> {
        // add another choice point for the next result
        self.push_choice(vec![vec![Goal::NextExternal {
            call_id,
            iterable: iterable.clone(),
        }]]);

        Ok(QueryEvent::NextExternal {
            call_id,
            iterable: iterable.clone(),
        })
    }

    fn query_event_make_external(
        &self,
        constructor: &Term,
        instance_id: u64,
    ) -> PolarResult<QueryEvent> {
        Ok(QueryEvent::MakeExternal {
            instance_id,
            constructor: self.deref(constructor),
        })
    }

    fn check_error(&mut self) -> PolarResult<&mut Self> {
        match &self.external_error {
            None => Ok(self),
            Some(error) => {
                let error = error::RuntimeError::Application {
                    msg: error.clone(),
                    stack_trace: Some(self.stack_trace()),
                };
                match self.trace.last().map(|t| t.node.clone()) {
                    Some(Node::Term(t)) => self.set_error_context(&t, error),
                    _ => Err(error.into()),
                }
            }
        }
    }

    /// Query for the provided term.
    ///
    /// Uses the knowledge base to get an ordered list of rules.
    /// Creates a choice point over each rule, where each alternative
    /// consists of unifying the rule head with the arguments, then
    /// querying for each body clause.
    fn query(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        use {Goal::*, Operator::*, Value::*, VariableState::*};
        // Don't log if it's just a single element AND like lots of rule bodies tend to be.
        match &term.value() {
            Expression(Operation {
                operator: And,
                args,
            }) if args.len() < 2 => (),
            _ => {
                self.log_with(|| format!("QUERY: {}", term.to_polar()), &[term]);
            }
        };

        self.queries.push(term.clone());
        self.push_goal(PopQuery(term.clone()))?
            .trace
            .push(Rc::new(Trace {
                node: Node::Term(term.clone()),
                children: vec![],
            }));

        match &term.value() {
            Boolean(true) => self.query_event_none(),
            Boolean(false) => self.backtrack()?.query_event_none(),
            Expression(_) => self.query_for_operation(term),
            Call(p) => self.query_for_predicate(p.clone()),
            Variable(sym) => {
                if let Bound(term) = self.variable_state(sym) {
                    self.query(&term)
                } else {
                    self.unify(term, &term!(true))?.query_event_none()
                }
            }
            _ => self.type_error(
                term,
                format!(
                    "{} isn't something that is true or false so can't be a condition",
                    term.value().to_polar()
                ),
            ),
        }
    }

    /// Select applicable rules for predicate.
    /// Sort applicable rules by specificity.
    /// Create a choice over the applicable rules.
    fn query_for_predicate(&mut self, predicate: Call) -> PolarResult<QueryEvent> {
        assert!(predicate.kwargs.is_none());
        let goals = match self.kb.read().unwrap().get_generic_rule(&predicate.name) {
            None => vec![Goal::Backtrack],
            Some(generic_rule) => {
                assert_eq!(generic_rule.name, predicate.name);

                // Pre-filter rules.
                let args = predicate.args.iter().map(|t| self.deref(t)).collect();
                let pre_filter = generic_rule.get_applicable_rules(&args);

                self.polar_log_mute = true;

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
        self.append_goals(goals)?.query_event_none()
    }

    fn query_for_operation(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        use crate::inverter::Inverter;
        use Operator::*;
        let operation = term.value().as_expression()?;
        let mut args = operation.args.clone();
        match operation.operator {
            And => {
                // Query for each conjunct.
                self.push_goal(Goal::TraceStackPop)?
                    .append_goals(args.into_iter().map(Goal::Query))?
                    .push_goal(Goal::TraceStackPush)?
                    .query_event_none()
            }
            Or => {
                // Make an alternative Query for each disjunct.
                self.choose(args.into_iter().map(|term| vec![Goal::Query(term)]))?
                    .query_event_none()
            }
            Not => {
                // Query in a sub-VM and invert the results.
                assert_eq!(args.len(), 1);
                let term = args.pop().unwrap();
                let constraints = Rc::new(RefCell::new(Bindings::new()));
                let inverter = Box::new(Inverter::new(
                    self,
                    vec![Goal::Query(term)],
                    constraints.clone(),
                    self.bsp(),
                ));
                self.choose_conditional(
                    vec![Goal::Run(inverter)],
                    vec![Goal::AddConstraints(constraints)],
                    vec![Goal::Backtrack],
                )?
                .query_event_none()
            }
            Assign => {
                assert_eq!(args.len(), 2);
                let (left, right) = (&args[0], &args[1]);
                match (left.value(), right.value()) {
                    (Value::Variable(var), _)
                        if self.variable_state(var) == VariableState::Unbound =>
                    {
                        self.unify(left, right)?.query_event_none()
                    }
                    (Value::Variable(var), _) => self.type_error(
                        left,
                        format!(
                            "Can only assign to unbound variables, {} is not unbound.",
                            var.to_polar()
                        ),
                    ),
                    _ => {
                        self.type_error(left, format!("Cannot assign to type {}.", left.to_polar()))
                    }
                }
            }

            Unify => {
                assert_eq!(args.len(), 2);
                self.unify(&args[0], &args[1])?.query_event_none()
            }
            Dot => self.query_op_helper(term, Self::dot_op_helper, false, false),

            Lt | Gt | Leq | Geq | Eq | Neq => {
                self.query_op_helper(term, Self::comparison_op_helper, true, true)
            }

            Add | Sub | Mul | Div | Mod | Rem => {
                self.query_op_helper(term, Self::arithmetic_op_helper, true, true)
            }

            In => self.query_op_helper(term, Self::in_op_helper, false, true),

            Debug => {
                let msg = self.debugger.break_msg(self).unwrap_or_else(|| {
                    format!(
                        "debug({})",
                        args.iter()
                            .map(|arg| self.deref(arg).to_polar())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                });
                self.push_goal(Goal::Debug(msg))?.query_event_none()
            }
            Print => {
                self.print(
                    &args
                        .iter()
                        .map(|arg| self.deref(arg).to_polar())
                        .collect::<Vec<String>>()
                        .join(", "),
                );
                self.query_event_none()
            }
            New => {
                assert_eq!(args.len(), 2);
                let result = args.pop().unwrap();
                assert!(
                    matches!(result.value(), Value::Variable(_)),
                    "Must have result variable as second arg."
                );
                let constructor = args.pop().unwrap();

                let instance_id = self.kb().new_id();
                let instance =
                    constructor.clone_with_value(Value::ExternalInstance(ExternalInstance {
                        instance_id,
                        constructor: Some(constructor.clone()),
                        repr: Some(constructor.to_polar()),
                    }));

                // A goal is used here in case the result is already bound to some external
                // instance.
                self.append_goals(vec![
                    Goal::Unify(result, instance),
                    Goal::MakeExternal {
                        instance_id,
                        constructor,
                    },
                ])?
                .query_event_none()
            }
            Cut => {
                if self.query_contains_partial {
                    self.set_error_context(
                        term,
                        error::RuntimeError::Unsupported {
                            msg: "cannot use cut with partial evaluation".to_string(),
                        },
                    )
                } else {
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

                    self.cut(choice_index)
                }
            }
            Isa => {
                // TODO (dhatch): Use query op helper.
                assert_eq!(args.len(), 2);
                self.isa(&args[0], &args[1])?.query_event_none()
            }
            ForAll => {
                assert_eq!(args.len(), 2);
                let action = args.pop().unwrap();
                let condition = args.pop().unwrap();
                // For all is implemented as !(condition, !action).
                let op = Operation {
                    operator: Not,
                    args: vec![term.clone_with_value(Value::Expression(Operation {
                        operator: And,
                        args: vec![
                            condition,
                            term.clone_with_value(Value::Expression(Operation {
                                operator: Not,
                                args: vec![action],
                            })),
                        ],
                    }))],
                };
                self.query(&term.clone_with_value(Value::Expression(op)))
            }
        }
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
        let Operation { operator: op, args } = term.value().as_expression().unwrap();

        let mut args = args.clone();
        assert!(args.len() >= 2);
        let (left, right) = (&args[0], &args[1]);

        match (left.value(), right.value()) {
            (Value::Expression(_), _)
            | (_, Value::Expression(_))
            | (Value::RestVariable(_), _)
            | (_, Value::RestVariable(_)) => {
                panic!("invalid query");
            }
            _ => {}
        };

        if let Value::Variable(r) = right.value() {
            if let VariableState::Bound(x) = self.variable_state(r) {
                args[1] = x;
                return self.query(&term.clone_with_value(Value::Expression(Operation {
                    operator: *op,
                    args,
                })));
            } else if !handle_unbound_right_var && left.value().as_symbol().is_err() {
                return eval(self, term);
            }
        }

        if let Value::Variable(l) = left.value() {
            if let VariableState::Bound(x) = self.variable_state(l) {
                args[0] = x;
                return self.query(&term.clone_with_value(Value::Expression(Operation {
                    operator: *op,
                    args,
                })));
            } else if !handle_unbound_left_var && right.value().as_symbol().is_err() {
                return eval(self, term);
            }
        }

        if left.value().as_symbol().is_ok() || right.value().as_symbol().is_ok() {
            return self.add_constraint(term)?.query_event_none();
        }

        eval(self, term)
    }

    /// Evaluate comparison operations.
    fn comparison_op_helper(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        let Operation { operator: op, args } = term.value().as_expression().unwrap();

        assert_eq!(args.len(), 2);
        let (left, right) = (&args[0], &args[1]);
        match (left.value(), right.value()) {
            (Value::ExternalInstance(_), _) | (_, Value::ExternalInstance(_)) => {
                // Generate a symbol for the external result and bind to `false` (default).
                let (call_id, answer) = self.new_call_var("external_op_result", false.into());

                // Check that the external result is `true` when we return.
                self.push_goal(Goal::Unify(answer, term!(true)))?;

                // Emit an event for the external operation.
                Ok(QueryEvent::ExternalOp {
                    call_id,
                    operator: *op,
                    args: vec![left.clone(), right.clone()],
                })
            }
            _ => if !compare(*op, left, right)? {
                self.backtrack()?
            } else {
                self
            }
            .query_event_none(),
        }
    }

    // TODO(ap, dhatch): Rewrite 3-arg arithmetic ops as 2-arg + unify,
    // like we do for dots; e.g., `+(a, b, c)` → `c = +(a, b)`.
    /// Evaluate arithmetic operations.
    fn arithmetic_op_helper(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        let Operation { operator: op, args } = term.value().as_expression().unwrap();

        assert_eq!(args.len(), 3);
        let left = &args[0];
        let right = &args[1];
        let result = &args[2];
        assert!(matches!(result.value(), Value::Variable(_)));

        use Operator::*;
        match (left.value(), right.value()) {
            (Value::Number(left), Value::Number(right)) => {
                if let Some(answer) = match op {
                    Add => *left + *right,
                    Sub => *left - *right,
                    Mul => *left * *right,
                    Div => *left / *right,
                    Mod => (*left).modulo(*right),
                    Rem => *left % *right,
                    _ => {
                        return self.set_error_context(
                            term,
                            error::RuntimeError::Unsupported {
                                msg: format!("numeric operation {}", op.to_polar()),
                            },
                        );
                    }
                } {
                    self.unify(&Term::from(Value::Number(answer)), result)?
                        .query_event_none()
                } else {
                    self.set_error_context(
                        term,
                        error::RuntimeError::ArithmeticError {
                            msg: term.to_polar(),
                        },
                    )
                }
            }
            (_, _) => self.set_error_context(
                term,
                error::RuntimeError::Unsupported {
                    msg: format!("unsupported arithmetic operands: {}", term.to_polar()),
                },
            ),
        }
    }

    /// Push appropriate goals for lookups on dictionaries and instances.
    fn dot_op_helper(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        let Operation { operator: op, args } = term.value().as_expression().unwrap();
        assert_eq!(*op, Operator::Dot, "expected a dot operation");

        let mut args = args.clone();
        assert_eq!(args.len(), 3);
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
                })
            }
            // Push an `ExternalLookup` goal for external instances and built-ins.
            Value::Dictionary(_)
            | Value::ExternalInstance(_)
            | Value::List(_)
            | Value::Number(_)
            | Value::String(_) => {
                let answer = self.kb().gensym("lookup_value");
                let call_id = self.new_call_id(&answer);
                self.append_goals(vec![
                    Goal::LookupExternal {
                        call_id,
                        field: field.clone(),
                        instance: object.clone(),
                    },
                    Goal::CheckError,
                    Goal::Unify(value.clone(), Term::from(answer)),
                ])
            }
            Value::Variable(v) => {
                if matches!(field.value(), Value::Call(_)) {
                    self.set_error_context(
                        object,
                        error::RuntimeError::Unsupported {
                            msg: format!("cannot call method on unbound variable {}", v),
                        },
                    )
                } else {
                    //                    let sym = var!(format!("__{}_dot_{}", v, field.value().as_string().unwrap()));
                    // Translate `.(object, field, value)` → `value = .(object, field)`.
                    let dot2 = term!(op!(Dot, object.clone(), field.clone()));
                    let term = term!(op!(Unify, value.clone(), dot2));
                    //      let term = term!(op!(And, term, term!(op!(Unify, sym, value.clone()))));
                    self.add_constraint(&term)
                }
            }
            _ => self.type_error(
                object,
                format!(
                    "can only perform lookups on dicts and instances, this is {}",
                    object.to_polar()
                ),
            ),
        }?
        .query_event_none()
    }

    fn in_op_helper(&mut self, term: &Term) -> PolarResult<QueryEvent> {
        let Operation { args, .. } = term.value().as_expression().unwrap();

        assert_eq!(args.len(), 2);
        let item = &args[0];
        let iterable = &args[1];
        let item_is_ground = item.is_ground();

        match iterable.value() {
            // Unify item with each element of the list, skipping non-matching ground terms.
            Value::List(terms) => self.choose(terms.iter().filter_map(|term| {
                (!item_is_ground || !term.is_ground() || term.value() == item.value()).then(|| {
                    match term.value() {
                        Value::RestVariable(v) => {
                            let term = op!(In, item.clone(), Term::from(v.clone())).into();
                            vec![Goal::Query(term)]
                        }
                        _ => vec![Goal::Unify(item.clone(), term.clone())],
                    }
                })
            })),
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
                    .map(|term| vec![Goal::Unify(item.clone(), term)]),
            ),
            // Unify item with each element of the string
            // FIXME (gw): this seems strange, wouldn't a substring search make more sense?
            Value::String(s) => self.choose(s.chars().filter_map(|c| {
                let c = Value::String(c.to_string());
                (!item_is_ground || c == *item.value())
                    .then(|| vec![Goal::Unify(item.clone(), iterable.clone_with_value(c))])
            })),
            // Push an `ExternalLookup` goal for external instances
            Value::ExternalInstance(_) => {
                // Generate symbol for next result and leave the variable unbound, so that unification with the result does not fail
                // Unification of the `next_sym` variable with the result of `NextExternal` happens in `fn external_call_result()`
                // `external_call_result` is the handler for results from both `LookupExternal` and `NextExternal`, so neither can bind the
                // call ID variable to `false`.
                let next_sym = self.kb().gensym("next_value");
                let call_id = self.new_call_id(&next_sym);

                // append unify goal to be evaluated after
                // next result is fetched
                self.append_goals(vec![
                    Goal::NextExternal {
                        call_id,
                        iterable: self.deref(iterable),
                    },
                    Goal::Unify(item.clone(), Term::from(next_sym)),
                ])
            }
            _ => self.type_error(
                iterable,
                format!(
                    "can only use `in` on an iterable value, this is {:?}",
                    iterable.value()
                ),
            ),
        }?
        .query_event_none()
    }

    /// Unify `left` and `right` terms.
    ///
    /// Outcomes of a unification are:
    ///  - Successful unification => bind zero or more variables to values
    ///  - Recursive unification => more `Unify` goals are pushed onto the stack
    ///  - Failure => backtrack
    fn unify(&mut self, left: &Term, right: &Term) -> PolarResult<&mut Self> {
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
                        self.push_goal(Goal::Query(term))
                    }
                    // otherwise this should never happen.
                    _ => self.type_error(
                        left,
                        format!(
                            "cannot unify expressions directly `{}` = `{}`",
                            left.to_polar(),
                            right.to_polar()
                        ),
                    ),
                }
            }
            (Value::Pattern(_), _) | (_, Value::Pattern(_)) => self.type_error(
                left,
                format!(
                    "cannot unify patterns directly `{}` = `{}`",
                    left.to_polar(),
                    right.to_polar()
                ),
            ),

            // Unify two variables.
            // TODO(gj): (Var, Rest) + (Rest, Var) cases might be unreachable.
            (Value::Variable(l), Value::Variable(r))
            | (Value::Variable(l), Value::RestVariable(r))
            | (Value::RestVariable(l), Value::Variable(r))
            | (Value::RestVariable(l), Value::RestVariable(r)) => {
                if l == r {
                    Ok(self)
                } else {
                    match (self.variable_state(l), self.variable_state(r)) {
                        (VariableState::Bound(x), VariableState::Bound(y)) => self.unify(&x, &y),
                        _ if self.bind(l, right.clone()).is_err() => self.backtrack(),
                        _ => Ok(self),
                    }
                }
            }

            // Unify/bind a variable on the left with/to the term on the right.
            (Value::Variable(var), _) | (Value::RestVariable(var), _) => {
                let right = right.clone();
                match self.variable_state(var) {
                    VariableState::Bound(value) => self.unify(&value, &right),
                    _ if self.bind(var, right).is_err() => self.backtrack(),
                    _ => Ok(self),
                }
            }

            // Unify/bind a variable on the right with/to the term on the left.
            (_, Value::Variable(var)) | (_, Value::RestVariable(var)) => {
                let left = left.clone();
                match self.variable_state(var) {
                    VariableState::Bound(value) => self.push_goal(Goal::Unify(left, value)),
                    _ if self.bind(var, left).is_err() => self.backtrack(),
                    _ => Ok(self),
                }
            }

            // Unify predicates like unifying heads
            (Value::Call(left), Value::Call(right))
                if left.name == right.name && left.args.len() == right.args.len() =>
            {
                // Handled in the parser.
                assert!(left.kwargs.is_none());
                assert!(right.kwargs.is_none());
                self.append_goals(
                    left.args
                        .iter()
                        .zip(right.args.iter())
                        .map(|(left, right)| Goal::Unify(left.clone(), right.clone())),
                )
            }

            // Unify lists by recursively unifying their elements.
            (Value::List(l), Value::List(r)) => self.unify_lists(Goal::Unify, l, r),

            (Value::Dictionary(left), Value::Dictionary(right)) => {
                // Check that the set of keys are the same.
                let lfs = left.fields.keys().collect::<HashSet<_>>();
                let rfs = right.fields.keys().collect();
                if lfs != rfs {
                    self.backtrack()
                } else {
                    left.fields.iter().fold(Ok(self), |this, (k, v)| {
                        let right = right.fields.get(k).unwrap();
                        this?.unify(v, right)
                    })
                }
            }

            // Unify integers by value.
            (Value::Number(left), Value::Number(right)) if left == right => Ok(self),
            (Value::String(left), Value::String(right)) if left == right => Ok(self),
            (Value::Boolean(left), Value::Boolean(right)) if left == right => Ok(self),

            (
                Value::ExternalInstance(ExternalInstance {
                    instance_id: left, ..
                }),
                Value::ExternalInstance(ExternalInstance {
                    instance_id: right, ..
                }),
            ) if left == right => Ok(self),

            // If either operand is an external instance, let the host
            // compare them for equality. This handles unification between
            // "equivalent" host and native types transparently.
            (Value::ExternalInstance(_), _) | (_, Value::ExternalInstance(_)) => {
                self.push_goal(Goal::Query(Term::from(Operation {
                    operator: Operator::Eq,
                    args: vec![left.clone(), right.clone()],
                })))
            }

            // Anything else fails.
            _ => self.backtrack(),
        }
    }

    /// "Unify" two lists element-wise, respecting rest-variables.
    /// Used by both `unify` and `isa`; hence the first, a goal
    /// constructor.
    fn unify_lists<'a, G>(
        &mut self,
        goal: G,
        l: &'a [Term],
        r: &'a [Term],
    ) -> PolarResult<&mut Self>
    where
        G: Fn(Term, Term) -> Goal,
    {
        self.join_lists(goal, vec![], l.iter(), r.iter())
    }

    /// Zip two lists into a list of goals, respecting rest variables.
    fn join_lists<'a, G, I>(
        &mut self,
        goal: G,
        mut goals: Vec<Goal>,
        mut l: I,
        mut r: I,
    ) -> PolarResult<&mut Self>
    where
        G: Fn(Term, Term) -> Goal,
        I: Iterator<Item = &'a Term>,
    {
        use std::iter::once;
        use Value::{List, RestVariable, Variable};
        let cons = |x, i| term!(List(once(x).chain(i).cloned().collect()));
        let rest = |y: &Symbol| term!(Variable(y.clone()));

        match (l.next(), r.next()) {
            // both are empty, append the collected goals
            (None, None) => self.append_goals(goals),
            // one list is shorter. is there a rest variable in the other one?
            (Some(v), None) | (None, Some(v)) => match v.value() {
                // if so, bind it to the empty list.
                RestVariable(y) => {
                    goals.push(goal(term!(Variable(y.clone())), term!(vec![])));
                    self.append_goals(goals)
                }
                // otherwise fail.
                _ => self.backtrack(),
            },
            // got an item off of each list.
            (Some(l0), Some(r0)) => match (l0.value(), r0.value()) {
                // are there rest variables? then we can return.
                (RestVariable(_), RestVariable(_)) => {
                    goals.push(goal(l0.clone(), r0.clone()));
                    self.append_goals(goals)
                }
                (RestVariable(ll), _) => {
                    goals.push(goal(rest(ll), cons(r0, r)));
                    self.append_goals(goals)
                }
                (_, RestVariable(rr)) => {
                    goals.push(goal(cons(l0, l), rest(rr)));
                    self.append_goals(goals)
                }
                // otherwise just push a normal goal.
                _ => {
                    goals.push(goal(l0.clone(), r0.clone()));
                    self.join_lists(goal, goals, l, r)
                }
            },
        }
    }

    /// Filter rules to just those applicable to a list of arguments,
    /// then sort them by specificity.
    fn filter_rules(
        &mut self,
        mut applicable_rules: Rules,
        mut unfiltered_rules: Rules,
        args: &[Term],
    ) -> PolarResult<&mut Self> {
        if unfiltered_rules.is_empty() {
            return self.sort_rules(applicable_rules.into_iter().rev().collect(), args, 1, 1);
        }
        let rule = unfiltered_rules.pop().unwrap();
        if rule.params.len() != args.len() {
            self.filter_rules(applicable_rules, unfiltered_rules, args) // wrong arity, filter out
        } else if rule.is_ground() {
            applicable_rules.push(rule);
            self.filter_rules(applicable_rules, unfiltered_rules, args)
        } else {
            use Goal::*;
            let inapplicable = FilterRules {
                args: args.to_vec(),
                applicable_rules: applicable_rules.clone(),
                unfiltered_rules: unfiltered_rules.clone(),
            };
            let Rule { params, .. } = self.rename_rule_vars(&rule);
            applicable_rules.push(rule);
            let applicable = FilterRules {
                args: args.to_vec(),
                applicable_rules: applicable_rules.clone(),
                unfiltered_rules: unfiltered_rules.clone(),
            };

            // Rename the variables in the rule (but not the args).
            // This avoids clashes between arg vars and rule vars.
            let mut check_applicability = vec![];
            for (arg, param) in args.iter().zip(params.iter()) {
                check_applicability.push(Unify(arg.clone(), param.parameter.clone()));
                if let Some(specializer) = &param.specializer {
                    check_applicability.push(Isa(arg.clone(), specializer.clone()));
                }
            }
            self.choose_conditional(check_applicability, vec![applicable], vec![inapplicable])
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
    fn sort_rules(
        &mut self,
        rules: Rules,
        args: &[Term],
        outer: usize,
        inner: usize,
    ) -> PolarResult<&mut Self> {
        use Goal::*;
        if rules.is_empty() {
            self.backtrack()
        } else if outer >= rules.len() {
            self.call_rules(rules, args) // finished
        } else if inner == 0 {
            self.sort_rules(rules, args, outer + 1, outer + 1)
        } else {
            let compare = IsMoreSpecific {
                left: rules[inner].clone(),
                right: rules[inner - 1].clone(),
                args: args.to_vec(),
            };

            let mut sw = rules.clone();
            sw.swap(inner - 1, inner);
            let next_inner = SortRules {
                rules: sw,
                outer,
                inner: inner - 1,
                args: args.to_vec(),
            };

            let next_outer = SortRules {
                rules,
                args: args.to_vec(),
                outer: outer + 1,
                inner: outer + 1,
            };

            // If the comparison fails, break out of the inner loop.
            // If the comparison succeeds, continue the inner loop with the swapped rules.
            self.choose_conditional(vec![compare], vec![next_inner], vec![next_outer])
        }
    }

    fn call_rules(&mut self, rules: Rules, args: &[Term]) -> PolarResult<&mut Self> {
        use Goal::*;
        // We're done; the rules are sorted.
        // Make alternatives for calling them.
        self.polar_log_mute = false;
        self.log_with(
            || {
                let mut rule_strs = "APPLICABLE_RULES:".to_owned();
                for rule in rules.iter() {
                    rule_strs.push_str(&format!("\n  {}", rule.to_polar()));
                }
                rule_strs
            },
            &[],
        );

        let alternatives = rules
            .iter()
            .map(|rule| {
                let Rule { body, params, .. } = self.rename_rule_vars(rule);
                let mut goals = vec![
                    TraceRule(Rc::new(Trace {
                        node: Node::Rule(rule.clone()),
                        children: vec![],
                    })),
                    TraceStackPush,
                ];

                // Unify the arguments with the formal parameters.
                for (arg, param) in args.iter().zip(params.iter()) {
                    goals.push(Unify(arg.clone(), param.parameter.clone()));
                    if let Some(specializer) = &param.specializer {
                        goals.push(Isa(param.parameter.clone(), specializer.clone()));
                    }
                }

                // Query for the body clauses.
                goals.push(Query(body));
                goals.push(TraceStackPop);
                goals
            })
            .collect::<Vec<_>>(); // call the closures so self isn't borrowed

        // Choose the first alternative, and push a choice for the rest.
        self.choose(alternatives)
    }

    /// Succeed if `left` is more specific than `right` with respect to `args`.
    fn is_more_specific(
        &mut self,
        left: &Rule,
        right: &Rule,
        args: &[Term]
    ) -> PolarResult<&mut Self> {
        let zipped = left.params.iter().zip(right.params.iter()).zip(args.iter());
        for ((left_param, right_param), arg) in zipped {
            match (&left_param.specializer, &right_param.specializer) {
                // If neither has a specializer, neither is more specific, so we continue to the next argument.
                (None, None) => continue,
                // If the left rule has no specializer and the right does, it is NOT more specific,
                // so we Backtrack (fail)
                (None, Some(_)) => break,
                // If the left rule has a specializer and the right does not, the left IS more specific,
                // so we return
                (Some(_), None) => return Ok(self),

                // If both specs are unions, they have the same specificity regardless of whether
                // they're the same or different unions.
                // TODO(gj): when we have unions beyond `Actor` and `Resource`, we'll need to be
                // smarter about this check since UnionA is more specific than UnionB if UnionA is
                // a member of UnionB.
                (Some(l), Some(r)) if l.is_union() && r.is_union() => continue,
                // If left is a union and right is not, left cannot be more specific, so we backtrack.
                (Some(l), Some(_)) if l.is_union() => break,
                // If right is a union and left is not, left IS more specific, so we return.
                (Some(_), Some(r)) if r.is_union() => return Ok(self),
                (Some(l), Some(r)) if l == r => continue,
                (Some(left_spec), Some(right_spec)) => {
                    // If you find two non-equal specializers, that comparison determines the relative
                    // specificity of the two rules completely. As soon as you have two specializers
                    // that aren't the same and you can compare them and ask which one is more specific
                    // to the relevant argument, you're done.
                    let answer = self.kb().gensym("is_subspecializer");
                    // Bind answer to false as a starting point in case is subspecializer doesn't
                    // bind any result.
                    // This is done here for safety to avoid a bug where `answer` is unbound by
                    // `IsSubspecializer` and the `Unify` Goal just assigns it to `true` instead
                    // of checking that is is equal to `true`.
                    return self.bind(&answer, Term::from(false))?.append_goals(vec![
                        Goal::IsSubspecializer {
                            answer: answer.clone(),
                            left: left_spec.clone(),
                            right: right_spec.clone(),
                            arg: arg.clone(),
                        },
                        Goal::Unify(term!(answer), term!(true)),
                    ]);
                }
            }
        }
        // Fail on any of the above branches that do not return
        self.backtrack()
    }

    /// Determine if `left` is a more specific specializer ("subspecializer") than `right`
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
                self.rebind_external_answer(answer, term!(true));
                Ok(QueryEvent::None)
            }
            _ => {
                self.rebind_external_answer(answer, term!(false));
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

    fn set_error_context<A>(
        &self,
        term: &Term,
        error: impl Into<error::PolarError>,
    ) -> PolarResult<A> {
        Err(self.kb().set_error_context(term, error))
    }

    fn type_error<A>(&self, term: &Term, msg: String) -> PolarResult<A> {
        let stack_trace = self.stack_trace();
        let error = error::RuntimeError::TypeError {
            msg,
            stack_trace: Some(stack_trace),
        };
        self.set_error_context(term, error)
    }

    fn run_runnable(&mut self, runnable: Box<dyn Runnable>) -> PolarResult<QueryEvent> {
        let (call_id, answer) = self.new_call_var("runnable_result", Value::Boolean(false));
        self.push_goal(Goal::Unify(answer, term!(true)))?;

        Ok(QueryEvent::Run { runnable, call_id })
    }

    /// Handle an error coming from outside the vm.
    pub fn external_error(&mut self, message: String) -> PolarResult<()> {
        self.external_error = Some(message);
        Ok(())
    }

    /// VM main loop entry point.
    fn run_to_query_event(&mut self) -> PolarResult<QueryEvent> {
        if !self.goals.is_empty() {
            self.goal_loop()
        } else if !self.choices.is_empty() {
            self.backtrack()?.goal_loop()
        } else {
            self.query_event_done(true)
        }
    }

    /// VM main loop. It'd be nice to make this recursive but rustc
    /// has trouble TCOing it :(
    fn goal_loop(&mut self) -> PolarResult<QueryEvent> {
        while let Some(goal) = self.goals.pop() {
            match self.do_goal(goal.clone())? {
                QueryEvent::None => {
                    self.maybe_break(DebugEvent::Goal(goal.clone()))?;
                }
                event => {
                    self.external_error = None;
                    return Ok(event);
                }
            }
        }
        self.finish_goal_loop()
    }

    /// VM results output
    fn finish_goal_loop(&mut self) -> PolarResult<QueryEvent> {
        if self.log {
            self.print("⇒ result");
            if self.tracing {
                for t in &self.trace {
                    self.print(&format!("trace\n{}", t.draw(self)));
                }
            }
        }

        let trace = if !self.tracing {
            None
        } else {
            self.trace.first().cloned().map(|trace| TraceResult {
                formatted: trace.draw(self),
                trace,
            })
        };

        let mut bindings = self.bindings(true);
        use crate::partial::{simplify_bindings, sub_this};
        if !self.inverting {
            match simplify_bindings(bindings, false) {
                None => return self.query_event_none(),
                Some(bs) => {
                    bindings = bs
                        .into_iter()
                        .map(|(var, value)| (var.clone(), sub_this(var, value)))
                        .collect()
                }
            }
        }
        Ok(QueryEvent::Result { bindings, trace })
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
        self.run_to_query_event()
    }

    fn handle_error(&mut self, error: PolarError) -> PolarResult<QueryEvent> {
        // if we pushed a debug goal, push an error goal underneath it.
        if self.maybe_break(DebugEvent::Error(error.clone()))? {
            let g = self.goals.pop().unwrap();
            self.push_goal(Goal::Error(error))?;
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
            self.log_with(|| format!("=> {}", value.to_string()), &[]);

            // Fetch variable to unify with call result.
            let sym = self.get_call_sym(call_id).to_owned();

            self.push_goal(Goal::Unify(term!(sym), value))?;
        } else {
            self.log("=> No more results.", &[]);

            // No more results. Clean up, cut out the retry alternative,
            // and backtrack.
            self.call_id_symbols.remove(&call_id).expect("bad call ID");

            let check_error = if let Some(goal) = self.goals.last() {
                matches!(*(*goal), Goal::CheckError)
            } else {
                false
            };

            self.push_goal(Goal::Backtrack)?;
            self.push_goal(Goal::Cut(self.choices.len() - 1))?;

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
    use permute::permute;

    use super::*;
    use crate::rewrites::unwrap_and;

    impl PolarVirtualMachine {
        /// Return true if there is nothing left to do.
        fn is_halted(&self) -> bool {
            self.goals.is_empty() && self.choices.is_empty()
        }
        fn set_stack_limit(&mut self, limit: usize) {
            self.stack_limit = limit;
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
            Goal::Query(term!($term))
        };
        ($($term:expr),+) => {
            Goal::Query(term!(op!(And, $($term),+)))
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
        ($vm:ident, [$( $pattern:pat )|+ $( if $guard: expr )?]) => {
            assert!(matches!($vm.run(None).unwrap(), $($pattern)|+ $(if $guard)?));
            assert_query_events!($vm, []);
        };
        ($vm:ident, [$( $pattern:pat )|+ $( if $guard: expr )?, $($tail:tt)*]) => {
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
        let parts = vec![f1, f2, f3];
        for permutation in permute(parts) {
            vm.push_goal(Goal::Query(Term::new_from_test(Value::Expression(
                Operation {
                    operator: Operator::And,
                    args: permutation,
                },
            ))))
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
        vm.push_goal(Goal::Isa(empty_list.clone(), empty_list.clone()))
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
        vm.push_goal(Goal::Isa(one_two_list.clone(), one_two_list.clone()))
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
        vm.push_goal(Goal::Isa(one_two_list.clone(), two_one_list))
            .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [1] isNOTa [1,2]
        vm.push_goal(Goal::Isa(one_list.clone(), one_two_list.clone()))
            .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [1,2] isNOTa [1]
        vm.push_goal(Goal::Isa(one_two_list.clone(), one_list.clone()))
            .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [1] isNOTa []
        vm.push_goal(Goal::Isa(one_list.clone(), empty_list.clone()))
            .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [] isNOTa [1]
        vm.push_goal(Goal::Isa(empty_list, one_list.clone()))
            .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [1] isNOTa 1
        vm.push_goal(Goal::Isa(one_list.clone(), one.clone()))
            .unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // 1 isNOTa [1]
        vm.push_goal(Goal::Isa(one, one_list)).unwrap();
        assert!(matches!(
            vm.run(None).unwrap(),
            QueryEvent::Done { result: true }
        ));
        assert!(vm.is_halted());

        // [1,2] isa [1, *rest]
        vm.push_goal(Goal::Isa(
            one_two_list,
            term!([1, Value::RestVariable(sym!("rest"))]),
        ))
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
        vm.push_goal(Goal::Isa(dict.clone(), dict_pattern.clone()))
            .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done { result: true }]);

        // Dicts with identical keys and different values DO NOT isa.
        let different_dict_pattern = term!(pattern!(btreemap! {
            sym!("x") => term!(2),
            sym!("y") => term!(1),
        }));
        vm.push_goal(Goal::Isa(dict.clone(), different_dict_pattern))
            .unwrap();
        assert_query_events!(vm, [QueryEvent::Done { result: true }]);

        let empty_dict = term!(btreemap! {});
        let empty_dict_pattern = term!(pattern!(btreemap! {}));
        // {} isa {}.
        vm.push_goal(Goal::Isa(empty_dict.clone(), empty_dict_pattern.clone()))
            .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done { result: true }]);

        // Non-empty dicts should isa against an empty dict.
        vm.push_goal(Goal::Isa(dict.clone(), empty_dict_pattern))
            .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done { result: true }]);

        // Empty dicts should NOT isa against a non-empty dict.
        vm.push_goal(Goal::Isa(empty_dict, dict_pattern.clone()))
            .unwrap();
        assert_query_events!(vm, [QueryEvent::Done { result: true }]);

        let subset_dict_pattern = term!(pattern!(btreemap! {sym!("x") => term!(1)}));
        // Superset dict isa subset dict.
        vm.push_goal(Goal::Isa(dict, subset_dict_pattern)).unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done { result: true }]);

        // Subset dict isNOTa superset dict.
        let subset_dict = term!(btreemap! {sym!("x") => term!(1)});
        vm.push_goal(Goal::Isa(subset_dict, dict_pattern)).unwrap();
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
        vm.push_goal(Goal::Unify(left.clone(), right)).unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done { result: true }]);

        // Dicts with identical keys and different values DO NOT unify.
        let right = term!(btreemap! {
            sym!("x") => term!(2),
            sym!("y") => term!(1),
        });
        vm.push_goal(Goal::Unify(left.clone(), right)).unwrap();
        assert_query_events!(vm, [QueryEvent::Done { result: true }]);

        // Empty dicts unify.
        vm.push_goal(Goal::Unify(term!(btreemap! {}), term!(btreemap! {})))
            .unwrap();
        assert_query_events!(vm, [QueryEvent::Result { hashmap!() }, QueryEvent::Done { result: true }]);

        // Empty dict should not unify against a non-empty dict.
        vm.push_goal(Goal::Unify(left.clone(), term!(btreemap! {})))
            .unwrap();
        assert_query_events!(vm, [QueryEvent::Done { result: true }]);

        // Subset match should fail.
        let right = term!(btreemap! {
            sym!("x") => term!(1),
        });
        vm.push_goal(Goal::Unify(left, right)).unwrap();
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
        vm.push_goal(Goal::Unify(left, right)).unwrap();
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
            vec![Goal::Debug("Hello".to_string())],
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
            vec![Goal::Unify(vars, vals)],
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
        vm.append_goals(vec![Goal::Unify(term!(x.clone()), term!(y))])
            .unwrap();
        let _ = vm.run(None).unwrap();
        assert_eq!(vm.deref(&term!(x)), one);
        vm.backtrack().unwrap();

        // Left variable bound to value.
        vm.bind(&z, one.clone()).unwrap();
        vm.append_goals(vec![Goal::Unify(term!(z.clone()), one.clone())])
            .unwrap();
        let _ = vm.run(None).unwrap();
        assert_eq!(vm.deref(&term!(z.clone())), one);

        // Left variable bound to value, unify with something else, backtrack.
        vm.append_goals(vec![Goal::Unify(term!(z.clone()), two)])
            .unwrap();
        let _ = vm.run(None).unwrap();
        assert_eq!(vm.deref(&term!(z)), one);
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
            source_info: SourceInfo::Test,
        };

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

        let answer = vm.kb().gensym("is_subspecializer");

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
                        err,
                        error::PolarError {
                            kind: error::ErrorKind::Runtime(
                                error::RuntimeError::QueryTimeout { .. }
                            ),
                            ..
                        }
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
        let _ = vm.do_goal(Rc::new(query!(call!("bar", [value!([sym!("x")])]))));
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
        let consequent = Goal::Debug("consequent".to_string());
        let alternative = Goal::Debug("alternative".to_string());

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
                Goal::Unify(term!(sym!("x")), term!(true)),
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
}
