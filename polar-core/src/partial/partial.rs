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
    use Operator::*;
    fn invert_args(args: Vec<Term>) -> Vec<Term> {
        args.into_iter()
            .map(|t| {
                t.clone_with_value(value!(invert_operation(t.as_expression().unwrap().clone())))
            })
            .collect()
    }

    match operator {
        // noop
        Debug | Print | New | Dot => Operation { operator, args },

        // de morgan
        And => Operation {
            operator: Or,
            args: invert_args(args),
        },
        Or => Operation {
            operator: And,
            args: invert_args(args),
        },

        // opposite operator
        Unify | Eq => Operation {
            operator: Neq,
            args,
        },
        Neq => Operation {
            operator: Unify,
            args,
        },
        Gt => Operation {
            operator: Leq,
            args,
        },
        Geq => Operation { operator: Lt, args },
        Lt => Operation {
            operator: Geq,
            args,
        },
        Leq => Operation { operator: Gt, args },

        // double negative
        Not => args[0].as_expression().expect("negated expression").clone(),

        // preserve the not
        Isa | In => Operation {
            operator: Not,
            args: vec![term!(Operation { operator, args })],
        },

        _ => todo!("negate {:?}", operator),
    }
}

impl Operation {
    /// Construct & return a set of symbols that occur in this operation.
    pub fn variables(&self) -> Vec<Symbol> {
        struct VariableVisitor {
            seen: HashSet<Symbol>,
            vars: Vec<Symbol>, // FIXME gw
                               // you may be wondering, why keep both a vec and a set? why not just a set?
                               // it's because there's a ton of sqlalchemy tests that break if the order of
                               // the variables changes :(
        }

        impl Visitor for VariableVisitor {
            fn visit_variable(&mut self, v: &Symbol) {
                if self.seen.insert(v.clone()) {
                    self.vars.push(v.clone())
                }
            }
        }

        let mut visitor = VariableVisitor {
            seen: HashSet::new(),
            vars: vec![],
        };

        walk_operation(&mut visitor, self);
        visitor.vars
    }

    /// Replace `var` with a ground (non-variable) value. Checks for
    /// consistent unifications along the way: if everything's fine,
    /// returns `Some(grounded_term)`, but if an inconsistent ground
    /// (anti-)unification is detected, return `None`.
    pub fn ground(&self, var: &Symbol, value: Term) -> Option<Self> {
        struct Grounder<'a> {
            var: &'a Symbol,
            value: Term,
            invert: bool,
            consistent: bool,
        }

