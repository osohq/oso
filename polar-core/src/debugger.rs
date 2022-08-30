use std::rc::Rc;

use super::bindings::Binding;
use super::error::{PolarError, PolarResult};
use super::formatting::source_lines;
use super::kb::KnowledgeBase;
use super::partial::simplify_bindings;
use super::terms::*;
use super::traces::*;
use super::vm::*;

impl PolarVirtualMachine {
    pub fn query_summary(&self, query: &Term) -> String {
        let relevant_bindings = self.relevant_bindings(&[query]);
        let bindings_str = relevant_bindings
            .iter()
            .map(|(var, val)| format!("{} = {}", var.0, val))
            .collect::<Vec<_>>()
            .join(", ");
        format!("QUERY: {}, BINDINGS: {{{}}}", query, bindings_str)
    }

    /// If the inner [`Debugger`](struct.Debugger.html) returns a [`Goal`](../vm/enum.Goal.html),
    /// push it onto the goal stack.
    pub fn maybe_break(&mut self, event: DebugEvent) -> PolarResult<bool> {
        self.debugger.maybe_break(event, self).map_or_else(
            || Ok(false),
            |goal| {
                self.push_goal(goal)?;
                Ok(true)
            },
        )
    }
}

/// [`Debugger`](struct.Debugger.html) step granularity.
#[derive(Clone, Debug)]
enum Step {
    /// Pause after evaluating the next [`Goal`](../vm/enum.Goal.html).
    Goal,
    /// Step **over** the current query. Will break on the next query where the trace stack is at the same
    /// level as the current one.
    Over {
        level: usize,
    },
    /// Step **out** of the current query. Will break on the next query where the trace stack is at a lower
    /// level than the current one.
    Out {
        level: usize,
    },
    /// Step **in**. Will break on the next query.
    Into,
    Error,
    Rule,
}

/// VM breakpoints.
///
/// There are currently two breakpoints in the VM, one that fires after every
/// [`Goal`](../vm/enum.Goal.html) and another that fires before every
/// [`Goal::Query`](../vm/enum.Goal.html). When either breakpoint is hit, we check the
/// [`Debugger`](struct.Debugger.html)'s internal [`step`](struct.Debugger.html#structfield.step)
/// field to determine how evaluation should proceed.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum DebugEvent {
    Goal(Rc<Goal>),
    Query,
    Pop,
    Error(PolarError),
    Rule,
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
    last: Option<String>,
}

