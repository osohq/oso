use std::collections::HashSet;

use crate::folder::{fold_operation, fold_term, Folder};
use crate::terms::{Operation, Operator, Symbol, Term, Value};
use crate::visitor::{walk_operation, Visitor};
use crate::vm::compare;

/// A trivially true expression.
pub const TRUE: Operation = op!(And);
/// A trivially false expression.
pub const FALSE: Operation = op!(Or);

/// Invert operators.
pub fn invert_operation(Operation { operator, args }: Operation) -> Operation {
    fn invert_args(args: Vec<Term>) -> Vec<Term> {
        args.into_iter()
            .map(|t| {
                t.clone_with_value(value!(invert_operation(
                    t.value().as_expression().unwrap().clone()
                )))
            })
            .collect()
    }

    match operator {
        Operator::And => Operation {
            operator: Operator::Or,
            args: invert_args(args),
        },
        Operator::Or => Operation {
            operator: Operator::And,
            args: invert_args(args),
        },
        Operator::Unify | Operator::Eq => Operation {
            operator: Operator::Neq,
            args,
        },
        Operator::Neq => Operation {
            operator: Operator::Unify,
            args,
        },
        Operator::Gt => Operation {
            operator: Operator::Leq,
            args,
        },
        Operator::Geq => Operation {
            operator: Operator::Lt,
            args,
        },
        Operator::Lt => Operation {
            operator: Operator::Geq,
            args,
        },
        Operator::Leq => Operation {
            operator: Operator::Gt,
            args,
        },
        Operator::Debug | Operator::Print | Operator::New | Operator::Dot => {
            Operation { operator, args }
        }
        Operator::Isa => Operation {
            operator: Operator::Not,
            args: vec![term!(op!(Isa, args[0].clone(), args[1].clone()))],
        },
        Operator::Not => args[0]
            .value()
            .as_expression()
            .expect("negated expression")
            .clone(),
        _ => todo!("negate {:?}", operator),
    }
}

impl Operation {
    /// Construct & return a set of symbols that occur in this operation.
    pub fn variables(&self) -> Vec<Symbol> {
        struct VariableVisitor {
            seen: HashSet<Symbol>,
            vars: Vec<Symbol>,
        }

        impl Visitor for VariableVisitor {
            fn visit_variable(&mut self, v: &Symbol) {
                if self.seen.insert(v.clone()) {
                    self.vars.push(v.clone());
                }
            }
        }

        let mut visitor = VariableVisitor {
            seen: HashSet::new(),
            vars: vec![],
        };

        walk_operation(&mut visitor, &self);
        visitor.vars
    }

    /// Replace `var` with a ground (non-variable) value. Checks for
    /// consistent unifications along the way: if everything's fine,
    /// returns `Some(grounded_term)`, but if an inconsistent ground
    /// (anti-)unification is detected, return `None`.
    pub fn ground(&self, var: Symbol, value: Term) -> Option<Self> {
        struct Grounder {
            var: Symbol,
            value: Term,
            invert: bool,
            consistent: bool,
        }

        impl Folder for Grounder {
            fn fold_term(&mut self, t: Term) -> Term {
                if let Value::Variable(v) = t.value() {
                    if v == &self.var {
                        return self.value.clone();
                    }
                }
                fold_term(t, self)
            }

            fn fold_operation(&mut self, o: Operation) -> Operation {
                match o.operator {
                    Operator::Unify | Operator::Eq | Operator::Neq => {
                        let neq = o.operator == Operator::Neq;

                        let l = self.fold_term(o.args[0].clone());
                        let r = self.fold_term(o.args[1].clone());
                        if l.is_ground() && r.is_ground() {
                            let consistent = if neq { l != r } else { l == r };
                            if self.invert {
                                if consistent {
                                    self.consistent = false;
                                    TRUE
                                } else {
                                    FALSE
                                }
                            } else if consistent {
                                TRUE
                            } else {
                                self.consistent = false;
                                FALSE
                            }
                        } else {
                            Operation {
                                operator: o.operator,
                                args: vec![l, r],
                            }
                        }
                    }
                    Operator::Not => {
                        self.invert = !self.invert;
                        let o = fold_operation(o, self);
                        self.invert = !self.invert;
                        o
                    }
                    Operator::Gt | Operator::Geq | Operator::Lt | Operator::Leq => {
                        let o = if self.invert { invert_operation(o) } else { o };
                        let left = self.fold_term(o.args[0].clone());
                        let right = self.fold_term(o.args[1].clone());
                        match (left.value(), right.value()) {
                            (Value::Number(_), Value::Number(_))
                            | (Value::Number(_), Value::Boolean(_))
                            | (Value::Boolean(_), Value::Number(_))
                            | (Value::Boolean(_), Value::Boolean(_))
                            | (Value::String(_), Value::String(_)) => {
                                if compare(o.operator, &left, &right).unwrap() {
                                    TRUE
                                } else {
                                    self.consistent = false;
                                    FALSE
                                }
                            }
                            _ => fold_operation(o, self),
                        }
                    }
                    _ => fold_operation(o, self),
                }
            }
        }

        let mut grounder = Grounder {
            var,
            value,
            invert: false,
            consistent: true,
        };
        let grounded = grounder.fold_operation(self.clone());
        if grounder.consistent {
            Some(grounded)
        } else {
            None
        }
    }

