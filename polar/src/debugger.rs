use std::rc::Rc;

use super::types::*;
use super::vm::*;
use super::{PolarResult, ToPolarString};

impl PolarVirtualMachine {
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

/// Traverse a [`Source`](../types/struct.Source.html) line-by-line until `offset` is reached, and
/// then return the source line containing the `offset` character as well as `num_lines` lines
/// above and below it.
fn source_lines(source: &Source, offset: usize, num_lines: usize) -> String {
    // Sliding window of lines: current line + indicator + additional context above + below.
    let max_lines = num_lines * 2 + 2;
    let push_line = |lines: &mut Vec<String>, line: String| {
        if lines.len() == max_lines {
            lines.remove(0);
        }
        lines.push(line);
    };
    let mut index = 0;
    let mut lines = Vec::new();
    let mut target = None;
    let prefix_len = "123: ".len();
    for (lineno, line) in source.src.lines().enumerate() {
        push_line(&mut lines, format!("{:03}: {}", lineno + 1, line));
        let end = index + line.len() + 1; // Adding one to account for new line byte.
        if target.is_none() && end >= offset {
            target = Some(lineno);
            let spaces = " ".repeat(offset - index + prefix_len);
            push_line(&mut lines, format!("{}^", spaces));
        }
        index = end;
        if target.is_some() && lineno == target.unwrap() + num_lines {
            break;
        }
    }
    lines.join("\n")
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
    /// a() := debug(), b();
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
    Over {
        /// Snapshot of the current query stack sans the current query.
        snapshot: Queries,
    },
    /// Step **out** of the current parent query, evaluating goals until reaching the
    /// [`Goal::Query`](../vm/enum.Goal.html) for its next sibling.
    ///
    /// To illustrate this movement, let's step through the queries for the following snippet of
    /// polar:
    ///
    /// ```text
    /// a() := b(), c();
    /// b() := debug(), d();
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
    ///   stored slice.
    Out {
        /// Snapshot of the current query stack sans its last three queries.
        snapshot: Queries,
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
        sources.get_source(query).map_or_else(
            || "".to_string(),
            |source| source_lines(&source, query.offset, num_lines),
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
        self.step.as_ref().and_then(|step| match (step, event) {
            (Step::Goal, DebugEvent::Goal(goal)) => Some(Rc::new(Goal::Debug {
                message: goal.to_string(),
            })),
            (Step::Over { snapshot }, DebugEvent::Query)
            | (Step::Out { snapshot }, DebugEvent::Query)
                if vm.queries[..vm.queries.len() - 1] == snapshot[..] =>
            {
                Some(Rc::new(Goal::Debug {
                    message: vm.queries.last().map_or_else(
                        || "".to_string(),
                        |query| self.query_source(&query, &vm.kb.read().unwrap().sources, 0),
                    ),
                }))
            }
            _ => None,
        })
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
            "bindings" => return Some(show(&vm.bindings)),
            "c" | "continue" | "q" | "quit" => self.step = None,
            "goals" => return Some(show(&vm.goals)),
            "l" | "line" => {
                let lines = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                return Some(Goal::Debug {
                    message: vm.queries.last().map_or_else(
                        || "".to_string(),
                        |query| self.query_source(&query, &vm.kb.read().unwrap().sources, lines),
                    ),
                });
            }
            "n" | "next" | "over" => {
                self.step = Some(Step::Over {
                    snapshot: vm.queries[..vm.queries.len() - 1].to_vec(),
                })
            }
            "out" => {
                self.step = Some(Step::Out {
                    snapshot: vm.queries[..vm.queries.len() - 3].to_vec(),
                })
            }
            "stack" | "queries" => return Some(show(&vm.queries)),
            "s" | "step" => self.step = Some(Step::Goal),
            "var" => {
                if parts.len() > 1 {
                    let vars: Vec<Binding> = parts[1..]
                        .iter()
                        .map(|var| {
                            let var = Symbol::new(var);
                            let value = vm.bindings(true).get(&var).cloned().unwrap_or_else(|| {
                                Rc::new(Term::new(Value::Symbol(Symbol::new("<unbound>"))))
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
  bindings                Print current binding stack.
  c[ontinue]              Continue evaluation.
  goals                   Print current goal stack.
  h[elp]                  Print this help documentation.
  l[ine] [<n>]            Print the current line and <n> lines of context.
  n[ext]                  Alias for 'over'.
  out                     Evaluate goals through the end of the current parent
                          query and stop at its next sibling (if one exists).
  over                    Evaluate goals until reaching the next sibling of the
                          current query (if one exists).
  queries                 Print current query stack.
  q[uit]                  Alias for 'continue'.
  stack                   Alias for 'queries'.
  s[tep]                  Evaluate one goal.
  var [<name> ...]        Print available variables. If one or more arguments
                          are provided, print the value of those variables."
                        .to_string(),
                })
            }
        }
        None
    }
}
