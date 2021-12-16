use crate::terms::*;

impl Term {
    /// intelligently negate an expression
    fn negated(self) -> Self {
        use {Operator::*, Value::*};
        match self.value() {
            // negate booleans
            Boolean(b) => self.clone_with_value(Boolean(!b)),

            Expression(Operation { operator, args }) => match operator {
                // cancel double negatives
                Not => self.clone_with_value(args[0].value().clone()),

                // swap and/or using De Morgan's to keep the nots nested
                // as deep as possible
                And => {
                    assert_eq!(args.len(), 2);
                    let args = vec![args[0].clone().negated(), args[1].clone().negated()];
                    self.clone_with_value(Expression(Operation { operator: Or, args }))
                }
                Or => {
                    assert_eq!(args.len(), 2);
                    let args = vec![args[0].clone().negated(), args[1].clone().negated()];
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

    /// binary tree normal form -- all and / or nodes form a binary tree
    fn btnf(self) -> Self {
        use Operator::*;
        match self.value().as_expression() {
            Ok(Operation { operator, args }) if *operator == And || *operator == Or => {
                match args.len() {
                    // empty -> boolean
                    0 => self.clone_with_value(Value::Boolean(*operator == And)),
                    // one -> lift it out
                    1 => args[0].clone().btnf(),
                    // else fold
                    _ => args
                        .iter()
                        .map(|a| a.clone().btnf())
                        .reduce(|m, x| {
                            self.clone_with_value(Value::Expression(Operation {
                                operator: *operator,
                                args: vec![m, x],
                            }))
                        })
                        .unwrap(),
                }
            }
            _ => self,
        }
    }

    /// negation normal form -- all not operators are nested inside of and/or
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

    fn lhs(&self) -> &Self {
        &self.value().as_expression().unwrap().args[0]
    }

    fn rhs(&self) -> &Self {
        &self.value().as_expression().unwrap().args[1]
    }

    /// normalize an expression by fully distributing logical connectives. the
    /// expression must already be in btnf and nnf. the arguments are two
    /// predicate / constructor pairs:
    /// - p1/c1: predicate/constructor for *inner* connective (and for dnf, or for cnf)
    /// - p2/c2: predicate/constructor for *outer* connective (or for dnf, and for cnf)
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

    /// Condition input for dnf/cnf transformation
    /// - negations fully nested
    /// - double negatives removed
    /// - and/or nodes form a binary tree
    fn pre_norm(self) -> Self {
        self.btnf().nnf()
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
