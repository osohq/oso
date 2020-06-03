use super::formatting::ToPolarString;
use super::types::*;
use super::vm::*;

impl PolarVirtualMachine {
    pub fn maybe_break(&mut self, context: Breakpoint) {
        fn query_to_polar(query: Option<&Term>) -> String {
            query.map_or("".to_string(), |term| term.to_polar())
        }

        match (&self.breakpoint, context) {
            (Breakpoint::Step { .. }, Breakpoint::Step { goal }) => self
                .push_goal(Goal::Debug {
                    message: goal.to_string(),
                })
                .map_or((), |_| ()),
            (Breakpoint::Over { queries }, Breakpoint::Over { .. }) if queries == &self.queries => {
                self.push_goal(Goal::Debug {
                    message: query_to_polar(self.queries.last()),
                })
                .map_or((), |_| ());
            }
            (Breakpoint::Out { queries }, Breakpoint::Over { .. })
                if queries[..queries.len() - 1] == self.queries[..] =>
            {
                self.push_goal(Goal::Debug {
                    message: query_to_polar(self.queries.last()),
                })
                .map_or((), |_| ());
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
                    .collect::<Vec<String>>()
                    .join("\n"),
            }
        }
        // TODO: handle any amount of whitespace
        let parts: Vec<&str> = command.split(' ').collect();
        match parts[0] {
            "bindings" => self.push_goal(show(&self.bindings)).map_or((), |_| ()),
            "c" | "continue" => self.breakpoint = Breakpoint::None,
            "goals" => self.push_goal(show(&self.goals)).map_or((), |_| ()),
            // Execute one instruction.
            "s" | "step" => self.breakpoint = Breakpoint::Step { goal: Goal::Noop },
            // Step over the current clause in a rule body.
            "n" | "next" | "over" => {
                self.breakpoint = Breakpoint::Over {
                    queries: self.queries.clone(),
                }
            }
            // Execute through the end of the current rule.
            "out" => {
                self.breakpoint = Breakpoint::Out {
                    queries: self.queries.clone(),
                }
            }
            "stack" | "queries" => self.push_goal(show(&self.queries)).map_or((), |_| ()),
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
                        message: "Please specify a var.".to_string(),
                    })
                    .map_or((), |_| ());
                }
            }
            _ => self
                .push_goal(Goal::Debug {
                    message: "Debugger Commands
  h[elp]                        Print this help documentation.
  bindings                      Print current binding stack.
  goals                         Print current goal stack.
  c[ontinue]                    Continue evaluation.
  s[tep]                        Evaluate one goal.
  out                           Stop at the next rule.
  n[ext] | over                 Stop at the next query.
  var <name> [<name> ...]       Print the value of one or more variables.
  stack | queries               Print current query stack."
                        .to_string(),
                })
                .map_or((), |_| ()),
        }
    }
}
