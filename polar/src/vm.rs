use super::types::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Instruction {
    Backtrack,
    Bind(Symbol, Term),
    Choice(Vec<InstructionStream>, usize),
    Cut,
    External(Symbol), // POC
    Halt,
    Isa(Term, Term),
    Lookup(Instance, Term),
    LookupExternal(Instance, Term),
    Query(Predicate),
    Result(i64),
    Unify(Term, Term),
}

type InstructionStream = Vec<Instruction>;

#[derive(Debug)]
struct Binding(Symbol, Term);

pub struct PolarVirtualMachine {
    instructions: InstructionStream,
    bindings: Vec<Binding>,
    result: i64,
    kb: KnowledgeBase,
}

impl PolarVirtualMachine {
    pub fn new(kb: KnowledgeBase, instructions: InstructionStream) -> Self {
        Self {
            instructions,
            bindings: vec![],
            result: 0,
            kb,
        }
    }

    pub fn run(&mut self) -> QueryEvent {
        while let Some(instruction) = self.instructions.pop() {
            match instruction {
                Instruction::Backtrack => self.backtrack(),
                Instruction::Bind(var, value) => self.bind(&var, &value),
                Instruction::Choice(choices, _) => self.choice(choices),
                Instruction::Cut => self.cut(),
                Instruction::External(name) => return QueryEvent::External(name), // POC
                Instruction::Halt => self.halt(),
                Instruction::Isa(_, _) => unimplemented!("isa"),
                Instruction::Lookup(_, _) => unimplemented!("lookup"),
                Instruction::LookupExternal(_, _) => unimplemented!("lookup external"),
                Instruction::Query(predicate) => {
                    return QueryEvent::Result{ bindings: self.query(&predicate) };
                }
                Instruction::Result(result) => self.result(result),
                Instruction::Unify(left, right) => self.unify(&left, &right),
            }
        }
        QueryEvent::Done
    }

    fn backtrack(&mut self) {
        while let Some(instruction) = self.instructions.pop() {
            match instruction {
                Instruction::Choice(choices, index) => {
                    self.bindings.drain(index..);
                    if choices.len() == 0 {
                        break;
                    }
                }
                _ => (),
            }
        }
    }

    fn query(&mut self, predicate: &Predicate) -> Bindings {
        // Select applicable rules for predicate.
        // Sort applicable rules by specificity.
        // Create a choice over the applicable rules.


        if let Some(generic_rule) = self.kb.rules.get(&predicate.name) {
            let generic_rule = generic_rule.clone();
            assert_eq!(generic_rule.name, predicate.name);
            let rule = &generic_rule.rules[0]; // just panic
            let var = &rule.params[0];
            let val = &predicate.args[0];

            self.unify(var,val); // @TODO: make instruction.
        }
        
        let mut bindings = HashMap::new();
        for binding in &self.bindings {
            bindings.insert(binding.0.clone(), binding.1.clone());
        }
        bindings
    }

    fn cut(&mut self) {
        for instruction in self.instructions.iter_mut().rev() {
            match instruction {
                Instruction::Choice(ref mut choices, _) => {
                    choices.clear();
                    break;
                }
                _ => (),
            }
        }
    }

    fn bind(&mut self, var: &Symbol, value: &Term) {
        self.bindings.push(Binding(var.clone(), value.clone()));
    }

    fn choice(&mut self, mut choices: Vec<InstructionStream>) {
        if let Some(mut choice) = choices.pop() {
            let index = self.bindings.len();
            self.instructions.push(Instruction::Choice(choices, index));
            self.instructions.append(&mut choice);
        }
    }

    pub fn halt(&mut self) {
        self.instructions.clear();
    }

    pub fn result(&mut self, result: i64) {
        self.result = result;
        self.bind(&Symbol("a".to_string()), &Term::new(Value::Integer(result)));
    }

    fn unify(&mut self, left: &Term, right: &Term) {
        match (&left.value, &right.value) {
            (Value::Symbol(_), _) => self.unify_var(left, right),
            (_, Value::Symbol(_)) => self.unify_var(right, left),
            (Value::List(left), Value::List(right)) => {
                if left.len() != right.len() {
                    self.backtrack();
                }

                for (left, right) in left.iter().zip(right) {
                    self.unify(&left, &right);
                }
            }
            (Value::Integer(left), Value::Integer(right)) => {
                if left != right {
                    self.backtrack();
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

        self.bind(left_sym, right);
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
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::new(), vec![Instruction::Bind(x.clone(), zero.clone())]);
        vm.run();
        assert_eq!(vm.value(&x), Some(&zero));
        assert_eq!(vm.value(&y), None);
    }

    #[test]
    fn halt() {
        let mut vm = PolarVirtualMachine::new(KnowledgeBase::new(), vec![Instruction::Halt]);
        vm.run();
        assert_eq!(vm.instructions.len(), 0);
        assert_eq!(vm.bindings.len(), 0);
    }

    #[test]
    fn unify() {
        let x = Symbol("x".to_string());
        let y = Symbol("y".to_string());
        let zero = Term::new(Value::Integer(0));
        let one = Term::new(Value::Integer(1));
        let mut vm = PolarVirtualMachine::new(
            KnowledgeBase::new(),
            vec![
                Instruction::Unify(Term::new(Value::Symbol(x.clone())), zero.clone()),
                Instruction::Choice(
                    vec![
                        vec![Instruction::Unify(zero.clone(), one.clone())],
                        vec![Instruction::Unify(
                            Term::new(Value::Symbol(y.clone())),
                            one.clone(),
                        )],
                    ],
                    0,
                ),
            ],
        );
        vm.run();
        assert_eq!(vm.value(&x), Some(&zero));
        assert_eq!(vm.value(&y), Some(&one));
    }
}
