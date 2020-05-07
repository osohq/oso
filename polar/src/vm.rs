use super::types::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Goal {
    Backtrack,
    Bind { variable: Symbol, value: Term },
    Bindings,
    Choice { choices: Vec<Goals>, bsp: usize }, // binding stack pointer
    Cut,
    External { name: Symbol }, // POC
    Halt,
    Isa { left: Term, right: Term },
    Lookup { instance: Instance, field: Term },
    LookupExternal { instance: Instance, field: Term },
    Query { predicate: Predicate },
    Result { name: Symbol, value: i64 },
    Unify { left: Term, right: Term },
}

type Goals = Vec<Goal>;

#[derive(Debug)]
struct Binding(Symbol, Term);

pub struct PolarVirtualMachine {
    goals: Goals,
    bindings: Vec<Binding>,
    kb: KnowledgeBase,
}

impl PolarVirtualMachine {
    pub fn new(kb: KnowledgeBase, goals: Goals) -> Self {
        Self {
            goals,
            bindings: vec![],
            kb,
        }
    }

    pub fn run(&mut self) -> QueryEvent {
        while let Some(goal) = self.goals.pop() {
            //println!("{:?}", goal);
            match goal {
                Goal::Backtrack => self.backtrack(),
                Goal::Bind { variable, value } => self.bind(&variable, &value),
                Goal::Bindings => {
                    return QueryEvent::Result {
                        bindings: self.bindings(),
                    };
                }
                Goal::Choice { choices, bsp } => self.choice(choices, bsp),
                Goal::Cut => self.cut(),
                Goal::External { name } => return QueryEvent::External { name }, // POC
                Goal::Halt => self.halt(),
                Goal::Isa { .. } => unimplemented!("isa"),
                Goal::Lookup { .. } => unimplemented!("lookup"),
                Goal::LookupExternal { .. } => unimplemented!("lookup external"),
                Goal::Query { predicate } => self.query(&predicate),
                Goal::Result { name, value } => self.result(&name, value),
                Goal::Unify { left, right } => self.unify(&left, &right),
            }
        }
        QueryEvent::Done
    }

    fn backtrack(&mut self) {
        while let Some(goal) = self.goals.pop() {
            match goal {
                Goal::Choice { choices, bsp } => {
                    self.bindings.drain(bsp..);
                    if choices.len() == 0 {
                        break;
                    }
                }
                _ => (),
            }
        }
    }