    /// Augment our constraints with those on `other`.
    ///
    /// Invariant: self and other are ANDs
    pub fn merge_constraints(&mut self, other: Self) {
        assert_eq!(self.operator, Operator::And);
        assert_eq!(other.operator, Operator::And);
        self.args.extend(other.args);
    }

    // Invert constraints in operation after CSP.
    pub fn invert(&self) -> Operation {
        self.clone_with_constraints(vec![op!(
            Not,
            term!(value!(Operation {
                operator: Operator::And,
                args: self
                    .constraints()
                    .iter()
                    .cloned()
                    .map(|o| term!(value!(o)))
                    .collect()
            }))
        )])
    }

    pub fn constraints(&self) -> Vec<Operation> {
        self.args
            .iter()
            .map(|a| a.value().as_expression().unwrap().clone())
            .collect()
    }

    // TODO(gj): can we replace every use of this w/ clone_with_new_constraint for Immutability
    // Purposes?
    pub fn add_constraint(&mut self, o: Operation) {
        assert_eq!(self.operator, Operator::And);
        self.args.push(o.into_term());
    }

    pub fn into_term(self) -> Term {
        Term::new_temporary(Value::Expression(self))
    }

    pub fn clone_with_constraints(&self, constraints: Vec<Operation>) -> Self {
        assert_eq!(self.operator, Operator::And);
        let mut new = self.clone();
        new.args = constraints.into_iter().map(|c| c.into_term()).collect();
        new
    }

    pub fn clone_with_new_constraint(&self, constraint: Term) -> Self {
        assert_eq!(self.operator, Operator::And);
        let mut new = self.clone();
        match constraint.value() {
            Value::Expression(e) if e.operator == Operator::And => new.args.extend(e.args.clone()),
            _ => new.args.push(constraint),
        }
        new
    }

