use super::types::*;
use super::vm::*;
use super::ToPolarString;

/// Traverse a `Source` line-by-line until `offset` is reached, and then return the source line
/// containing the `offset` character as well as `source_context_lines` lines above and below it.
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

impl PolarVirtualMachine {
    fn query_source(&self, num_lines: usize) -> String {
        match (self.queries.last(), self.source.as_ref()) {
            (Some(query), Some((source, term))) if query == term => {
                source_lines(&source, term.offset, num_lines)
            }
            (Some(query), _) => source_lines(&self.kb.get_source(&query), query.offset, num_lines),
            _ => "".to_string(),
        }
    }

    /// Potential debugger entrypoints.
    pub fn maybe_break(&mut self, context: Breakpoint) {
        match (&self.breakpoint, context) {
            (Breakpoint::Step { .. }, Breakpoint::Step { goal }) => self
                .push_goal(Goal::Debug {
                    message: goal.to_string(),
                })
                .map_or((), |_| ()),
            (Breakpoint::Over { queries }, Breakpoint::Over { .. }) => {
                let n = self.queries.len() - 1;
                if n < queries.len() && self.queries[..n] == queries[..n] {
                    self.push_goal(Goal::Debug {
                        message: self.query_source(0),
                    })
                    .map_or((), |_| ());
                }
            }
            (Breakpoint::Out { queries }, Breakpoint::Over { .. }) => {
                if queries[..queries.len() - 3] == self.queries[..self.queries.len() - 1] {
                    self.push_goal(Goal::Debug {
                        message: self.query_source(0),
                    })
                    .map_or((), |_| ());
                }
            }
            _ => (),
        }
    }

    /// Respond to debugging commands from the user.
    ///
    /// The help output in the catch-all arm is a reference for all the other arms.
    pub fn debug_command(&mut self, command: &str) {
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
            "bindings" => self.push_goal(show(&self.bindings)).map_or((), |_| ()),
            "c" | "continue" | "q" | "quit" => self.breakpoint = Breakpoint::None,
            "goals" => self.push_goal(show(&self.goals)).map_or((), |_| ()),
            "l" | "line" => {
                let lines = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                self.push_goal(Goal::Debug {
                    message: self.query_source(lines),
                })
                .map_or((), |_| ());
            }
            "n" | "next" | "over" => {
                self.breakpoint = Breakpoint::Over {
                    queries: self.queries.clone(),
                }
            }
            "out" => {
                self.breakpoint = Breakpoint::Out {
                    queries: self.queries.clone(),
                }
            }
            "stack" | "queries" => self.push_goal(show(&self.queries)).map_or((), |_| ()),
            "s" | "step" => self.breakpoint = Breakpoint::Step { goal: Goal::Noop },
            "var" => {
                if parts.len() > 1 {
                    let vars: Vec<Binding> = parts[1..]
                        .iter()
                        .map(|var| {
                            let var = Symbol::new(var);
                            let value =
                                self.bindings(true).get(&var).cloned().unwrap_or_else(|| {
                                    Term::new(Value::Symbol(Symbol::new("<unbound>")))
                                });
                            Binding(var, value)
                        })
                        .collect();
                    self.push_goal(show(&vars)).map_or((), |_| ());
                } else {
                    let mut vars = self
                        .bindings(true)
                        .keys()
                        .map(|k| k.to_polar())
                        .collect::<Vec<_>>()
                        .join(", ");
                    if vars.is_empty() {
                        vars = "No variables in scope.".to_string();
                    }
                    self.push_goal(Goal::Debug { message: vars })
                        .map_or((), |_| ());
                }
            }
            _ => self
                .push_goal(Goal::Debug {
                    message: "Debugger Commands
  bindings                Print current binding stack.
  c[ontinue]              Continue evaluation.
  goals                   Print current goal stack.
  h[elp]                  Print this help documentation.
  l[ine] [<n>]            Print the current line and <n> lines of context.
  n[ext]                  Alias for 'over'.
  out                     Stop at the next rule.
  over                    Stop at the next query.
  queries                 Print current query stack.
  q[uit]                  Alias for 'continue'.
  stack                   Alias for 'queries'.
  s[tep]                  Evaluate one goal.
  var [<name> ...]        Print available variables. If one or more arguments
                          are provided, print the value of those variables."
                        .to_string(),
                })
                .map_or((), |_| ()),
        }
    }
}