    fn push_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
    }

    fn append_goals(&mut self, mut goals: Goals) {
        goals.reverse();
        self.goals.append(&mut goals);
    }

    fn query(&mut self, predicate: &Predicate) {
        // Select applicable rules for predicate.
        // Sort applicable rules by specificity.
        // Create a choice over the applicable rules.

        if let Some(generic_rule) = self.kb.rules.get(&predicate.name) {
            let generic_rule = generic_rule.clone();
            assert_eq!(generic_rule.name, predicate.name);

            let mut choices = vec![];
            for rule in &generic_rule.rules {
                let var = &rule.params[0];
                let val = &predicate.args[0];
                choices.push(vec![
                    Goal::Unify {
                        left: var.clone(),
                        right: val.clone(),
                    },
                    Goal::Bindings,
                ]);
            }

            self.push_goal(Goal::Choice {
                choices,
                bsp: self.bindings.len(),
            });
        } else {
            self.push_goal(Goal::Backtrack)
        }
    }

    fn bindings(&mut self) -> Bindings {
        let mut bindings = HashMap::new();
        for binding in &self.bindings {
            bindings.insert(binding.0.clone(), binding.1.clone());
        }
        bindings
    }

    fn cut(&mut self) {
        for goal in self.goals.iter_mut().rev() {
            match goal {
                Goal::Choice {
                    ref mut choices,
                    bsp: _,
                } => {
                    choices.clear();
                    break;
                }
                _ => (),
            }
        }
    }

    fn bind(&mut self, var: &Symbol, value: &Term) {
        //println!("{:?} ← {:?}", var, value);
        self.bindings.push(Binding(var.clone(), value.clone()));
    }

    // sp [[a,b,c][d,e,f]]
    // [[d,e,f] a, b, c]
    // [[] a, b, c, d, e, f]
    //
    // [a,b.c] [other choices later]

    fn choice(&mut self, mut choices: Vec<Goals>, bsp: usize) {
        if choices.len() > 0 {
            let choice = choices.remove(0);
            self.push_goal(Goal::Choice { choices, bsp });
            self.append_goals(choice);
        }
    }

    pub fn halt(&mut self) {
        self.goals.clear();
    }

    pub fn result(&mut self, name: &Symbol, value: i64) {
        self.push_goal(Goal::Bind {
            variable: name.clone(),
            value: Term::new(Value::Integer(value)),
        });
    }

    fn unify(&mut self, left: &Term, right: &Term) {
        match (&left.value, &right.value) {
            (Value::Symbol(_), _) => self.unify_var(left, right),
            (_, Value::Symbol(_)) => self.unify_var(right, left),
            (Value::List(left), Value::List(right)) => {
                if left.len() == right.len() {
                    self.append_goals(
                        left.iter()
                            .zip(right)
                            .map(|(left, right)| Goal::Unify {
                                left: left.clone(),
                                right: right.clone(),
                            })
                            .collect(),
                    )
                } else {
                    self.push_goal(Goal::Backtrack);
                }
            }
            (Value::Integer(left), Value::Integer(right)) => {
                if left != right {
                    self.push_goal(Goal::Backtrack);
                }
            }
            (_, _) => unimplemented!("unhandled unification {:?} = {:?}", left, right),
        }
    }

    fn unify_var(&mut self, left: &Term, right: &Term) {
        let left_sym = if let Value::Symbol(left_sym) = &left.value {
            left_sym
        } else {
            panic!("unify_var must be called with left as a Symbol");
        };

        // if let Some(left_value) = self.value(&left_sym) {
        //     return self.unify(&left_value, right);
        // }

        // if let Value::Symbol(right_sym) = &right.value {
        //     if let Some(right_value) = self.value(&right_sym) {
        //         return self.unify(left, &right_value);
        //     }
        // }

        self.push_goal(Goal::Bind {
            variable: left_sym.clone(),
            value: right.clone(),
        });
    }

    fn value(&self, variable: &Symbol) -> Option<&Term> {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.0 == *variable)
            .map(|binding| &binding.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind() {
        let x = Symbol("x".to_string());
        let y = Symbol("y".to_string());
        let zero = Term::new(Value::Integer(0));
        let mut vm = PolarVirtualMachine::new(
            KnowledgeBase::new(),
            vec![Goal::Bind {
                variable: x.clone(),
                value: zero.clone(),
            }],
        );
        vm.run();
        assert_eq!(vm.value(&x), Some(&zero));
        assert_eq!(vm.value(&y), None);
    }

    #[test]
    fn halt() {
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::new(), vec![Goal::Halt]);
        vm.run();
        assert_eq!(vm.goals.len(), 0);
        assert_eq!(vm.bindings.len(), 0);
    }

    #[test]
    fn unify() {
        let x = Symbol("x".to_string());
        let y = Symbol("y".to_string());
        let vars = Term::new(Value::List(vec![
            Term::new(Value::Symbol(x.clone())),
            Term::new(Value::Symbol(y.clone())),
        ]));
        let zero = Term::new(Value::Integer(0));
        let one = Term::new(Value::Integer(1));
        let vals = Term::new(Value::List(vec![zero.clone(), one.clone()]));
        let mut vm = PolarVirtualMachine::new(
            KnowledgeBase::new(),
            vec![Goal::Unify {
                left: vars.clone(),
                right: vals.clone(),
            }],
        );
        vm.run();
        assert_eq!(vm.value(&x), Some(&zero));
        assert_eq!(vm.value(&y), Some(&one));
    }
}