        impl<'a> Folder for Grounder<'a> {
            fn fold_term(&mut self, t: Term) -> Term {
                if let Value::Variable(v) = t.value() {
                    if v == self.var {
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
                                if compare(o.operator, &left, &right, None).unwrap() {
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

    fn constrain(&mut self, t: Term) {
        if !self.args.iter().any(|p| *p == t) {
            self.args.push(t);
        }
    }
    // TODO(gj): can we replace every use of this w/ clone_with_new_constraint for Immutability
    // Purposes?
    pub fn add_constraint(&mut self, o: Operation) {
        assert_eq!(self.operator, Operator::And);
        self.constrain(o.into());
    }

    /// Augment our constraints with those on `other`.
    ///
    /// Invariant: self and other are ANDs
    pub fn merge_constraints(mut self, other: Self) -> Self {
        assert_eq!(self.operator, Operator::And);
        assert_eq!(other.operator, Operator::And);
        for t in other.args.into_iter() {
            self.constrain(t);
        }
        self
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
            .map(|a| a.as_expression().unwrap().clone())
            .collect()
    }

    pub fn clone_with_constraints(&self, constraints: Vec<Operation>) -> Self {
        assert_eq!(self.operator, Operator::And);
        let mut new = self.clone();
        new.args = constraints.into_iter().map(|c| c.into()).collect();
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

impl Iterator for Operation {
    type Item = Term;
    fn next(&mut self) -> Option<Term> {
        self.args.pop()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::bindings::Bindings;
    use crate::error::{ErrorKind, PolarError, PolarResult, RuntimeError};
    use crate::events::QueryEvent;
    use crate::polar::Polar;
    use crate::query::Query;
    use crate::terms::{Call, Dictionary, InstanceLiteral, Pattern};

    macro_rules! assert_partial_expression {
        ($bindings:expr, $sym:expr, $right:expr) => {
            assert_eq!(
                $bindings
                    .get(&sym!($sym))
                    .expect(&format!("{} is unbound", $sym))
                    .as_expression()
                    .unwrap()
                    .to_string(),
                $right
            )
        };
    }
    macro_rules! assert_partial_binding {
        ($bindings:expr, $sym:expr, $($args:expr),+) => {
            let l = $bindings
                    .get(&sym!($sym))
                    .expect(&format!("{} is unbound", $sym))
                    .as_expression()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .collect::<HashSet<Term>>();
            let r = hashset! { $($args),+ };
            let fmt = |hs: &HashSet<Term>| format!("{{ {} }}", hs.iter().map(Term::to_string).collect::<Vec<_>>().join(", "));

            assert_eq!(&l, &r, "{} != {}", fmt(&l), fmt(&r));

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
            let event = $query.next_event()?;
            assert!(
                matches!(event, QueryEvent::Done { .. }),
                "expected `QueryEvent::Done`, got: {}",
                if let QueryEvent::Result { bindings, .. } = event {
                    format!(
                        "Bindings: {}",
                        bindings
                            .iter()
                            .map(|(k, v)| format!("{}: {}", k.0, v))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                } else {
                    format!("{:#?}", event)
                }
            );
        };
    }

    macro_rules! nextb {
        ($query:expr) => {{
            let query = $query;
            let event = query.next_event()?;
            if let QueryEvent::Result { bindings, .. } = event {
                bindings
            } else {
                panic!("not bindings, {:?}", &event);
            }
        }};
    }

    fn next_binding(query: &mut Query) -> PolarResult<Bindings> {
        let event = query.next_event()?;
        if let QueryEvent::Result { bindings, .. } = event {
            Ok(bindings)
        } else {
            panic!("not bindings, {:?}", &event);
        }
    }

    type TestResult = PolarResult<()>;

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
    fn test_partial_isa_unify() -> TestResult {
        let p = Polar::new();
        p.load_str("foo(u: User, x: Post) if x.user = u;")?;
        let mut q = p.new_query_from_term(term!(call!("foo", [sym!("user"), sym!("post")])), false);
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "user" => "_this matches User{} and post matches Post{} and _this = post.user",
            "post" => "user matches User{} and _this matches Post{} and user = _this.user"
        );
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

        p.clear_rules();

        // Test permutations of variable states in isa.
        // NOTE(gj): only one permutation remains parse-able.
        p.load_str("m(x) if [_y] matches [x];")?;
        let mut q = p.new_query_from_term(term!(call!("m", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "__y_36 matches _this");
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
        p.load_str("f(x0, {x: x0});")?;

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
            term!(btreemap! { sym!("x") => term!(sym!("_x0_5")) })
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("f", [1, sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("y")], term!(btreemap! { sym!("x") => term!(1) }));
        assert_query_done!(q);

        let p = Polar::new();
        p.load_str("g(x1, _: {x: x1});")?;

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
        p.load_str("h(x2: X, _: {x: x2});")?;

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x"), btreemap! {}])), false);
        assert_query_done!(q);

        let mut q = p.new_query_from_term(
            term!(call!("h", [sym!("x"), btreemap! {sym!("x") => term!(1)}])),
            false,
        );
        let binding = |q: &mut Query| loop {
            match q.next_event().unwrap() {
                QueryEvent::Result { bindings, .. } => return bindings,
                QueryEvent::ExternalIsa { call_id, .. } => {
                    q.question_result(call_id, true).unwrap();
                }
                e => panic!("unexpected event: {:?}", e),
            }
        };

        assert_eq!(binding(&mut q)[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x"), sym!("y")])), false);
        assert_partial_expressions!(next_binding(&mut q)?,
            "x" => "_this matches X{} and y.x = _this",
            "y" => "x matches X{} and _this.x = x"
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
        p.load_str("i(x3, _: Y{x: x3});")?;

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
        p.load_str("j(x4: X, _: Y{x: x4});")?;

        let mut q = p.new_query_from_term(term!(call!("j", [sym!("x"), btreemap! {}])), false);
        assert!(maybe_binding(&mut q).is_none());
        assert_query_done!(q);

        let mut q = p.new_query_from_term(
            term!(call!("j", [sym!("x"), btreemap! {sym!("x") => term!(1)}])),
            false,
        );
        assert_eq!(maybe_binding(&mut q).unwrap()[&sym!("x")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("j", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_binding!(
            next,
            "x",
            term!(op!(
                Unify,
                term!(op!(Dot, var!("y"), str!("x"))),
                var!("_this")
            )),
            term!(op!(Isa, var!("y"), term!(pattern!(instance!("Y"))))),
            term!(op!(Isa, var!("_this"), term!(pattern!(instance!("X")))))
        );

        assert_partial_binding!(
            next,
            "y",
            term!(op!(
                Unify,
                term!(op!(Dot, var!("_this"), str!("x"))),
                var!("x")
            )),
            term!(op!(Isa, var!("_this"), term!(pattern!(instance!("Y"))))),
            term!(op!(Isa, var!("x"), term!(pattern!(instance!("X")))))
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
            r#"f(_: Post{id: 1});
               g(x: Post{id: 1}) if x matches {id: 2}; # Will fail.
               h(x: Post{id: 1}) if x matches Post{id: 2};
               i(x: Post{id: 1}) if x matches User{id: 2}; # Will fail.
               j(x: Post{id: 1}) if x matches {id: 2, bar: 2};
               k(x: Post{id: 1, bar: 1}) if x matches User{id: 2}; # Will fail.
               l(x: Post{id: 1, bar: 3}) if x matches Post{id: 2} and x.y = 1;
               m(x: {id: 1, bar: 1}) if x matches {id: 2};
               n(x: {id: 1}) if x matches {id: 2, bar: 2};
               o(_: {id: 1});
               p(x: {id: 1}) if x matches {id: 2};
               q(x: {id: 1}) if x matches Post{id: 2};
               r(_: 1);"#,
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
               g(_: Post);"#,
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
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(nextb!(&mut q), "x", "1 > 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(2));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [sym!("x")])), false);
        assert_partial_expression!(nextb!(&mut q), "x", "2 > 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("j", [sym!("y")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("y")], term!(1));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("k", [sym!("y")])), false);
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
            error.0,
            ErrorKind::Runtime(RuntimeError::Unsupported { .. }),
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
               i(y: B{bar: 2}) if y.b > 0;

               # traversing `in` is tough!
               r(a: A) if b in a.b and t(b);
               s(a: A) if b in a.b and b matches B and u(b.c);
               # This is still incorrect
               # this should be equivalent to `s`
               s_bad(a: A) if b in a.b and u(b.c);
               t(b: B) if b.foo = 1;
               t(b: C) if b.foo = 2;
               u(c: C) if c.bar = 1;
               u(c: D) if c.bar = 2;

               # the rebinding here sometimes trips up the simplifier
               # PR: https://github.com/osohq/oso/pull/1289
               a(x: A) if y = x.b and b(y);
               b(b: B) if b.z = 1;
               b(c: C) if c.z = 2;
               "#,
        )?;

        #[track_caller]
        fn next_binding(q: &mut Query) -> Bindings {
            loop {
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
                            last_segment.as_string().unwrap().to_uppercase() == class_tag.0,
                        )
                        .unwrap();
                    }
                    QueryEvent::None => (),
                    e => panic!("not bindings: {:?}", e),
                }
            }
        }

        // Register `x` as a partial.
        p.register_constant(
            sym!("x"),
            op!(
                And,
                op!(Isa, term!(sym!("x")), term!(pattern!(instance!("A")))).into()
            )
            .into(),
        )?;

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

        let mut q = p.new_query_from_term(term!(call!("a", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q),
            "x",
            "_this matches A{} and _this.b matches B{} and 1 = _this.b.z"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("r", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q),
            "x",
            "_this matches A{} and _b_81 in _this.b and _b_81 matches B{} and 1 = _b_81.foo"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("s", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q),
            "x",
            "_this matches A{} and _b_92 in _this.b and _b_92 matches B{} and _b_92.c matches C{} and 1 = _b_92.c.bar"
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("s_bad", [sym!("x")])), false);
        assert_partial_expression!(
            next_binding(&mut q),
            "x",
            "_this matches A{} and _b_115 in _this.b and _b_115.c matches C{} and 1 = _b_115.c.bar"
        );
        // @TODO(sam): this result is incorrect. We *could* know
        // that `_b_104` matches B{} by checking `a.b` first
        // or perhaps by also traversing `in` and checking whether a.b.c matches D
        assert_partial_expression!(
            next_binding(&mut q),
            "x",
            "_this matches A{} and _b_115 in _this.b and _b_115.c matches D{} and 2 = _b_115.c.bar"
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
        p.register_constant(sym!("y"), term!(value!(op!(And))))?;
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
    fn test_partial_unification_2() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"f(x, y) if x = y;
               f(x, y) if x = y and 1 < x;
               f(x, y) if 2 > y and x = y and x > 1;

               g(x, y) if x > 1 and y < 2;
               g(x, y) if x > 1 and y < 2 and x = y;"#,
        )?;

        // Register `x` as a partial.
        p.register_constant(sym!("x"), term!(value!(op!(And))))?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "_this = y", "y" => "x = _this");
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "_this = y and 1 < _this", "y" => "x = _this and 1 < x");
        let next = next_binding(&mut q)?;
        // TODO: This seems wrong. Should be 2 > this and this > 1 for `y` binding.
        // Not an issue for now because it's a partial of two inputs.
        assert_partial_expressions!(next, "x" => "_this = y and 2 > _this and _this > 1", "y" => "x = _this and 2 > x and x > 1");
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expressions!(next, "x" => "_this > 1", "y" => "_this < 2");
        let next = next_binding(&mut q)?;
        assert_partial_binding!(
            next,
            "x",
            term!(op!(Unify, var!("_this"), var!("y"))),
            term!(op!(Gt, var!("_this"), term!(1))),
            term!(op!(Lt, var!("_this"), term!(2)))
        );

        assert_partial_binding!(
            next,
            "y",
            term!(op!(Unify, var!("x"), var!("_this"))),
            term!(op!(Gt, var!("x"), term!(1))),
            term!(op!(Lt, var!("x"), term!(2)))
        );

        assert_query_done!(q);

        // Register `y` as a partial.
        p.register_constant(sym!("y"), term!(value!(op!(And))))?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this = y");
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this = y and 1 < _this");
        assert_partial_expression!(next, "y", "x = _this and 1 < x");
        let next = next_binding(&mut q)?;
        assert_partial_binding!(
            next,
            "x",
            term!(op!(Unify, var!("_this"), var!("y"))),
            term!(op!(Gt, var!("_this"), term!(1))),
            term!(op!(Gt, term!(2), var!("_this")))
        );

        // TODO: This seems kind of wrong.
        assert_partial_binding!(
            next,
            "y",
            term!(op!(Unify, var!("x"), var!("_this"))),
            term!(op!(Gt, var!("x"), term!(1))),
            term!(op!(Gt, term!(2), var!("x")))
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this > 1");
        assert_partial_expression!(next, "y", "_this < 2");
        let next = next_binding(&mut q)?;
        assert_partial_expression!(next, "x", "_this > 1 and _this = y and _this < 2");
        assert_partial_expression!(next, "y", "x > 1 and x = _this and x < 2");
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

        p.clear_rules();

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

        p.clear_rules();

        p.load_str("h(x, y) if x > y and y = 1;")?;
        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x"), sym!("y")])), false);
        let next = next_binding(&mut q)?;
        assert_eq!(next[&sym!("y")], term!(1));
        assert_partial_expressions!(next, "x" => "_this > 1");
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
        p.register_constant(sym!("x"), term!(value!(op!(And))))?;
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
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 != _this.foo"); // FIXME order reversed??
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 != _this.foo"); // here too
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 != _this.foo.bar"); // and here
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("i", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "1 != _this.foo.bar"); // not here!
        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_negate_dot() -> TestResult {
        let p = Polar::new();
        p.load_str(r#"f(x) if not (x.y = 1 and x.b = 2);"#)?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "(1 != _this.y or 2 != _this.b)");
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
            "_this > 0 and _this > 1 and _this > 2 and (_this > 3 or _this > 4 or _this > 5)"
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
            r#"f(x) if not (x.foo = _y);
               g(x) if not (x.foo.bar = _y);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_eq!(
            next_binding(&mut q)?.get(&sym!("x")).unwrap(),
            &term!(op!(
                And,
                term!(op!(
                    Neq,
                    var!("__y_9"),
                    term!(op!(Dot, var!("_this"), str!("foo")))
                ))
            ))
        );
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_eq!(
            next_binding(&mut q)?.get(&sym!("x")).unwrap(),
            &term!(op!(
                And,
                term!(op!(
                    Neq,
                    var!("__y_17"),
                    term!(op!(
                        Dot,
                        term!(op!(Dot, var!("_this"), str!("foo"))),
                        str!("bar")
                    ))
                ))
            ))
        );
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
            r#"f(_);
               g(_) if false;"#,
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
            r#"f(x) if _y in x.values;
               g(x, y) if y in x.values;
               h(x) if y in x.values and (y.bar = 1 and y.baz = 2 or y.bar = 3);
               k(x) if x > 1 and x in [2, 3];
               l(x) if _y in x;
               m(x) if 1 in y and y = x;"#,
        )?;

        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "x" => "__y_12 in _this.values"
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

        let mut q = p.new_query_from_term(term!(call!("k", [sym!("x")])), false);
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(2));
        assert_eq!(next_binding(&mut q)?[&sym!("x")], term!(3));
        assert_query_done!(q);

        let mut q = p.new_query_from_term(term!(call!("l", [sym!("x")])), false);
        assert_partial_expressions!(next_binding(&mut q)?, "x" => "__y_36 in _this");
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
        p.load_str("f(_) if cut;")?;
        p.register_constant(sym!("x"), op!(And).into())?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let error = q.next_event().unwrap_err();
        assert!(matches!(
            error.0,
            ErrorKind::Runtime(RuntimeError::Unsupported { .. }),
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
            r#"f(x) if x > 1 and (cut or x = 2) and x = 3;
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
            r#"f(x, _) if cut and x = 1;
               f(x, _: 2) if x = 2;"#,
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
            r#"f(y) if (not y matches Foo{} or not g(y)) and y == 1;
               g(_: Bar);"#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expression!(nextb!(&mut q), "x", "not 1 matches Foo{}");
        assert_partial_expression!(nextb!(&mut q), "x", "not 1 matches Bar{}");
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
        p.load_str(r#"f() if _x = 1;"#)?;

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

    #[test]
    fn test_querying_for_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            "f(x) if x.foo;
        g(x) if not x.foo;
        h(x) if x;
        a(x) if f(x);
        b(x) if f(x.bar);",
        )?;

        // does f(x) call with x unbound turn into an equality constraint to true on the field?
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        let r = next_binding(&mut q)?;
        assert_partial_expression!(r, "x", "true = _this.foo");
        assert_query_done!(q);

        // does g(x) call with x unbound turn into an inequality constraint on the field?
        let mut q = p.new_query_from_term(term!(call!("g", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "true != _this.foo");
        assert_query_done!(q);

        // does h(x) call with x unbound turn into an equality constraint to true on x?
        let mut q = p.new_query_from_term(term!(call!("h", [sym!("x")])), false);
        let left = next_binding(&mut q)?;
        let right = hashmap! {
            sym!("x") => term!(true)
        };
        assert_eq!(left, right);
        //   assert_partial_expression!(next_binding(&mut q)?, "x", "true");
        assert_query_done!(q);

        // does a(x) call with x unbound turn into constraining x.foo to true?
        let mut q = p.new_query_from_term(term!(call!("a", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "true = _this.foo");
        assert_query_done!(q);

        // does b(x) call with x unbound turn into constraining b.bar.foo to true?
        let mut q = p.new_query_from_term(term!(call!("b", [sym!("x")])), false);
        assert_partial_expression!(next_binding(&mut q)?, "x", "true = _this.bar.foo");
        assert_query_done!(q);

        Ok(())
    }

    // Grounding tests
    fn test_grounding<T>(rules: &str, query: Term, assert_fns: T) -> TestResult
    where
        T: IntoIterator,
        T::Item: Fn(Bindings) -> TestResult,
    {
        let p = Polar::new();
        p.load_str(rules)?;

        let mut q = p.new_query_from_term(query, false);
        for assert_fn in assert_fns.into_iter() {
            let r = next_binding(&mut q)?;
            assert_fn(r)?;
        }

        assert_query_done!(q);

        Ok(())
    }

    #[test]
    fn test_grounding_1() -> TestResult {
        test_grounding(
            "f(x) if x in y and x > 0 and y = [1, 2, 3] and x = 1;",
            term!(call!("f", [sym!("x")])),
            &[|r: Bindings| {
                assert_eq!(r.get(&sym!("x")).unwrap(), &term!(1));
                Ok(())
            }],
        )
    }

    #[test]
    fn test_grounding_2() -> TestResult {
        test_grounding(
            r#"
            f(x, y) if x > y and x = 1;
        "#,
            term!(call!("f", [sym!("x"), sym!("y")])),
            &[|r: Bindings| {
                assert_eq!(r.get(&sym!("x")).unwrap(), &term!(1));
                assert_partial_expression!(r, "y", "1 > _this");
                Ok(())
            }],
        )
    }

    #[test]
    fn test_grounding_not_2() -> TestResult {
        test_grounding(
            r#"
            f(x) if x > 0 and not (x > 5 and x = 3);
        "#,
            term!(call!("f", [sym!("x")])),
            &[|r: Bindings| {
                // x > 5 and x = 3 is always false so negation always succeeds.
                assert_partial_expression!(r, "x", "_this > 0");
                Ok(())
            }],
        )
    }

    #[test]
    fn test_grounding_not_3() -> TestResult {
        test_grounding(
            r#"
            f(x) if x > 0 and not (x >= 1 and x = 1);
        "#,
            term!(call!("f", [sym!("x")])),
            &[|r: Bindings| {
                // x >= 1 and x = 1 are compatible, so the negation binds x to 1
                // if x is 1 the negated query succeeds, failing overall query
                assert_partial_expression!(r, "x", "_this > 0 and _this != 1");
                Ok(())
            }],
        )
    }

    // THIS IS the manually rewritten version of test_grounding_not_4.
    // Considering whether we can just do this rewrite to execute inversion.
    #[test]
    fn test_grounding_not_rewrite_4() -> TestResult {
        test_grounding(
            r#"
            f(x, y) if x > 0 and (x < 1 or x != 1 or y <= x);
        "#,
            term!(call!("f", [sym!("x"), sym!("y")])),
            &[
                |r: Bindings| {
                    assert_partial_expression!(r, "x", "_this > 0 and _this < 1");
                    assert_eq!(r.get(&sym!("y")).unwrap(), &term!(sym!("y")));
                    Ok(())
                },
                |r: Bindings| {
                    assert_partial_expression!(r, "x", "_this > 0 and _this != 1");
                    assert_eq!(r.get(&sym!("y")).unwrap(), &term!(sym!("y")));
                    Ok(())
                },
                |r: Bindings| {
                    assert_partial_expressions!(r,
                        "x" => "_this > 0 and y <= _this",
                        "y" => "x > 0 and _this <= x"
                    );
                    Ok(())
                },
            ],
        )
    }

    #[test]
    fn test_grounding_not_5() -> TestResult {
        test_grounding(
            r#"
            f(x, y) if x > 0 and not (x >= 1 and x = 1 and y > x and g(x, y));
            g(x, _) if x >= 3;
            g(1, y) if y >= 3 and y > 5;
        "#,
            term!(call!("f", [sym!("x"), sym!("y")])),
            &[|r: Bindings| {
                assert_partial_expressions!(r,
                    "x" => "_this > 0 and _this != 1",
                    // Right now we output "y" => "_this <= 1".
                    // This constraint is incorrect. If x = 1, then y <= 1
                    // (we reach the y > x constraint in the
                    // negation). Otherwise, the query succeeds because x = 1 fails
                    // and y > x is never reached.
                    "y" => "(_this <= 1 or _this < 3 or _this <= 5)"
                );
                Ok(())
            }],
        )
    }

    /* FIXME
    #[test]
    fn test_grounding_not_1() -> TestResult {
        test_grounding(
            r#"
            f(x, y, z) if x > y and not (z < y and z = 1);
            "#,
            term!(call!("f", [sym!("x"), sym!("y"), sym!("z")])),
            &[|r: Bindings| {
                assert_partial_expression!(r, "x", "_this > y and 1 >= y");
                // Expected: x > _this and (z >= _this or z != 1)
                assert_partial_expression!(r, "y", "x > _this and z >= _this"); // 1 has been substituted for z which is incorrect.
                                                                                // Expected: z >= _this or z != 1
                assert_partial_expression!(r, "z", "_this != 1"); // MISSING or z >= y
                Ok(())
            }],
        )
    }

    #[test]
    fn test_grounding_not_4() -> TestResult {
        test_grounding(
            r#"
            f(x, y) if x > 0 and not (x >= 1 and x = 1 and y > x);
        "#,
            term!(call!("f", [sym!("x"), sym!("y")])),
            &[|r: Bindings| {
                assert_partial_expressions!(r,
                    "x" => "_this > 0 and _this != 1",
                    // Right now we output "y" => "_this <= 1".
                    // This constraint is incorrect. If x = 1, then y <= 1
                    // (we reach the y > x constraint in the
                    // negation). Otherwise, the query succeeds because x = 1 fails
                    // and y > x is never reached.
                    "y" => "x != 1 or _this <= 1"
                );
                Ok(())
            }],
        )
    }

    #[test]
    fn test_grounding_not_4_eq() -> TestResult {
        test_grounding(
            r#"
            f(x, y) if x > 0 and not (x >= 1 and x == 1 and y > x);
        "#,
            term!(call!("f", [sym!("x"), sym!("y")])),
            &[|r: Bindings| {
                assert_partial_expressions!(r,
                    "x" => "_this > 0",
                    // Right now we output "y" => "1 < 1 or _this <= 1".
                    // This constraint is incorrect. If x = 1, then y <= 1
                    // (we reach the y > x constraint in the
                    // negation). Otherwise, the query succeeds because x = 1 fails
                    // and y > x is never reached.
                    "y" => "x != 1 or _this <= 1"
                );
                Ok(())
            }],
        )
    }

    #[test]
    fn test_grounding_not_5_eq() -> TestResult {
        test_grounding(
            r#"
            f(x, y) if x > 0 and not (x >= 1 and x == 1 and y > x and g(x, y));
            g(x, _) if x >= 3;
            g(1, y) if y >= 3 and y > 5;
        "#,
            term!(call!("f", [sym!("x"), sym!("y")])),
            &[|r: Bindings| {
                assert_partial_expressions!(r,
                    "x" => "_this > 0 and 1 < 1 or _1_11 <= 1 or 1 < 3 and _this != 1",
                    // Right now we output "y" => "_this <= 1".
                    // This constraint is incorrect. If x = 1, then y <= 1
                    // (we reach the y > x constraint in the
                    // negation). Otherwise, the query succeeds because x = 1 fails
                    // and y > x is never reached.
                    "y" => "1 < 1 or _this <= 1 or 1 < 3 and _this < 3 or _this <= 5"
                );
                Ok(())
            }],
        )
    }
    */

    #[test]
    fn test_grounding_not_5_rewrite() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"
            f(x, y) if x > 0 and not (x >= 1 and x = 1 and y > x and g(x, y));
            g(x, _) if x >= 3;
            g(1, y) if y >= 3 and y > 5;
        "#,
        )?;

        let p_rewrite = Polar::new();
        p_rewrite.load_str(
            r#"
            # f(x, y) if x > 0 and (x < 1 or x != 1 or y <= x or not g(x, y));
            f(x, y) if x > 0 and (x < 1 or x != 1 or y <= x or (g_1_not(x, y) and g_2_not(x, y)));
            g_1_not(x, _) if x < 3;
            g_2_not(1, y) if y < 3 or y <= 5;
        "#,
        )?;

        let xs = (-10..10).collect::<Vec<_>>();
        let ys = (-10..10).collect::<Vec<_>>();

        for x in xs.iter() {
            for y in ys.iter() {
                let mut p_query = p.new_query_from_term(term!(call!("f", [*x, *y])), false);
                let mut p_rewrite_query =
                    p_rewrite.new_query_from_term(term!(call!("f", [*x, *y])), false);

                let p_has_next = matches!(p_query.next_event()?, QueryEvent::Result { .. });
                let p_rewrite_has_next =
                    matches!(p_rewrite_query.next_event()?, QueryEvent::Result { .. });
                assert_eq!(p_has_next, p_rewrite_has_next)
            }
        }

        Ok(())
    }

    #[test]
    fn test_grounded_negated_dot_comparison() -> TestResult {
        let p = Polar::new();

        let mut q = p.new_query("x in _ and not x.a = q and x = {}", false)?;
        assert_query_done!(q);

        let mut q = p.new_query("x in _ and not q = x.a and x = {}", false)?;
        assert_query_done!(q);

        let mut q = p.new_query("x in _ and not q = x.a and x = {a: q}", false)?;
        let bindings = next_binding(&mut q)?;
        let expd = term!(Value::Dictionary(dict!(
            btreemap! { sym!("a") => var!("q") }
        )));
        assert_eq!(bindings.get(&sym!("x")).unwrap(), &expd);

        let mut q = p.new_query("x in _ and not x.a = q and x = {a: q}", false)?;
        let bindings = next_binding(&mut q)?;
        let expd = term!(Value::Dictionary(dict!(
            btreemap! { sym!("a") => var!("q") }
        )));
        assert_eq!(bindings.get(&sym!("x")).unwrap(), &expd);
        Ok(())
    }

    // Tests that if a policy constructs any partials that aren't tied to
    // the result variables, that we will get an error
    #[test]
    fn test_for_unhandled_partial() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"
            # All of these should error
            f(x) if y = y and y > 0 and x = 1;
            g() if x = x and y = y and x in y;
            h() if x = x and x > 0;
            i() if x.a = 1 and x.b = 2;
            j() if x.a = 1 and x.a = 2;

            # Cases that look similar but work
            a(x) if y.foo = x and y.bar = 1;
            b() if _x_dot_a = 1 and _x_dot_b = 2;
        "#,
        )?;

        // all the failing cases
        let query_terms = vec![
            term!(call!("f", [sym!("x")])),
            term!(call!("g", [])),
            term!(call!("h", [])),
            term!(call!("i", [])),
            term!(call!("j", [])),
        ];
        for query in query_terms {
            let mut q = p.new_query_from_term(query.clone(), false);
            let res = q.next_event();
            assert!(
                matches!(
                    res,
                    Err(PolarError(ErrorKind::Runtime(
                        RuntimeError::UnhandledPartial { .. }
                    )))
                ),
                "unexpected result: {:#?} for {}",
                res,
                query
            );
        }

        // successful cases
        let mut q = p.new_query_from_term(term!(call!("a", [sym!("x")])), false);
        // well, this is semi-successful!
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "x" => "_this = _y_26.foo and 1 = _y_26.bar"
        );

        assert_query_done!(q);
        Ok(())
    }

    /// Test a case that previously caused an unhandled partial.
    ///
    /// Fixed in: https://github.com/osohq/oso/pull/1467
    #[test]
    fn test_unhandled_partial_regression_gh1467() -> TestResult {
        let p = Polar::new();
        p.load_str(
            r#"
            f(a) if b(a, b) and b.id = 0;
            b(a, b) if a = b;
            "#,
        )?;
        let mut q = p.new_query_from_term(term!(call!("f", [sym!("x")])), false);
        assert_partial_expressions!(
            next_binding(&mut q)?,
            "x" => "0 = _this.id"
        );

        assert_query_done!(q);
        Ok(())
    }
}
