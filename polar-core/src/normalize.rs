use crate::terms::*;

impl Term {
    /// convert expression to disjunctive normal form
    pub fn disjunctive_normal_form(&self) -> Self {
        self.pre_normalize().distribute(is_and, and_, is_or, or_)
    }

    /// convert expression to conjunctive normal form
    pub fn conjunctive_normal_form(&self) -> Self {
        self.pre_normalize().distribute(is_or, or_, is_and, and_)
    }

    /// Condition input for dnf/cnf transformation
    /// - negations fully nested
    /// - double negatives removed
    /// - and/or nodes form a binary tree
    fn pre_normalize(&self) -> Self {
        self.as_binary_tree().negation_normal_form()
    }

    /// normalize an expression by fully distributing logical connectives. the
    /// expression must already be in btnf and nnf. the arguments are two
    /// predicate / constructor pairs:
    /// - p1/c1: predicate/constructor for *inner* connective (and for dnf, or for cnf)
    /// - p2/c2: predicate/constructor for *outer* connective (or for dnf, and for cnf)
    fn distribute(
        &self,
        p1: fn(&Term) -> bool,
        c1: fn(Term, Term) -> Term,
        p2: fn(&Term) -> bool,
        c2: fn(Term, Term) -> Term,
    ) -> Self {
        use Value::*;

        match self.as_expression() {
            Err(_) => self.clone(),
            Ok(Operation { operator, args }) => {
                let args: Vec<_> = args.iter().map(|t| t.distribute(p1, c1, p2, c2)).collect();

                if p1(self) && p2(&args[0]) {
                    c2(
                        c1(args[0].lhs(), args[1].clone()),
                        c1(args[0].rhs(), args[1].clone()),
                    )
                    .distribute(p1, c1, p2, c2)
                } else if p1(self) && p2(&args[1]) {
                    c2(
                        c1(args[0].clone(), args[1].lhs()),
                        c1(args[0].clone(), args[1].rhs()),
                    )
                    .distribute(p1, c1, p2, c2)
                } else {
                    self.clone_with_value(Expression(Operation {
                        operator: *operator,
                        args,
                    }))
                }
            }
        }
    }

    /// binary tree normal form -- all and / or nodes form a binary tree
    fn as_binary_tree(&self) -> Self {
        use Operator::*;
        match self.as_expression() {
            Ok(Operation { operator, args }) if *operator == And || *operator == Or => {
                match args.len() {
                    // empty -> boolean
                    0 => self.clone_with_value(Value::Boolean(*operator == And)),
                    // one -> lift it out
                    1 => args[0].as_binary_tree(),
                    // else fold
                    _ => args
                        .iter()
                        .map(|a| a.as_binary_tree())
                        .reduce(|m, x| {
                            self.clone_with_value(Value::Expression(Operation {
                                operator: *operator,
                                args: vec![m, x],
                            }))
                        })
                        .unwrap(),
                }
            }
            _ => self.clone(),
        }
    }

    /// negation normal form -- all not operators are nested inside of and/or
    /// using De Morgan's law. must already be in btnf.
    fn negation_normal_form(&self) -> Self {
        use Operator::*;
        match self.as_expression() {
            Err(_) => self.clone(),
            Ok(Operation {
                operator: Not,
                args,
            }) => args[0].negation_normal_form().negated(),
            Ok(Operation { operator, args }) => {
                self.clone_with_value(Value::Expression(Operation {
                    operator: *operator,
                    args: args
                        .iter()
                        .cloned()
                        .map(|t| t.negation_normal_form())
                        .collect(),
                }))
            }
        }
    }

    /// negate an expression, inverting connectives to keep `not`s nested
    /// inside
    fn negated(&self) -> Self {
        use {Operator::*, Value::*};
        match self.value() {
            // negate booleans
            Boolean(b) => self.clone_with_value(Boolean(!b)),

            Expression(Operation { operator, args }) => match operator {
                // cancel double negatives
                Not => self.clone_with_value(args[0].value().clone()),

                // swap and/or
                And => {
                    assert_eq!(args.len(), 2);
                    let args = vec![args[0].negated(), args[1].negated()];
                    self.clone_with_value(Expression(Operation { operator: Or, args }))
                }
                Or => {
                    assert_eq!(args.len(), 2);
                    let args = vec![args[0].negated(), args[1].negated()];
                    self.clone_with_value(Expression(Operation {
                        operator: And,
                        args,
                    }))
                }
                _ => self.clone_with_value(Expression(op!(Not, self.clone()))),
            },
            _ => self.clone_with_value(Expression(op!(Not, self.clone()))),
        }
    }

    fn _hs(&self, n: usize) -> Self {
        self.as_expression().unwrap().args[n].clone()
    }

    fn lhs(&self) -> Self {
        self._hs(0)
    }
    fn rhs(&self) -> Self {
        self._hs(1)
    }
}

pub fn or_(l: Term, r: Term) -> Term {
    term!(op!(Or, l, r))
}

pub fn and_(l: Term, r: Term) -> Term {
    term!(op!(And, l, r))
}

pub fn not_(t: Term) -> Term {
    term!(op!(Not, t))
}

fn is_op(l: &Term, op: Operator) -> bool {
    matches!(l.as_expression(),
        Ok(Operation { operator, .. }) if op == *operator)
}