impl Debugger {
    /// Retrieve the original source line (and, optionally, additional lines of context) for the
    /// current query.
    fn query_source(&self, query: &Term, num_lines: usize) -> String {
        query.parsed_context().map_or_else(
            || "".to_string(),
            |context| source_lines(&context.source, context.left, num_lines),
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
    fn maybe_break(&self, event: DebugEvent, vm: &PolarVirtualMachine) -> Option<Goal> {
        self.step.as_ref().and_then(|step| match (step, event) {
            (Step::Goal, DebugEvent::Goal(goal)) => Some(Goal::Debug {
                message: goal.to_string(),
            }),
            (Step::Into, DebugEvent::Query) => self.break_query(vm),
            (Step::Out { level }, DebugEvent::Query)
                if vm.trace_stack.is_empty() || vm.trace_stack.len() < *level =>
            {
                self.break_query(vm)
            }
            (Step::Over { level }, DebugEvent::Query) if vm.trace_stack.len() == *level => {
                self.break_query(vm)
            }
            (Step::Error, DebugEvent::Error(error)) => {
                let context = error
                    .get_context()
                    .map_or_else(|| "".into(), |c| c.source_position());
                self.break_msg(vm).map(|message| Goal::Debug {
                    message: format!("{}\nERROR: {}{}\n", message, error.0, context),
                })
            }
            (Step::Rule, DebugEvent::Rule) => self.break_query(vm),
            _ => None,
        })
    }

    pub fn break_msg(&self, vm: &PolarVirtualMachine) -> Option<String> {
        vm.trace.last().and_then(|trace| match trace.node {
            Node::Term(ref q) => match q.value() {
                Value::Expression(Operation {
                    operator: Operator::And,
                    args,
                }) if args.len() == 1 => None,
                _ => {
                    let source = self.query_source(q, 3);
                    Some(format!("{}\n\n{}\n", vm.query_summary(q), source))
                }
            },
            Node::Rule(ref r) => Some(r.to_string()),
        })
    }

    /// Produce the `Goal::Debug` for breaking on a Query (as opposed to breaking on a Goal).
    /// This is used to implement the `step`, `over`, and `out` debug commands.
    fn break_query(&self, vm: &PolarVirtualMachine) -> Option<Goal> {
        self.break_msg(vm).map(|message| Goal::Debug { message })
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
    pub fn debug_command(&mut self, command: &str, vm: &PolarVirtualMachine) -> Option<Goal> {
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
        let default_command = match self.last.take() {
            Some(s) => s,
            _ => "help".to_owned(),
        };
        let command = *parts.first().unwrap_or(&&default_command[..]);
        self.last = Some(String::from(command));
        match command {
            "c" | "continue" | "q" | "quit" => self.step = None,

            "n" | "next" | "over" => {
                self.step = Some(Step::Over{ level: vm.trace_stack.len() })
            }
            "s" | "step" | "into" => {
                self.step = Some(Step::Into)
            }
            "o" | "out" => {
                self.step = Some(Step::Out{ level: vm.trace_stack.len() })
            }
            "g" | "goal" => {
                self.step = Some(Step::Goal)
            }
            "e" | "error" => {
                self.step = Some(Step::Error)
            }
            "r" | "rule" => {
                self.step = Some(Step::Rule)
            }
            "l" | "line" => {
                let lines = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                return Some(Goal::Debug {
                    message: vm.queries.last().map_or_else(
                        || "".to_string(),
                        |query| self.query_source(query, lines),
                    ),
                });
            }
            "query" => {
                let mut level = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let mut trace_stack = vm.trace_stack.clone();

                // Walk up the trace stack to get the query at the requested level.
                let mut term = vm.trace.last().and_then(|t| t.term());
                while level > 0 {
                    if let Some(trace) = trace_stack.pop().map(|ts| ts.as_ref().clone()) {
                        if let Some(t) = trace.last() {
                            if let Trace{node: Node::Term(t), ..} = &**t {
                                term = Some(t.clone());
                                level-=1;
                            }
                        }
                    } else {
                        return Some(Goal::Debug {
                            message: "Error: level is out of range".to_owned()
                        })
                    }
                }

                if let Some(query) = term {
                    return Some(Goal::Debug {
                        message: vm.query_summary(&query)});
                } else {
                    return Some(Goal::Debug {
                        message: "".to_owned()
                    })
                }
            }
            "stack" | "trace" => {
                return Some(Goal::Debug {
                    message: vm.stack_trace()
                })
            }
            "goals" => return Some(show(&vm.goals)),
            "bindings" => {
                return Some(show(vm.bindings_debug().as_slice()))
            }
            "var" => {
                if parts.len() > 1 {
                    let vars: Vec<Binding> = parts[1..]
                        .iter()
                        .map(|name| {
                            get_binding_for_var(name, vm)
                        })
                        .collect();
                    return Some(show(&vars));
                } else {
                    let mut vars = vm
                        .bindings(true)
                        .keys()
                        .map(|k| k.0.as_ref())
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
  s[tep] | into           Step to the next query (will step into rules).
  n[ext] | over           Step to the next query at the same level of the query stack (will not step into rules).
  o[ut]                   Step out of the current query stack level to the next query in the level above.
  g[oal]                  Step to the next goal of the Polar VM.
  e[rror]                 Step to the next error.
  r[ule]                  Step to the next rule.
  l[ine] [<n>]            Print the current line and <n> lines of context.
  query [<i>]             Print the current query or the query at level <i> in the query stack.
  stack | trace           Print the current query stack.
  goals                   Print the current goal stack.
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

/// *** variable name mapping ***
/// if the requested variable is bound, then we return that binding.
/// otherwise, we look for the matching bound temp variable with the
/// highest numeric component in its name, and return that binding
/// if we find it. otherwise, show that the variable is unbound.
pub fn get_binding_for_var(name: &str, vm: &PolarVirtualMachine) -> Binding {
    let var = Symbol::new(name);
    let bindings = simplify_bindings(vm.bindings(true)).unwrap();
    bindings.get(&var).cloned().map_or_else(
        || {
            let prefix = KnowledgeBase::temp_prefix(name);
            bindings
                .keys()
                .filter_map(|k| {
                    k.0.strip_prefix(&prefix)
                        .and_then(|i| i.parse::<i64>().map_or(None, |i| Some((k, i))))
                })
                .max_by(|a, b| a.1.cmp(&b.1))
                .map_or_else(
                    || Binding(sym!(name), Term::from(sym!("<unbound>"))),
                    |b| {
                        Binding(
                            sym!(format!("{}@{}", name, b.0 .0).as_str()),
                            bindings.get(b.0).unwrap().clone(),
                        )
                    },
                )
        },
        |val| Binding(var, val),
    )
}
