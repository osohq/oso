use std::rc::Rc;

use super::error::PolarResult;
use super::formatting::{source_lines, ToPolarString};
use super::sources::*;
use super::terms::*;
use super::traces::*;

use super::vm::*;

impl PolarVirtualMachine {
    pub fn query_summary(&self, query: &Term) -> String {
        let relevant_bindings = self.relevant_bindings(&[&query]);
        let bindings_str = relevant_bindings
            .iter()
            .map(|(var, val)| format!("{} = {}", var.0, val.to_polar()))
            .collect::<Vec<String>>()
            .join(", ");
        let query_str = query.to_polar();
        format!("QUERY: {}, BINDINGS: {{{}}}", query_str, bindings_str)
    }

    /// Drive debugger.
    pub fn debug_command(&mut self, command: &str) -> PolarResult<()> {
        let mut debugger = self.debugger.clone();
        let maybe_goal = debugger.debug_command(command, self);
        if let Some(goal) = maybe_goal {
            self.push_goal(goal)?;
        }
        self.debugger = debugger;
        Ok(())
    }

    /// If the inner [`Debugger`](struct.Debugger.html) returns a [`Goal`](../vm/enum.Goal.html),
    /// push it onto the goal stack.
    pub fn maybe_break(&mut self, event: DebugEvent) -> PolarResult<()> {
        let maybe_goal = self.debugger.maybe_break(event, self);
        if let Some(goal) = maybe_goal {
            self.push_goal((*goal).clone())?;
        }
        Ok(())
    }
}

/// [`Debugger`](struct.Debugger.html) step granularity.
#[derive(Clone, Debug)]
enum Step {
    /// Pause after evaluating the next [`Goal`](../vm/enum.Goal.html).
    Goal,

    /// Step **over** goals until reaching the next sibling [`Goal::Query`](../vm/enum.Goal.html).
    /// This is not necessarily the next [`Goal::Query`](../vm/enum.Goal.html), but rather the next
    /// [`Goal::Query`](../vm/enum.Goal.html) where the query stack sans that query is identical to
    /// the current query stack sans the current query. For example, when the `debug()` predicate
    /// is evaluated in the following snippet of Polar...
    ///
    /// ```polar
    /// a() if debug() and b();
    /// b();
    /// ?= a()
    /// ```
    ///
    /// ...the query stack will look as follows:
    ///
    /// ```text
    /// a()          # Head of rule a().
    /// debug(), b() # Body of rule a().
    /// debug()      # First term in the body of rule a().
    /// ```
    ///
    /// If the user wants to jump **over** `debug()` (the current query) to arrive at `b()` (the
    /// next query in the body of `a()`), evaluate goals until the query stack looks as follows:
    ///
    /// ```text
    /// a()          # Head of rule a().
    /// debug(), b() # Body of rule a().
    /// b()          # Second term in the body of rule a().
    /// ```
    // Over {
    //     /// Snapshot of the current query stack sans the current query.
    //     snapshot: Queries,
    // },
    /// Step **out** of the current parent query, evaluating goals until reaching the
    /// [`Goal::Query`](../vm/enum.Goal.html) for its next sibling.
    ///
    /// To illustrate this movement, let's step through the queries for the following snippet of
    /// polar:
    ///
    /// ```text
    /// a() if b() and c();
    /// b() if debug() and d();
    /// c();
    /// d();
    /// ?= a()
    /// ```
    ///
    /// First we query for the head of `a()`, then the body of `a()`, and then `b()`, the first
    /// clause in the body of `a()`. Next we query for the body of `b()` and the first clause in
    /// the body of `b()`, a `debug()` predicate. By the time we reach that `debug()` predicate,
    /// the query stack looks like this:
    ///
    /// ```text
    /// a()          # head of a()
    /// b(), c()     # body of a()
    /// b()          # first clause in body of a() / head of b()
    /// debug(), d() # body of b()
    /// debug()      # first clause in body of b()
    /// ```
    ///
    /// From our current position (evaluating the `debug()` predicate in the body of `b()`), we
    /// would [`Step::Out`](enum.Step.html) if we wanted to continue evaluating goals until
    /// reaching `c()` in the body of `a()`. We're stepping entirely **out** of the current parent
    /// query, `b()`, to arrive at its next sibling, `c()`. When we arrive at the
    /// [`Goal::Query`](../vm/enum.Goal.html) for `c()`, the query stack will look as follows:
    ///
    /// ```text
    /// a()          # head of a()
    /// b(), c()     # body of a()
    /// c()          # second clause in body of a() / head of c()
    /// ```
    ///
    /// The query stack above `c()` is identical to the previous stack snippet above `b()`, which
    /// makes sense since they share a parent -- the query for the body of `a()`. That gives us our
    /// test for stepping **out**:
    ///
    /// - Store a slice of the current query stack *without the last three queries* (current query,
    ///   body of current rule, and head of current rule).
    /// - Evaluate goals until the current query stack *without the current query* matches the
    //   stored slice.
    // Out {
    //     /// Snapshot of the current query stack sans its last three queries.
    //     snapshot: Queries,
    // },
    Over {
        level: usize,
    }, // break on query if we haven't tracestackpushed, otherwise break when we pop back to this level.
    InTo, // break on any trace.push, break on query
    // break on TraceStackPop
    Out {
        level: usize,
    },
}