    pub fn mirror(&self) -> Self {
        let args = self.args.clone().into_iter().rev().collect();
        match self.operator {
            Operator::Unify | Operator::Eq | Operator::Neq => Self {
                operator: self.operator,
                args,
            },
            Operator::Gt => Self {
                operator: Operator::Leq,
                args,
            },
            Operator::Geq => Self {
                operator: Operator::Lt,
                args,
            },
            Operator::Lt => Self {
                operator: Operator::Geq,
                args,
            },
            Operator::Leq => Self {
                operator: Operator::Gt,
                args,
            },
            _ => self.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::bindings::Bindings;
    use crate::error::{ErrorKind, PolarError, RuntimeError};
    use crate::events::QueryEvent;
    use crate::formatting::ToPolarString;
    use crate::polar::{Polar, Query};
    use crate::terms::{Call, Dictionary, InstanceLiteral, Pattern};

    macro_rules! assert_partial_expression {
        ($bindings:expr, $sym:expr, $right:expr) => {
            assert_eq!(
                $bindings
                    .get(&sym!($sym))
                    .expect(&format!("{} is unbound", $sym))
                    .value()
                    .as_expression()
                    .unwrap()
                    .to_polar(),
                $right
            )
        };
    }

    macro_rules! assert_partial_expressions {
        ($bindings:expr, $($sym:expr => $value:expr),*) => {
            {
                let bindings = $bindings;
                $(assert_partial_expression!(bindings, $sym, $value);)*
            }
        };
    }

    macro_rules! assert_query_done {
        ($query:expr) => {
            assert!(matches!($query.next_event()?, QueryEvent::Done { .. }));
        };
    }

    fn next_binding(query: &mut Query) -> Result<Bindings, PolarError> {
        let event = query.next_event()?;
        if let QueryEvent::Result { bindings, .. } = event {
            Ok(bindings)
        } else {
            panic!("not bindings, {:?}", &event);
        }
    }

    type TestResult = Result<(), PolarError>;

    #[test]
    fn basic_test() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x = 1;
               f(x) if x = 2;
               f(x) if x.a = 3 or x.b = 4;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(2));
        assert_partial_expression!(next_binding(&mut q)?, "x", "3 = _this.a");
        assert_partial_expression!(next_binding(&mut q)?, "x", "4 = _this.b");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_and() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y, z) if x = y and x = z;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), 1, 1])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), 1, 2])), false);
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_two_rule() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x, y, z) if x < y and x < z and g(x);
               g(x) if x < 3;
               g(x) if x < 4 or x < 5;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), 1, 2])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this < 1 and _this < 2 and _this < 3");
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this < 1 and _this < 2 and _this < 4");
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this < 1 and _this < 2 and _this < 5");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_isa() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x: Post) if x.foo = 1;
               f(x: User) if x.bar = 1;

               f(x: Post) if g(x.y);
               g(x: User) if x.z = 1;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let mut binding = || loop {
            match q.next_event().unwrap() {
                QueryEvent::Result { bindings, .. } => return bindings,
                QueryEvent::ExternalIsa { call_id, .. } => {
                    q.question_result(call_id, true).unwrap();
                }
                QueryEvent::ExternalIsaWithPath { call_id, .. } => {
                    q.question_result(call_id, true).unwrap();
                }
                e => panic!("unexpected event: {:?}", e),
            }
        };
        assert_partial_expression!(binding(), "x", "_this matches Post{} and 1 = _this.foo");
        assert_partial_expression!(binding(), "x", "_this matches User{} and 1 = _this.bar");
        assert_partial_expression!(
            binding(),
            "x",
            "_this matches Post{} and _this.y matches User{} and 1 = _this.y.z"
        );
        assert_query_done!(q);

        // Test permutations of variable states in isa.
        p.load_str(
            r#"h(x: (y));
               i(x: (y), y: (z));
               j(x: (x), y: (x));
               k(x, y: (y), y: (x));
               l(x: (x), x: (x));
               m(x) if [y] matches [x];
               n(x: (x)) if [y] matches [x];"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this matches _y_34");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [sym!("x"), sym!("y")])), false);
        assert_partial_expressions!(next_binding(&mut q)?,
            "x" => "_this matches _y_39 and _y_39 matches _z_40",
            "y" => "x matches _this and _this matches _z_40");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("j", [sym!("x"), sym!("y")])), false);
        assert_partial_expressions!(next_binding(&mut q)?,
            "x" => "_this matches _this and y matches _this",
            "y" => "x matches x and _this matches x");
        assert_query_done!(q);

        let mut q =
            p.new_query_from_term(term!(call!("k", [sym!("x"), sym!("y"), sym!("y")])), false);
        assert_partial_expressions!(next_binding(&mut q)?,
            "x" => "y matches y and y matches _this",
            "y" => "_this matches _this and _this matches x");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("l", [sym!("x"), sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this matches _this");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("m", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_y_54 matches _this");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("n", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "x",
            "_this matches _this and _y_58 matches _this"
        );
        assert_query_done!(q);

        // TODO(gj): Make the below work.
        // let mut q = p.new_query("x matches Integer and x = 1", false)?;
        // assert_partial_expression!(next_binding(&mut q)?, "x", "1 matches Integer");
        // assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_field_unification() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, {x: x});")?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), btreemap! {}])), false);
        assert_query_done!(q);

        let mut q = p.new_query_from_term(
            term!(call!("f", [sym!("x"), btreemap! {sym!("x") => term!(1)}])),
            false,
        );
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(sym!("x")));
        assert_eq!(
            next[&sym!("y")],
            // TODO(gj): do something with the x <-> _x_5 cycle?
            term!(btreemap! { sym!("x") => term!(sym!("_x_5")) })
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("f", [1, sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("y")], term!(btreemap! { sym!("x") => term!(1) }));
        assert_query_done!(q);

        let p = Polar::new();
        p.load_str("g(x, _: {x: x});")?;

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), btreemap! {}])), false);
        assert_query_done!(q);

        let mut q = p.new_query_from_term(
            term!(call!("g", [sym!("x"), btreemap! {sym!("x") => term!(1)}])),
            false,
        );
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        assert_partial_expressions!(next_binding(&mut q)?, "x" => "y.x = _this", "y" => "_this.x = x");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [1, sym!("y")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "y", "_this.x = 1");
        assert_query_done!(q);

        let p = Polar::new();
        p.load_str("h(x: X, _: {x: x});")?;

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x"), btreemap! {}])), false);
        assert_query_done!(q);

        let mut q = p.new_query_from_term(
            term!(call!("h", [sym!("x"), btreemap! {sym!("x") => term!(1)}])),
            false,
        );
        assert_partial_expression!(
            next_binding(&mut q)?,
            "x",
            "_this matches X{} and 1 = _this"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x"), sym!("y")])), false);
        assert_partial_expressions!(next_binding(&mut q)?,
            "x" => "_this matches X{} and y.x = _this",
            "y" => "_this.x matches X{}"
        );
        assert_query_done!(q);

        let binding = |q: &mut Query| loop {
            match q.next_event().unwrap() {
                QueryEvent::Result { bindings, .. } => return bindings,
                QueryEvent::ExternalIsa { call_id, .. } => {
                    q.question_result(call_id, true).unwrap();
                }
                e => panic!("unexpected event: {:?}", e),
            }
        };
        let mut q = p.new_query_from_term(term!(call!("h", [1, sym!("y")])), false);
        assert_partial_expression!(binding(&mut q), "y", "_this.x = 1");
        assert_query_done!(q);

        let p = Polar::new();
        p.load_str("i(x, _: Y{x: x});")?;

        let maybe_binding = |q: &mut Query| loop {
            match q.next_event().unwrap() {
                QueryEvent::Result { bindings, .. } => return Some(bindings),
                QueryEvent::ExternalIsa { call_id, .. } => {
                    q.question_result(call_id, true).unwrap();
                }
                QueryEvent::Done { .. } => return None,
                e => panic!("unexpected event: {:?}", e),
            }
        };
        let mut q = p.new_query_from_term(term!(call!("i", [sym!("x"), btreemap! {}])), false);
        assert!(maybe_binding(&mut q).is_none());
        assert_query_done!(q);

        let mut q = p.new_query_from_term(
            term!(call!("i", [sym!("x"), btreemap! {sym!("x") => term!(1)}])),
            false,
        );
        assert_eq!(maybe_binding(&mut q).unwrap()[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [sym!("x"), sym!("y")])), false);
        assert_partial_expressions!(next_binding(&mut q)?,
            "x" => "y matches Y{} and y.x = _this",
            "y" => "_this matches Y{} and _this.x = x");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [1, sym!("y")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "y",
            "_this matches Y{} and _this.x = 1"
        );
        assert_query_done!(q);

        let p = Polar::new();
        p.load_str("j(x: X, _: Y{x: x});")?;

        let mut q = p.new_query_from_term(term!(call!("j", [sym!("x"), btreemap! {}])), false);
        assert!(maybe_binding(&mut q).is_none());
        assert_query_done!(q);

        let mut q = p.new_query_from_term(
            term!(call!("j", [sym!("x"), btreemap! {sym!("x") => term!(1)}])),
            false,
        );
        assert_partial_expression!(
            maybe_binding(&mut q).unwrap(),
            "x",
            "_this matches X{} and 1 = _this"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("j", [sym!("x"), sym!("y")])), false);
        assert_partial_expressions!(next_binding(&mut q)?,
            "x" => "y matches Y{} and _this matches X{} and y.x = _this",
            "x" => "y matches Y{} and _this matches X{} and y.x = _this",
            "y" => "_this matches Y{} and _this.x matches X{}"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("j", [1, sym!("y")])), false);
        assert_partial_expression!(binding(&mut q), "y", "_this matches Y{} and _this.x = 1");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_isa_with_fields() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x: Post{id: 1});
               g(x: Post{id: 1}) if x matches {id: 2}; # Will fail.
               h(x: Post{id: 1}) if x matches Post{id: 2};
               i(x: Post{id: 1}) if x matches User{id: 2}; # Will fail.
               j(x: Post{id: 1}) if x matches {id: 2, bar: 2};
               k(x: Post{id: 1, bar: 1}) if x matches User{id: 2}; # Will fail.
               l(x: Post{id: 1, bar: 3}) if x matches Post{id: 2} and x.y = 1;
               m(x: {id: 1, bar: 1}) if x matches {id: 2};
               n(x: {id: 1}) if x matches {id: 2, bar: 2};
               o(x: {id: 1});
               p(x: {id: 1}) if x matches {id: 2};
               q(x: {id: 1}) if x matches Post{id: 2};
               r(x: 1);"#,
        )?;
        let next_binding = |q: &mut Query| loop {
            match q.next_event().unwrap() {
                QueryEvent::Result { bindings, .. } => return Some(bindings),
                QueryEvent::ExternalIsSubclass {
                    call_id,
                    left_class_tag,
                    right_class_tag,
                } => {
                    q.question_result(call_id, left_class_tag.0.starts_with(&right_class_tag.0))
                        .unwrap();
                }
                QueryEvent::Done { .. } => return None,
                _ => panic!("not bindings"),
            }
        };

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q).unwrap(),
            "x",
            "_this matches Post{} and _this.id = 1"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        // TODO(gj): Inconsistent dot op unifications.
        assert_partial_expression!(
            next_binding(&mut q).unwrap(),
            "x",
            "_this matches Post{} and _this.id = 1 and _this.id = 2"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q).unwrap(),
            "x",
            "_this matches Post{} and _this.id = 1 and _this.id = 2"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [sym!("x")])), false);
        assert!(next_binding(&mut q).is_none());
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("j", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q).unwrap(),
            "x",
            "_this matches Post{} and _this.id = 1 and _this.id = 2 and _this.bar = 2"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("k", [sym!("x")])), false);
        assert!(next_binding(&mut q).is_none());
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("l", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q).unwrap(),
            "x",
            "_this matches Post{} and _this.id = 1 and _this.bar = 3 and _this.id = 2 and 1 = _this.y"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("m", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q).unwrap(),
            "x",
            "_this.id = 1 and _this.bar = 1 and _this.id = 2"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("n", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q).unwrap(),
            "x",
            "_this.id = 1 and _this.id = 2 and _this.bar = 2"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("o", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q).unwrap(), "x", "_this.id = 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("p", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q).unwrap(),
            "x",
            "_this.id = 1 and _this.id = 2"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("q", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q).unwrap(),
            "x",
            "_this.id = 1 and _this matches Post{} and _this.id = 2"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("r", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q).unwrap()[&sym!("x")], term!(1));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_isa_two_rule() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x: Post) if x.foo = 0 and g(x);
               f(x: User) if x.bar = 1 and g(x);
               g(x: Post) if x.post = 1;
               g(x: PostSubclass) if x.post_subclass = 1;
               g(x: User) if x.user = 1;
               g(x: UserSubclass) if x.user_subclass = 1;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let mut next_binding = || loop {
            match q.next_event().unwrap() {
                QueryEvent::Result { bindings, .. } => return bindings,
                QueryEvent::ExternalIsSubclass {
                    call_id,
                    left_class_tag,
                    right_class_tag,
                } => {
                    q.question_result(call_id, left_class_tag.0.starts_with(&right_class_tag.0))
                        .unwrap();
                }
                e => panic!("unexpected event: {:?}", e),
            }
        };
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this matches Post{} and 0 = _this.foo and 1 = _this.post"
        );
        assert_partial_expression!(
            next_binding(),
"x",
            "_this matches Post{} and 0 = _this.foo and _this matches PostSubclass{} and 1 = _this.post_subclass"
        );
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this matches User{} and 1 = _this.bar and 1 = _this.user"
        );
        assert_partial_expression!(
            next_binding(),
"x",
            "_this matches User{} and 1 = _this.bar and _this matches UserSubclass{} and 1 = _this.user_subclass"
        );
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_isa_subclass_superclass() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x: PostSubclass) if g(x);
               g(x: Post);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let mut next_binding = || loop {
            match q.next_event().unwrap() {
                QueryEvent::Result { bindings, .. } => return bindings,
                QueryEvent::ExternalIsSubclass {
                    call_id,
                    left_class_tag,
                    right_class_tag,
                } => {
                    q.question_result(call_id, left_class_tag.0.starts_with(&right_class_tag.0))
                        .unwrap();
                }
                _ => panic!("not bindings"),
            }
        };
        assert_partial_expression!(
            next_binding(),
            "x",
            "_this matches PostSubclass{} and _this matches Post{}"
        );
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_comparison() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"positive(x) if x > 0;
               positive(x) if x > 0 and x < 0;
               zero(x) if x == 0;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("positive", [sym!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this > 0");
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this > 0 and _this < 0");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("zero", [sym!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "_this == 0");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_comparison_with_variable_indirection() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x > 1 and y = z and x = z and y = 1;
               g(x) if x > 1 and y = z and x == z and y = 1;
               h(x) if x > 1 and y = z and x = z and y = 2;
               i(x) if x > 1 and y = z and x == z and y = 2;

               j(y) if x = y and y == z and z = 1 and x = 1;
               k(y) if x = y and y == z and z = 1 and x = 2;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this > 1 and _this = 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this > 1 and _this == 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this > 1 and _this = 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this > 1 and _this == 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("j", [sym!("y")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "y", "_this = 1 and _this == 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("k", [sym!("y")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "y", "_this = 2 and _this == 1");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_comparison_dot() -> TestResult {
        let p = Polar::new();
        p.load_str("a_positive(x) if x.a > 0 and 0 < x.a;")?;
        let mut q = p.new_query_from_term(term!(call!("a_positive", [sym!("x")])), false);
        // TODO(gj): Canonicalize comparisons.
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this.a > 0 and 0 < _this.a");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_nested_dot_ops() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x.y.z > 0;
               g(x) if x.y = 0 and x.y > 1 and x.y.z > 1;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this.y.z > 0");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "x",
            "0 = _this.y and _this.y > 1 and _this.y.z > 1"
        );
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_multiple_partials() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y) if x = 1 and y = 2;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(2));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_in_arithmetic_op() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if x = x + 0;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this + 0 = _this");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_method_call_on_partial() -> TestResult {
        let p = Polar::new();
        p.load_str("g(x) if x.foo();")?;
        let mut q = p.new_query_from_term(term!(call!("g", [sym!("a")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(
            error,
            PolarError {
                kind: ErrorKind::Runtime(RuntimeError::Unsupported { .. }),
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn test_rule_filtering_with_partials() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if g(x.c);
               g(y: B) if y.b > 0;
               g(y: C{foo: 1, bar: 2}) if y.c > 0;

               h(y: A{foo: 1}) if i(y.b);
               i(y: C) if y.c > 0;
               i(y: B{bar: 2}) if y.b > 0;"#,
        )?;

        let next_binding = |q: &mut Query| loop {
            match q.next_event().unwrap() {
                QueryEvent::Result { bindings, .. } => return bindings,
                QueryEvent::ExternalIsSubclass { call_id, .. } => {
                    q.question_result(call_id, false).unwrap();
                }
                QueryEvent::ExternalIsaWithPath {
                    call_id,
                    path,
                    class_tag,
                    ..
                } => {
                    let last_segment = path.last().unwrap();
                    q.question_result(
                        call_id,
                        last_segment.value().as_string().unwrap().to_uppercase() == class_tag.0,
                    )
                    .unwrap();
                }
                QueryEvent::None => (),
                e => panic!("not bindings: {:?}", e),
            }
        };

        // Register `x` as a partial.
        p.register_constant(
            sym!("x"),
            op!(
                And,
                op!(Isa, term!(sym!("x")), term!(pattern!(instance!("A")))).into_term()
            )
            .into_term(),
        );

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q),
            "x",
            "_this matches A{} and _this.c matches C{} and _this.c.foo = 1 and _this.c.bar = 2 and _this.c.c > 0"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("y")])), false);
        assert_partial_expression!(
            next_binding(&mut q),
            "y",
            "_this matches A{} and _this.foo = 1 and _this.b matches B{} and _this.b.bar = 2 and _this.b.b > 0"
        );
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_unification_1() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x, y) if x = y;
               f(x, y) if x = y and 1 = x;
               f(x, y) if 2 = y and x = y and x = 1;

               g(x, y) if x = 1 and y = 2;
               g(x, y) if x = 1 and y = 2 and x = y;"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(sym!("y")));
        assert_eq!(next[&sym!("y")], term!(sym!("x")));
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(2));
        assert_query_done!(q);

        // Register `y` as a partial.
        p.register_constant(sym!("y"), term!(value!(op!(And))));
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "_this = y", "y" => "x = _this");
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(1));
        //assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(2));
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_partial_unification_2() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x, y) if x = y;
               f(x, y) if x = y and 1 = x;
               f(x, y) if 2 = y and x = y and x = 1;

               g(x, y) if x = 1 and y = 2;
               g(x, y) if x = 1 and y = 2 and x = y;"#,
        )?;

        // Register `x` as a partial.
        p.register_constant(sym!("x"), term!(value!(op!(And))));
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "_this = y", "y" => "x = _this");
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(2));
        assert_query_done!(q);

        // Register `y` as a partial.
        p.register_constant(sym!("y"), term!(value!(op!(And))));
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "_this = y", "y" => "x = _this");
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!(1));
        assert_eq!(next[&sym!("y")], term!(2));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_comparing_partials() -> TestResult {
        let p = Polar::new();

        p.load_str("f(x, y) if x > y;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "_this > y", "y" => "x > _this");

        p.load_str("g(x, y) if y = 1 and x > y;")?;
        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this > 1");
        assert_eq!(next[&sym!("y")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), value!(1)])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this > 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), value!(2)])), false);
        assert_query_done!(q);

        p.load_str("h(x, y) if x > y and y = 1;")?;
        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "_this > 1", "y" => "1 = _this and x > 1");
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_dot_lookup_with_unbound_as_field() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if {a: 1, b: 2}.(x) > 0;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!("a"));
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!("b"));
        Ok(())
    }

    #[test]
    fn test_dot_lookup_with_partial_as_field() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y) if {a: y, b: y}.(x) > 0;")?;
        p.register_constant(sym!("x"), term!(value!(op!(And))));
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!("a"));
        assert_partial_expression!(next, "y", "_this > 0");
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("x")], term!("b"));
        assert_partial_expression!(next, "y", "_this > 0");
        Ok(())
    }

    #[test]
    fn test_partial_inverter() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if not x = 1;
               g(x) if not x > 1;
               h(x) if not (x = 1 and x = 2);
               i(x) if not (x = 1 or x = 2);
               j(x) if not (not x = 1);
               k(x) if not (not (not x = 1));"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this != 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this <= 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(sym!("x")));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this != 1 and _this != 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("j", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("k", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this != 1");
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_negate_conjunctions() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if not (y = 1 and x.foo = y);
               g(x) if not (x.foo = y and 1 = y);
               h(x) if not (y = 1 and x.foo.bar = y);
               i(x) if not (y = x.foo.bar and 1 = y);"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this.foo != 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this.foo != 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this.foo.bar != 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 != _this.foo.bar");
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_negate_dot() -> TestResult {
        let p = Polar::new();
        p.load_str(r#"f(x) if not (x.y = 1 and x.b = 2);"#)?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 != _this.y or 2 != _this.b");
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn partially_negated_constraints() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x = 3 and not (x = 1 and (not x = 2));
               g(x) if x = 3 and not (x = 3 and (not x = 2));
               h(x) if not (x = 1 and (not x = 2));
               i(x) if x = 1 and not (x = 2 or x = 3);
               j(x) if x != 2 and x = 1;
               k(x) if (x != 2 or x != 3) and x = 1;
               l(x) if not (x = 2) and x = 1;
               m(x) if not (x = 2 or x = 3) and x = 1;
               n(x) if not (x != 2) and x = 1;
               o(x) if x > 1 and not x = 1;
               p(x) if x > 1 and not x = 1 and x = 2;
               q(x) if not (x = 2 or x = 3 or x = 4) and x = 1;
               r(x) if not (x = 2 or x > 3 or x = 4) and x = 1;
               s(x) if not (x = 2 or x > 3 or x = 4) and x > 1;
               t(x) if not (x <= 0 or x <= 1 or x <= 2 or not (x > 3 or x > 4 or x > 5));
               u(x) if not ((x <= 0 or x <= 1 or x <= 2 or not (x > 3 or x > 4 or x > 5)) and x = 6);
               v(x) if x = y and not (y = 1) and x = y;
               w(x) if not ((x <= 0 or x <= 1 or x <= 2 or not (x > 9 or x > 8 or x > 7)) and x = 6);"#
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(3));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this != 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("j", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("k", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("l", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("m", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("n", [sym!("x")])), false);
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("o", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this > 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("p", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(2));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("q", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("r", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("s", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "x",
            "_this != 2 and _this <= 3 and _this != 4 and _this > 1"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("t", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "x",
            "_this > 0 and _this > 1 and _this > 2 and _this > 3 or _this > 4 or _this > 5"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("u", [sym!("x")])), false);
        let binding = next_binding(&mut q)?;
        // This is unbound because any input succeeds.
        assert_eq!(binding.get(&sym!("x")).unwrap(), &term!(sym!("x")));
        assert_query_done!(q);

        let mut v = p.new_query_from_term(term!(call!("v", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut v)?, "x", "_this != 1");
        assert_query_done!(v);

        let mut q = p.new_query_from_term(term!(call!("w", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this != 6");
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_doubly_negated_ground() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if not (x != 1);
               g(x) if not (not (x = 1));"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_partial_before_negation() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x > 1 and not (x < 0);
               g(x) if x > 1 and not (x = 2);"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this > 1 and _this >= 0");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this > 1 and _this != 2");
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn partial_with_unbound_variables() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if not (x.foo = y);
               g(x) if not (x.foo.bar = y);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this.foo != _y_9");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this.foo.bar != _y_18");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_negate_disjunctions() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if not (x.foo = 1 or 2 = x.foo);
               g(x) if not (1 = x or x = 2);
               h(x) if not (x.foo.bar = 1 or 2 = x.foo.bar);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "1 != _this.foo and 2 != _this.foo");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this != 1 and _this != 2");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "1 != _this.foo.bar and 2 != _this.foo.bar");
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_trivial_partials() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x);
               g(x) if false;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(sym!("x")));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_in_partial_lhs() -> TestResult {
        let p = Polar::new();
        p.load_str("lhs(x) if x in [1, 2];")?;
        // Partials on the LHS of `in` accumulate constraints disjunctively.
        let mut q = p.new_query_from_term(term!(call!("lhs", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(1));
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(2));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_negated_in_partial_lhs() -> TestResult {
        let p = Polar::new();
        p.load_str("not_lhs(x) if not x in [1, 2];")?;
        // Inverting an `in` produces a conjunction of the inverted disjunctive constraints.
        let mut q = p.new_query_from_term(term!(call!("not_lhs", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "_this != 1 and _this != 2");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_contains_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"contains(x, y) if x in y;
               contains_dot(x, y) if x in y.foo;
               contains_dot_dot(x, y) if x in y.foo.bar and y.foo = 2;"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("contains", [1, sym!("y")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 in _this");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("contains_dot", [1, sym!("y")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 in _this.foo");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("contains_dot_dot", [1, sym!("y")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "y",
            "1 in _this.foo.bar and 2 = _this.foo"
        );
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_in_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if y in x.values;
               g(x, y) if y in x.values;
               h(x) if y in x.values and (y.bar = 1 and y.baz = 2) or y.bar = 3;
               i() if x in y;
               j() if x in [];
               k(x) if x > 1 and x in [2, 3];
               l(x) if y in x;
               m(x) if 1 in y and y = x;"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "x" => "_y_12 in _this.values"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "y in _this.values");
        assert_partial_expression!(next, "y", "_this in x.values");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "x" => "_y_27 in _this.values and 1 = _y_27.bar and 2 = _y_27.baz"
        );
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "x" => "_y_27 in _this.values and 3 = _y_27.bar"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i")), false);
        assert!(next_binding(&mut q)?.is_empty());
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("j")), false);
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("k", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(2));
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(3));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("l", [sym!("x")])), false);
        assert_partial_expressions!(next_binding(&mut q)?, "x" => "_y_39 in _this");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("m", [sym!("x")])), false);
        assert_partial_expressions!(next_binding(&mut q)?, "x" => "1 in _this");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_in_partial_2() -> TestResult {
        let p = Polar::new();
        p.load_str(r#"f(x) if y in x and y = 1;"#)?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 in _this");
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_that_cut_with_partial_errors() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if cut;")?;
        p.register_constant(sym!("x"), op!(And).into_term());
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(
            error,
            PolarError {
                kind: ErrorKind::Runtime(RuntimeError::Unsupported { .. }),
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn test_cut_with_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x = 1;
               f(x) if x = 2 and cut;
               f(x) if x = 3;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("a")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(1));
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(2));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_conditional_cut_with_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if x > 1 and cut or x = 2 and x = 3;
               g(1) if cut;
               g(2);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("a")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(3));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("a")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(1));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_method_sorting_with_cut_and_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x, y) if cut and x = 1;
               f(x, y: 2) if x = 2;"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("a"), value!(2)])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(2));
        assert_eq!(next_binding(&mut q)?[&sym!("a")], term!(1));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn nonlogical_inversions() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x) if not print(x);")?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_nested_dot_in() -> TestResult {
        let p = Polar::new();
        p.load_str("f(x, y) if x in y.a.b.c;")?;
        let mut q = p.new_query_from_term(term!(call!("f", [1, sym!("a")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "a", "1 in _this.a.b.c");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_nested_dot_lookup() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x, y) if x = y.a.b.c;
               f(x, y) if x > y.a.b.c and x < y.a.b and y.a.b.c > x;
               f(x, y) if x = y.a;
               f(x, y) if x = y.a.b;
               f(x, y) if x = y.a.b.c.d;
               f(x, y) if x = y.a.b.c.d.e;
               f(x, y) if x = y.a.b.c.d.e.f;"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [1, sym!("y")])), false);

        assert_partial_expression!(next_binding(&mut q)?, "y", "1 = _this.a.b.c");
        assert_partial_expression!(
            next_binding(&mut q)?,
            "y",
            "1 > _this.a.b.c and 1 < _this.a.b and _this.a.b.c > 1"
        );
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 = _this.a");
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 = _this.a.b");
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 = _this.a.b.c.d");
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 = _this.a.b.c.d.e");
        assert_partial_expression!(next_binding(&mut q)?, "y", "1 = _this.a.b.c.d.e.f");
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_partial_negated_isa() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x) if (not x matches Foo{} or not g(x)) and x = 1;
               g(_: Bar);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q)?,
            "x",
            "1 = _this and not 1 matches Foo{}"
        );
        assert_partial_expression!(
            next_binding(&mut q)?,
            "x",
            "1 = _this and not 1 matches Bar{}"
        );
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_multiple_gt_three_variables() -> TestResult {
        let p = Polar::new();
        p.load_str(r#"f(x, y, z) if x > z and y > z;"#)?;
        let mut q =
            p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y"), sym!("z")])), false);
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "x" => "_this > z and y > z",
            "y" => "x > z and _this > z",
            "z" => "x > _this and y > _this"
        );
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_negated_any_value() -> TestResult {
        let p = Polar::new();
        p.load_str(r#"f(x, y) if x = 1 and not (y = 1 and x = 2);"#)?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let bindings = next_binding(&mut q)?;
        assert_eq!(bindings.get(&sym!("x")).unwrap(), &term!(1));
        // y is unbound (to itself)
        assert_eq!(bindings.get(&sym!("y")).unwrap(), &term!(sym!("y")));
        assert_query_done!(q);
        Ok(())
    }

    #[test]
    fn test_negation_two_rules() -> TestResult {
        let p = Polar::new();
        // TODO: More complicated version:
        //  f(x) if not g(z) and z = x;
        //  g(y) if y = 1;
        p.load_str(
            r#"f(x) if not g(x);
                      g(y) if y = 1;

                h(x) if not (not g(x));
                      "#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let bindings = next_binding(&mut q)?;
        assert_partial_expressions!(
            &bindings,
            "x" => "_this != 1"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("f", [2])), false);
        assert_eq!(next_binding(&mut q)?.len(), 0);
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("f", [1])), false);
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        let bindings = next_binding(&mut q)?;
        assert_eq!(bindings.get(&sym!("x")).unwrap(), &term!(1));
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_constraint_no_input_variables() -> TestResult {
        let p = Polar::new();
        // TODO: More complicated version:
        //  f(x) if not g(z) and z = x;
        //  g(y) if y = 1;
        p.load_str(r#"f() if x = 1;"#)?;

        let mut q = p.new_query_from_term(term!(call!("f", [])), false);
        let r = next_binding(&mut q)?;
        assert_eq!(r.len(), 0);
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_output_variable() -> TestResult {
        let p = Polar::new();
        p.load_str(r#"f(a, b) if a = b;"#)?;

        let mut q = p.new_query_from_term(term!(call!("f", [1, sym!("x")])), false);
        let r = next_binding(&mut q)?;
        assert_eq!(r.get(&sym!("x")).unwrap(), &term!(1));
        assert_query_done!(q);

        Ok(())
    }

    // TODO(gj): add test where we have a partial prior to an inversion
    // TODO (dhatch): We have few tests involving multiple rules and partials.
}
