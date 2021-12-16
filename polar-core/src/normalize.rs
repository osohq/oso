use crate::terms::*;

impl Term {
    fn negated(mut self) -> Self {
        use {Operator::*, Value::*};
        match self.value() {
            Expression(Operation { operator, args }) => match operator {
                Not => {
                    assert_eq!(args.len(), 1);
                    let val = args[0].value().clone();
                    self.replace_value(val)
                }
                And => {
                    assert_eq!(args.len(), 2);
                    let args = vec![args[0].clone().negated(), args[1].clone().negated()];
                    self.replace_value(Expression(Operation { operator: Or, args }))
                }
                Or => {
                    assert_eq!(args.len(), 2);
                    let args = vec![args[0].clone().negated(), args[1].clone().negated()];
                    self.replace_value(Expression(Operation {
                        operator: And,
                        args,
                    }))
                }
                _ => self.replace_value(Expression(op!(Not, self.clone()))),
            },
            Boolean(b) => {
                let b = !b;
                self.replace_value(Boolean(!b))
            }
            _ => self.replace_value(Expression(op!(Not, self.clone()))),
        }
        self
    }

    /// Condition input for dnf/cnf transformation
    /// - negations fully nested
    /// - double negatives removed
    /// - and/or nodes form a binary tree
    fn pre_norm(self) -> Self {
        impl Term {
            /// binary normal form -- all `and`/`or` operations have args.len() == 2
            fn binf(self) -> Self {
                use Operator::*;
                match self.value().as_expression() {
                    Ok(Operation { operator, args })
                        if args.len() == 1 && (*operator == And || *operator == Or) =>
                    {
                        args[0].clone().pre_norm()
                    }
                    Ok(Operation { operator, args }) if *operator == And || *operator == Or => args
                        .iter()
                        .map(|a| a.clone().pre_norm())
                        .reduce(|m, x| {
                            self.clone_with_value(Value::Expression(Operation {
                                operator: *operator,
                                args: vec![m, x],
                            }))
                        })
                        .unwrap(),
                    _ => self,
                }
            }

            /// negation normal form -- all `not` operators are nested inside of `and`/`or`
            /// using De Morgan's law
            fn nnf(self) -> Self {
                use Operator::*;
                match self.value().as_expression() {
                    Ok(Operation {
                        operator: Not,
                        args,
                    }) => args[0].clone().nnf().negated(),
                    Ok(Operation { operator, args }) => {
                        self.clone_with_value(Value::Expression(Operation {
                            operator: *operator,
                            args: args.iter().cloned().map(|t| t.nnf()).collect(),
                        }))
                    }
                    _ => self,
                }
            }
        }
        self.binf().nnf()
    }

    fn lhs(&self) -> &Self {
        &self.value().as_expression().unwrap().args[0]
    }

    fn rhs(&self) -> &Self {
        &self.value().as_expression().unwrap().args[1]
    }

    fn distribute(
        mut self,
        p1: fn(&Term) -> bool,
        c1: fn(Term, Term) -> Term,
        p2: fn(&Term) -> bool,
        c2: fn(Term, Term) -> Term,
    ) -> Self {
        use Value::*;
        match self.value().as_expression() {
            Ok(Operation { operator, args }) => {
                let args: Vec<_> = args
                    .iter()
                    .cloned()
                    .map(|t| t.distribute(p1, c1, p2, c2))
                    .collect();
                let operator = *operator;
                if p1(&self) && p2(&args[0]) {
                    c2(
                        c1(args[0].lhs().clone(), args[1].clone()),
                        c1(args[0].rhs().clone(), args[1].clone()),
                    )
                    .distribute(p1, c1, p2, c2)
                } else if p1(&self) && p2(&args[1]) {
                    c2(
                        c1(args[0].clone(), args[1].lhs().clone()),
                        c1(args[0].clone(), args[1].rhs().clone()),
                    )
                    .distribute(p1, c1, p2, c2)
                } else {
                    self.replace_value(Expression(Operation { operator, args }));
                    self
                }
            }
            _ => self,
        }
    }

    pub fn dnf(self) -> Self {
        self.pre_norm().distribute(andp, and_, orp, or_)
    }

    pub fn cnf(self) -> Self {
        self.pre_norm().distribute(orp, or_, andp, and_)
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
fn andp(l: &Term) -> bool {
    matches!(
        l.value().as_expression(),
        Ok(Operation {
            operator: Operator::And,
            ..
        })
    )
}
fn orp(l: &Term) -> bool {
    matches!(
        l.value().as_expression(),
        Ok(Operation {
            operator: Operator::Or,
            ..
        })
    )
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nnf() {
        let ex = or_(
            not_(var!("p")),
            and_(var!("q"), not_(and_(not_(var!("r")), var!("s")))),
        );

        assert_eq!(ex.clone().nnf(), ex.clone().nnf().nnf());

        let nnf = or_(
            not_(var!("p")),
            and_(var!("q"), or_(var!("r"), not_(var!("s")))),
        );

        assert_eq!(nnf, ex.nnf());
    }

    #[test]
    fn test_dnf() {
        let ex = or_(
            not_(var!("p")),
            and_(var!("q"), not_(and_(not_(var!("r")), var!("s")))),
        );

        assert_eq!(ex.clone().dnf(), ex.clone().dnf().dnf());

        let dnf = or_(
            not_(var!("p")),
            or_(and_(var!("q"), var!("r")), and_(var!("q"), not_(var!("s")))),
        );

        assert_eq!(dnf, ex.dnf());
    }

    #[test]
    fn test_cnf() {
        let ex = or_(
            not_(var!("p")),
            and_(var!("q"), not_(and_(not_(var!("r")), var!("s")))),
        );

        assert_eq!(ex.clone().cnf(), ex.clone().cnf().cnf());

        let cnf = and_(
            or_(not_(var!("p")), var!("q")),
            or_(not_(var!("p")), or_(var!("r"), not_(var!("s")))),
        );

        assert_eq!(cnf.to_polar(), ex.cnf().to_polar());
    }
}