/// VM breakpoints.
///
/// There are currently two breakpoints in the VM, one that fires after every
/// [`Goal`](../vm/enum.Goal.html) and another that fires before every
/// [`Goal::Query`](../vm/enum.Goal.html). When either breakpoint is hit, we check the
/// [`Debugger`](struct.Debugger.html)'s internal [`step`](struct.Debugger.html#structfield.step)
/// field to determine how evaluation should proceed.
#[derive(Clone, Debug)]
pub enum DebugEvent {
    Goal(Rc<Goal>),
    Query,
    Pop,
}

/// Tracks internal debugger state.
#[derive(Clone, Debug, Default)]
pub struct Debugger {
    /// Next stopping point, as set by the user.
    ///
    /// - `None`: Don't stop.
    /// - `Some(step)`: View the stopping logic in
    ///   [`maybe_break`](struct.Debugger.html#method.maybe_break).
    step: Option<Step>,
}

impl Debugger {
    /// Retrieve the original source line (and, optionally, additional lines of context) for the
    /// current query.
    fn query_source(&self, query: &Term, sources: &Sources, num_lines: usize) -> String {
        query
            .get_source_id()
            .and_then(|id| sources.get_source(id))
            .map_or_else(
                || "".to_string(),
                |source| source_lines(&source, query.offset(), num_lines),
            )
    }