fn is_and(l: &Term) -> bool {
    is_op(l, Operator::And)
}

fn is_or(l: &Term) -> bool {
    is_op(l, Operator::Or)
}

#[cfg(test)]
mod test {
    use super::*;

    fn ex1() -> Term {
        or_(
            not_(var!("p")),
            and_(var!("q"), not_(and_(not_(var!("r")), var!("s")))),
        )
    }

    fn ex2() -> Term {
        or_(
            and_(var!("q"), var!("r")),
            and_(not_(var!("q")), or_(var!("a"), var!("b"))),
        )
    }

    fn ex3() -> Term {
        and_(
            or_(var!("a"), var!("b")),
            and_(
                or_(var!("c"), var!("d")),
                and_(or_(var!("e"), var!("f")), or_(var!("g"), var!("h"))),
            ),
        )
    }

    #[test]
    fn test_pre_normalize() {
        let ex = ex1();
        let nnf = or_(
            not_(var!("p")),
            and_(var!("q"), or_(var!("r"), not_(var!("s")))),
        );

        assert_eq!(ex.pre_normalize(), ex.pre_normalize().pre_normalize()); // should be idempotent
        assert_eq!(nnf, ex.pre_normalize());
    }

    #[test]
    fn test_cnf() {
        let ex = ex1();
        let cnf = and_(
            or_(not_(var!("p")), var!("q")),
            or_(not_(var!("p")), or_(var!("r"), not_(var!("s")))),
        );

        assert_eq!(
            ex.conjunctive_normal_form(),
            ex.conjunctive_normal_form().conjunctive_normal_form()
        );
        assert_eq!(cnf, ex.conjunctive_normal_form());

        let ex = ex2();
        let cnf = and_(
            and_(
                or_(var!("q"), not_(var!("q"))),
                or_(var!("q"), or_(var!("a"), var!("b"))),
            ),
            and_(
                or_(var!("r"), not_(var!("q"))),
                or_(var!("r"), or_(var!("a"), var!("b"))),
            ),
        );

        assert_eq!(
            ex.conjunctive_normal_form(),
            ex.conjunctive_normal_form().conjunctive_normal_form()
        );
        assert_eq!(cnf, ex.conjunctive_normal_form());
    }

    #[test]
    fn test_dnf() {
        let ex = ex1();
        let dnf = or_(
            not_(var!("p")),
            or_(and_(var!("q"), var!("r")), and_(var!("q"), not_(var!("s")))),
        );

        assert_eq!(
            ex.disjunctive_normal_form(),
            ex.disjunctive_normal_form().disjunctive_normal_form()
        );
        assert_eq!(dnf, ex.disjunctive_normal_form());

        let ex = ex2();
        let dnf = or_(
            and_(var!("q"), var!("r")),
            or_(
                and_(not_(var!("q")), var!("a")),
                and_(not_(var!("q")), var!("b")),
            ),
        );

        assert_eq!(
            ex.disjunctive_normal_form(),
            ex.disjunctive_normal_form().disjunctive_normal_form()
        );
        assert_eq!(dnf, ex.disjunctive_normal_form());

        let ex = ex3();
        let dnf = or_(
            or_(
                or_(
                    or_(
                        and_(var!("a"), and_(var!("c"), and_(var!("e"), var!("g")))),
                        and_(var!("a"), and_(var!("c"), and_(var!("e"), var!("h")))),
                    ),
                    or_(
                        and_(var!("a"), and_(var!("c"), and_(var!("f"), var!("g")))),
                        and_(var!("a"), and_(var!("c"), and_(var!("f"), var!("h")))),
                    ),
                ),
                or_(
                    or_(
                        and_(var!("a"), and_(var!("d"), and_(var!("e"), var!("g")))),
                        and_(var!("a"), and_(var!("d"), and_(var!("e"), var!("h")))),
                    ),
                    or_(
                        and_(var!("a"), and_(var!("d"), and_(var!("f"), var!("g")))),
                        and_(var!("a"), and_(var!("d"), and_(var!("f"), var!("h")))),
                    ),
                ),
            ),
            or_(
                or_(
                    or_(
                        and_(var!("b"), and_(var!("c"), and_(var!("e"), var!("g")))),
                        and_(var!("b"), and_(var!("c"), and_(var!("e"), var!("h")))),
                    ),
                    or_(
                        and_(var!("b"), and_(var!("c"), and_(var!("f"), var!("g")))),
                        and_(var!("b"), and_(var!("c"), and_(var!("f"), var!("h")))),
                    ),
                ),
                or_(
                    or_(
                        and_(var!("b"), and_(var!("d"), and_(var!("e"), var!("g")))),
                        and_(var!("b"), and_(var!("d"), and_(var!("e"), var!("h")))),
                    ),
                    or_(
                        and_(var!("b"), and_(var!("d"), and_(var!("f"), var!("g")))),
                        and_(var!("b"), and_(var!("d"), and_(var!("f"), var!("h")))),
                    ),
                ),
            ),
        );

        assert_eq!(
            ex.disjunctive_normal_form(),
            ex.disjunctive_normal_form().disjunctive_normal_form()
        );
        assert_eq!(dnf, ex.disjunctive_normal_form());
    }
}
