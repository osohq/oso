use super::types::*;
use super::vm::*;

fn source_lines(source: &Source, offset: usize, source_context_lines: usize) -> String {
    // Sliding window of lines: current line + indicator + additional context above + below.
    let max_lines = source_context_lines * 2 + 2;
    let push_line = |lines: &mut Vec<String>, line: String| {
        if lines.len() == max_lines {
            lines.remove(0);
        }
        lines.push(line);
    };

    match source {
        Source::Load { src, .. } | Source::Query { src } => {
            let mut index = 0;
            let mut lines = Vec::new();
            let mut target = None;
            let prefix_len = "123: ".len();
            for (lineno, line) in src.lines().enumerate() {
                push_line(&mut lines, format!("{:03}: {}", lineno + 1, line));
                let end = index + line.len() + 1; // Adding one to account for new line byte.
                if target.is_none() && end >= offset {
                    target = Some(lineno);
                    let spaces = " ".repeat(offset - index + prefix_len);
                    push_line(&mut lines, format!("{}^", spaces));
                }
                index = end;
                if target.is_some() && lineno == target.unwrap() + source_context_lines {
                    break;
                }
            }
            lines.join("\n")
        }
    }
}

fn query_source(kb: &KnowledgeBase, term: Option<&Term>, source_context_lines: usize) -> String {
    term.map_or("".to_string(), |term| {
        source_lines(&kb.get_source(term), term.offset, source_context_lines)
    })
}

impl PolarVirtualMachine {
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
                        message: query_source(&self.kb, self.queries.last(), 0),
                    })
                    .map_or((), |_| ());
                }
            }
            (Breakpoint::Out { queries }, Breakpoint::Over { .. }) => {
                if queries[..queries.len() - 3] == self.queries[..self.queries.len() - 1] {
                    self.push_goal(Goal::Debug {
                        message: query_source(&self.kb, self.queries.last(), 0),
                    })
                    .map_or((), |_| ());
                }
            }
            _ => (),
        }
    }

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
                    message: query_source(&self.kb, self.queries.last(), lines),
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
                    let vars: Bindings = parts[1..]
                        .iter()
                        .map(|var| {
                            let var = Symbol::new(var);
                            Binding(var.clone(), self.deref(&Term::new(Value::Symbol(var))))
                        })
                        .collect();
                    self.push_goal(show(&vars)).map_or((), |_| ());
                } else {
                    self.push_goal(Goal::Debug {
                        message: "Please specify one or more vars.".to_string(),
                    })
                    .map_or((), |_| ());
                }
            }
            _ => self
                .push_goal(Goal::Debug {
                    message: "Debugger Commands
  bindings                      Print current binding stack.
  c[ontinue] | q[uit]           Continue evaluation.
  goals                         Print current goal stack.
  h[elp]                        Print this help documentation.
  l[ine] [<n>]                  Print out the current line and <n> lines of context.
  n[ext] | over                 Stop at the next query.
  out                           Stop at the next rule.
  stack | queries               Print current query stack.
  s[tep]                        Evaluate one goal.
  var <name> [<name> ...]       Print the value of one or more variables."
                        .to_string(),
                })
                .map_or((), |_| ()),
        }
    }
}