    /// When the [`VM`](../vm/struct.PolarVirtualMachine.html) hits a breakpoint, check if
    /// evaluation should pause.
    ///
    /// The check is a comparison of the [`Debugger`](struct.Debugger.html)'s
    /// [`step`](struct.Debugger.html#structfield.step) field with the passed-in
    /// [`DebugEvent`](enum.DebugEvent.html). If [`step`](struct.Debugger.html#structfield.step) is
    /// set to `None`, evaluation continues. For details about how the `Some()` values of
    /// [`step`](struct.Debugger.html#structfield.step) are handled, see the explanations in the
    /// [`Step`](enum.Step.html) documentation.
    ///
    /// ## Returns
    ///
    /// - `Some(Goal::Debug { message })` -> Pause evaluation.
    /// - `None` -> Continue evaluation.
    fn maybe_break(&self, event: DebugEvent, vm: &PolarVirtualMachine) -> Option<Rc<Goal>> {
        if let Some(step) = self.step.as_ref() {
            //eprintln!("maybe break {:?}, {:?}", event, step);
            match (step, event) {
                (Step::Goal, DebugEvent::Goal(goal)) => Some(Rc::new(Goal::Debug {
                    message: goal.to_string(),
                })),
                (Step::InTo, DebugEvent::Query) => self.break_query(vm),
                (Step::Out { level }, DebugEvent::Query)
                    if vm.trace_stack.is_empty() || vm.trace_stack.len() < *level =>
                {
                    self.break_query(vm)
                }
                (Step::Over { level }, DebugEvent::Query) if vm.trace_stack.len() == *level => {
                    self.break_query(vm)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Produce the `Goal::Debug` for breaking on a Query (as opposed to breaking on a Goal).
    /// This is used to implement the `step`, `over`, and `out` debug commands
    fn break_query(&self, vm: &PolarVirtualMachine) -> Option<Rc<Goal>> {
        let message = vm.trace.last().and_then(|trace| {
            if let Trace {
                node: Node::Term(q),
                ..
            } = &**trace
            {
                match q.value() {
                    Value::Expression(Operation {
                        operator: Operator::And,
                        args,
                    }) if args.len() == 1 => return None,
                    _ => {
                        let source = self.query_source(&q, &vm.kb.read().unwrap().sources, 3);
                        Some(format!("{}\n\n{}", vm.query_summary(q), source))
                    }
                }
            } else {
                None
            }
        });
        message.map(|message| Rc::new(Goal::Debug { message }))
    }

    /// Process debugging commands from the user.
    ///
    /// For informational commands (`"bindings"`, `"goals"`, `"line"`, `"queries"`, and `"var"`),
    /// look up relevant data via the passed-in
    /// [`PolarVirtualMachine`](../vm/struct.PolarVirtualMachine.html), format it, and return a
    /// [`Goal::Debug`](../vm/enum.Goal.html) containing the formatted string that will be
    /// displayed to the user.
    ///
    /// For movement commands (`"continue"`, `"over"`, `"out"`, `"step"`), set the internal state
    /// of the [`Debugger`](struct.Debugger.html) to the appropriate
    /// [`Option<Step>`](struct.Debugger.html#structfield.step).
    fn debug_command(&mut self, command: &str, vm: &PolarVirtualMachine) -> Option<Goal> {
        fn show<T>(stack: &[T]) -> Goal
        where
            T: std::fmt::Display,
        {
            Goal::Debug {
                message: stack
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n"),
            }
        }
        let parts: Vec<&str> = command.split_whitespace().collect();
        match *parts.get(0).unwrap_or(&"help") {
            "c" | "continue" | "q" | "quit" => self.step = None,

            "n" | "next" | "over" => {
                self.step = Some(Step::Over{ level: vm.trace_stack.len() })
            }
            "s" | "step" | "into" => {
                self.step = Some(Step::InTo)
            }
            "o" | "out" => {
                self.step = Some(Step::Out{ level: vm.trace_stack.len() })
            }
            "g" | "goal" => {
                self.step = Some(Step::Goal)
            }
            "l" | "line" => {
                let lines = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                return Some(Goal::Debug {
                    message: vm.queries.last().map_or_else(
                        || "".to_string(),
                        |query| self.query_source(&query, &vm.kb.read().unwrap().sources, lines),
                    ),
                });
            }
            "query" => {
                if let Some(query) = vm.trace.last().and_then(|t| t.term()) {
                    return Some(Goal::Debug {
                        message: vm.query_summary(&query)});
                }
            }
            // "stack" | "trace" => {
            // }
            "goals" => return Some(show(&vm.goals)),
            "bindings" => {
                return Some(show(&vm.bindings))
            }
            "var" => {
                if parts.len() > 1 {
                    let vars: Vec<Binding> = parts[1..]
                        .iter()
                        .map(|var| {
                            let var = Symbol::new(var);
                            let value = vm.bindings(true).get(&var).cloned().unwrap_or_else(|| {
                                Term::new_temporary(Value::Variable(Symbol::new("<unbound>")))
                            });
                            Binding(var, value)
                        })
                        .collect();
                    return Some(show(&vars));
                } else {
                    let mut vars = vm
                        .bindings(true)
                        .keys()
                        .map(|k| k.to_polar())
                        .collect::<Vec<_>>()
                        .join(", ");
                    if vars.is_empty() {
                        vars = "No variables in scope.".to_string();
                    }
                    return Some(Goal::Debug { message: vars });
                }
            }
            _ => {
                return Some(Goal::Debug {
                    message: "Debugger Commands
  h[elp]                  Print this help documentation.
  c[ontinue]              Continue evaluation.
  n[ext]                  Step to the next query at the same level of the stack (step over in vscode)
  s[tep]                  Step to the next query                                (step into in vscode)
  o[ut]                   Step out of the current level to the one above        (step out in vscode)
  g[oal]                  Step to the next goal
  l[ine] [<n>]            Print the current line and <n> lines of context.
  query                   Print the current query
  stack                   Print the query stack
  goals                   Print current goal stack.
  bindings                Print all bindings
  var [<name> ...]        Print available variables. If one or more arguments
                          are provided, print the value of those variables.
  q[uit]                  Alias for 'continue'."
                        .to_string(),
                })
            }
        }
        None
    }
}
