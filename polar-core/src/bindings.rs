/// Manage binding state in the VM.
///
/// Bindings associate variables in the VM with constraints or values.
use std::collections::{HashMap, HashSet};

use crate::error::PolarResult;
use crate::folder::{fold_term, Folder};
use crate::formatting::ToPolarString;
use crate::terms::{has_rest_var, Operation, Operator, Symbol, Term, Value};
use crate::vm::cycle_constraints;

#[derive(Clone, Debug)]
pub struct Binding(pub Symbol, pub Term);

// TODO This is only public for debugger and inverter.
// Eventually this should be an internal interface.
pub type BindingStack = Vec<Binding>;
pub type Bindings = HashMap<Symbol, Term>;

pub type Bsp = usize;
pub type FollowerId = usize;

/// Variable binding state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariableState {
    Unbound,
    Bound(Term),

    // NOTE (dhatch): The simplifier only cares about variables that are bound
    // together if the variable is constrained. If the variable is still in the
    // Cycle state, the simplifier does nothing.
    Cycle(Vec<Symbol>),
    Partial(Operation),
}

#[derive(Clone, Debug, Default)]
/// The binding manager is responsible for managing binding & constraint state.
/// It is updated primarily using:
/// - `bind`
/// - `add_constraint`
///
/// Bindings are retrived with:
/// - `deref`
/// - `value`
/// - `variable_state`
/// - `bindings`
pub struct BindingManager {
    bindings: BindingStack,
    followers: HashMap<FollowerId, BindingManager>,

    /// Track the bsp of followers when they were added so they can be
    /// backtracked.
    follower_bsps: HashMap<FollowerId, Bsp>,
    next_follower_id: usize,
}

/// The `BindingManager` maintains associations between variables and values,
/// and constraints.
///
/// A variable may be:
/// - unbound
/// - bound
/// - constrained
///
/// Variables may also be bound together such that their values or constraints
/// will be the same.
///
/// A binding is created with the `bind` method.
///
/// The constraints or value associated with a variable is retrieved with `variable_state`.
impl BindingManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind `var` to `val`.
    ///
    ///
    /// If `var` is already bound or constrained, the
    /// binding or constraints are replaced with `val`.
    pub fn bind(&mut self, var: &Symbol, val: Term) {
        self.do_followers(|_, follower| {
            follower.bind(var, val.clone());
            Ok(())
        }).unwrap();

        // TODO (dhatch): Would like to disable rebinding, but this has a large fallout.
        // We use it extensively for testing and in external_question_result to give the result
        // variable a default value (we could probably fix this some other way).
        // If we don't disable rebinding, we need to do something with the rebind_variable_group
        // test so that the behavior is better defined.
        // assert!(!matches!(self.variable_state(var), VariableState::Bound(_)), "Variable is bound");

        if let Ok(symbol) = val.value().as_symbol() {
            self.bind_variables(var, symbol);
        } else {
            if let VariableState::Partial(p) = self.variable_state(var) {
                if let Some(grounded) = p.ground(var.clone(), val.clone()) {
                    self.add_binding(var, val);
                    // TODO (dhatch): Return error.
                    self.constrain(&grounded).unwrap();
                } else {
                    println!("ground failed for {}", p.to_polar());
                    panic!("check ground precondition");
                    // TODO (dhatch): Return error.
                }
            } else {
                self.add_binding(var, val);
            }
        }
    }

    /// Bind two variables together.
    fn bind_variables(&mut self, left: &Symbol, right: &Symbol) {
        match (self.variable_state(left), self.variable_state(right)) {
            (VariableState::Bound(_), VariableState::Unbound) => {
                // Replace binding.
                self.add_binding(left, term!(right.clone()));
            }
            (VariableState::Unbound, VariableState::Bound(_)) => {
                // Bind variables in cycle.
                if left != right {
                    self.add_binding(left, term!(right.clone()));
                    self.add_binding(right, term!(left.clone()));
                }
            }

            // Cycles: one or more variables are bound together.
            (VariableState::Unbound, VariableState::Unbound) => {
                // Both variables are unbound. Bind them in a new cycle,
                // but do not create 1-cycles.
                if left != right {
                    self.add_binding(left, term!(right.clone()));
                    self.add_binding(right, term!(left.clone()));
                }
            }
            (VariableState::Cycle(cycle), VariableState::Unbound) => {
                // Left is in a cycle. Extend it to include right.
                let last = cycle.last().unwrap();
                assert_ne!(last, left);
                self.add_binding(last, term!(right.clone()));
                self.add_binding(right, term!(left.clone()));
            }
            (VariableState::Unbound, VariableState::Cycle(cycle)) => {
                // Right is in a cycle. Extend it to include left.
                let last = cycle.last().unwrap();
                assert_ne!(last, right);
                self.add_binding(last, term!(left.clone()));
                self.add_binding(left, term!(right.clone()));
            }
            (VariableState::Cycle(left_cycle), VariableState::Cycle(right_cycle)) => {
                // Both variables are in cycles.
                let iter_left = left_cycle.iter().collect::<HashSet<&Symbol>>();
                let iter_right = right_cycle.iter().collect::<HashSet<&Symbol>>();
                if iter_left.intersection(&iter_right).next().is_some() {
                    // The cycles must be the same. Do nothing.
                    assert_eq!(iter_left, iter_right);
                } else {
                    // Join the two cycles.
                    let last_left = left_cycle.last().unwrap();
                    let last_right = right_cycle.last().unwrap();
                    assert_ne!(last_left, left);
                    assert_ne!(last_right, right);
                    self.add_binding(last_left, term!(right.clone()));
                    self.add_binding(last_right, term!(left.clone()));
                }
            }
            (VariableState::Cycle(_), VariableState::Bound(right_value)) => {
                // Ground out the cycle.
                self.add_binding(left, right_value);
            }
            (VariableState::Bound(_), VariableState::Cycle(cycle)) => {
                // Left is currently bound. Instead, rebind it by adding it to
                // the cycle.
                let last = cycle.last().unwrap();
                assert_ne!(last, right);
                self.add_binding(last, term!(left.clone()));
                self.add_binding(left, term!(right.clone()));
            }
            (VariableState::Bound(_), VariableState::Bound(right_value)) => {
                self.add_binding(left, right_value);
            }
            (VariableState::Bound(left_value), VariableState::Partial(_)) => {
                // Left is bound, right has constraints.
                // TODO (dhatch): No unwrap.
                self.add_constraint(&op!(Unify, left_value, term!(right.clone())).into_term())
                    .unwrap();
            }
            (VariableState::Partial(_), VariableState::Bound(right_value)) => {
                self.add_constraint(&op!(Unify, term!(left.clone()), right_value).into_term())
                    .unwrap();
            }
            (VariableState::Partial(_), _) | (_, VariableState::Partial(_)) => {
                self.add_constraint(
                    &op!(Unify, term!(left.clone()), term!(right.clone())).into_term(),
                )
                .unwrap();
            }
        }
    }

    fn add_binding(&mut self, var: &Symbol, val: Term) {
        self.bindings.push(Binding(var.clone(), val));
    }

    /// Look up a variable in the bindings stack and return
    /// a reference to its value if it's bound.
    pub fn value(&self, variable: &Symbol, bsp: usize) -> Option<&Term> {
        self.bindings[..bsp]
            .iter()
            .rev()
            .find(|Binding(var, _)| var == variable)
            .map(|Binding(_, val)| val)
    }

    /// If `term` is a variable, return the value bound to that variable.
    /// If `term` is a list, dereference all items in the list.
    /// Otherwise, return `term`.
    pub fn deref(&self, term: &Term) -> Term {
        match &term.value() {
            Value::List(list) => {
                // Deref all elements.
                let mut derefed: Vec<Term> =
                    // TODO(gj): reduce recursion here.
                    list.iter().map(|t| self.deref(t)).collect();

                // If last element was a rest variable, append the list it derefed to.
                if has_rest_var(list) {
                    if let Some(last_term) = derefed.pop() {
                        if let Value::List(terms) = last_term.value() {
                            derefed.append(&mut terms.clone());
                        } else {
                            derefed.push(last_term);
                        }
                    }
                }

                term.clone_with_value(Value::List(derefed))
            }
            Value::Variable(v) => match self.variable_state(v) {
                VariableState::Bound(value) => value,
                _ => term.clone(),
            },
            Value::RestVariable(v) => match self.variable_state(v) {
                VariableState::Bound(value) => match value.value() {
                    Value::List(l) if has_rest_var(l) => self.deref(&value),
                    _ => value,
                },
                _ => term.clone(),
            },
            _ => term.clone(),
        }
    }

    /// Dereference all variables in term, including within nested structures like
    /// lists and dictionaries.
    /// Do not dereference variables inside expressions.
    pub fn deep_deref(&self, term: &Term) -> Term {
        pub struct Derefer<'a> {
            binding_manager: &'a BindingManager,
        }

        impl<'a> Derefer<'a> {
            pub fn new(binding_manager: &'a BindingManager) -> Self {
                Self { binding_manager }
            }
        }

        impl<'a> Folder for Derefer<'a> {
            fn fold_term(&mut self, t: Term) -> Term {
                match t.value() {
                    Value::List(_) => fold_term(self.binding_manager.deref(&t), self),
                    Value::Variable(_) | Value::RestVariable(_) => {
                        let derefed = self.binding_manager.deref(&t);
                        match derefed.value() {
                            Value::Expression(_) => t,
                            _ => fold_term(derefed, self),
                        }
                    }
                    _ => fold_term(t, self),
                }
            }
        }

        Derefer::new(self).fold_term(term.clone())
    }

    /// Get constraints on variable `variable`. If the variable is in a cycle,
    /// the cycle is expressed as a partial.
    pub fn get_constraints(&self, variable: &Symbol) -> Operation {
        match self.variable_state(variable) {
            VariableState::Unbound => op!(And),
            VariableState::Bound(val) => op!(And, term!(op!(Unify, term!(variable.clone()), val))),
            VariableState::Partial(expr) => expr,
            VariableState::Cycle(c) => cycle_constraints(c)
        }
    }


    // TODO (dhatch): Replace variable state with this.
    // Instead of returning a Cycle, it returns a Partial with the constraints
    // in the partial.
    pub fn variable_state_new(&self, variable: &Symbol) -> VariableState {
        match self.variable_state_at_point(variable, self.bsp()) {
            // NOTE: (dhatch) Investigating this... changing to partial seems fine, but
            // it may cause every variable to always be a partial in the VM and never
            // become bound.
            // I also think representing this as Unbound would be fine, except the
            // inverter needs the information as a constraint. not (x = y) adds a constraint
            // x != y.
            VariableState::Cycle(c) => VariableState::Partial(cycle_constraints(c)),
            vs => vs,
        }
    }

    pub fn variable_state_new_at_point(&self, variable: &Symbol, bsp: Bsp) -> VariableState {
        match self.variable_state_at_point(variable, bsp) {
            // NOTE: (dhatch) Investigating this... changing to partial seems fine, but
            // it may cause every variable to always be a partial in the VM and never
            // become bound.
            // I also think representing this as Unbound would be fine, except the
            // inverter needs the information as a constraint. not (x = y) adds a constraint
            // x != y.
            VariableState::Cycle(c) => VariableState::Partial(cycle_constraints(c)),
            vs => vs,
        }
    }

    /// Check the state of `variable`.
    pub fn variable_state(&self, variable: &Symbol) -> VariableState {
        self.variable_state_at_point(variable, self.bsp())
    }

    // TODO: Get rid of this, only used in inverter.
    /// Check the state of `variable` at `bsp`.
    pub fn variable_state_at_point(&self, variable: &Symbol, bsp: Bsp) -> VariableState {
        let mut path = vec![variable];
        while let Some(value) = self.value(path.last().unwrap(), bsp) {
            match value.value() {
                Value::Expression(e) => return VariableState::Partial(e.clone()),
                Value::Variable(v) | Value::RestVariable(v) => {
                    if v == variable {
                        return VariableState::Cycle(path.into_iter().cloned().collect());
                    } else {
                        path.push(v);
                    }
                }
                _ => return VariableState::Bound(value.clone()),
            }
        }
        VariableState::Unbound
    }

    pub fn add_constraint(&mut self, term: &Term) -> PolarResult<()> {
        self.do_followers(|_, follower| follower.add_constraint(term))?;

        assert!(term.value().as_expression().is_ok());
        let mut op = op!(And, term.clone());
        for var in op.variables().clone().iter().rev() {
            match self.variable_state(&var) {
                VariableState::Unbound => {},
                VariableState::Cycle(c) => {
                    let mut cycle = cycle_constraints(c);
                    cycle.merge_constraints(op.clone());
                    op = cycle;
                },
                VariableState::Partial(mut e) => {
                    e.merge_constraints(op);
                    op = e;
                },
                VariableState::Bound(_) => {
                    panic!("variable {:?} bound in constraint", var);
                }
            }
        }

        self.constrain(&op)
    }

    /// Add `term` as a constraint.
    //pub fn add_constraint(&mut self, term: &Term) -> PolarResult<()> {
        //self.do_followers(|follower| follower.add_constraint(term))?;

        //let Operation { operator: op, args } = term.value().as_expression().unwrap();
        //assert!(
            // !matches!(*op, Operator::And | Operator::Or),
            // "Expected a bare constraint."
        //);
        //assert!(args.len() >= 2);

        //let (left, right) = (&args[0], &args[1]);
        //match (
            //extract_variable(left.value()),
            //extract_variable(right.value()),
        //) {
            //(Value::Variable(left_name), Value::Variable(right_name)) => {
                //match (
                    //self.variable_state(left_name),
                    //self.variable_state(right_name),
                //) {
                    //(VariableState::Unbound, VariableState::Unbound) => {
                        //self.constrain(&op!(And, term.clone()))?;
                    //}
                    //(VariableState::Cycle(left_cycle), VariableState::Cycle(right_cycle)) => {
                        //let mut merged_cycles = cycle_constraints(left_cycle);
                        //merged_cycles.merge_constraints(cycle_constraints(right_cycle));
                        //self.constrain(&merged_cycles.clone_with_new_constraint(term.clone()))?;
                    //}
                    //(VariableState::Partial(partial), VariableState::Unbound)
                    //| (VariableState::Unbound, VariableState::Partial(partial)) => {
                        //self.constrain(&partial.clone_with_new_constraint(term.clone()))?;
                    //}
                    //(
                        //VariableState::Partial(mut left_partial),
                        //VariableState::Partial(right_partial),
                    //) => {
                        //left_partial.merge_constraints(right_partial);
                        //self.constrain(&left_partial.clone_with_new_constraint(term.clone()))?;
                    //}
                    //(VariableState::Partial(mut partial), VariableState::Cycle(cycle))
                    //| (VariableState::Cycle(cycle), VariableState::Partial(mut partial)) => {
                        //partial.merge_constraints(cycle_constraints(cycle));
                        //self.constrain(&partial.clone_with_new_constraint(term.clone()))?;
                    //}
                    //(VariableState::Cycle(cycle), VariableState::Unbound)
                    //| (VariableState::Unbound, VariableState::Cycle(cycle)) => {
                        //let partial = cycle_constraints(cycle);
                        //self.constrain(&partial.clone_with_new_constraint(term.clone()))?;
                    //}
                    //(VariableState::Bound(left_value), _) => {
                        //panic!(
                            //"Variable {} unexpectedly bound to {} in constraint {}.",
                            //left.to_polar(),
                            //left_value.to_polar(),
                            //term.to_polar(),
                        //);
                    //}
                    //(_, VariableState::Bound(right_value)) => {
                        //panic!(
                            //"Variable {} unexpectedly bound to {} in constraint {}.",
                            //right.to_polar(),
                            //right_value.to_polar(),
                            //term.to_polar(),
                        //);
                    //}
                //}
            //}
            //(Value::Variable(name), _) | (_, Value::Variable(name)) => {
                //match self.variable_state(name) {
                    //VariableState::Unbound => {
                        //self.constrain(&op!(And, term.clone()))?;
                    //}
                    //VariableState::Cycle(cycle) => {
                        //let partial = cycle_constraints(cycle);
                        //self.constrain(&partial.clone_with_new_constraint(term.clone()))?;
                    //}
                    //VariableState::Partial(partial) => {
                        //self.constrain(&partial.clone_with_new_constraint(term.clone()))?;
                    //}
                    //VariableState::Bound(value) => {
                        //panic!(
                            //"Variable {} unexpectedly bound to {} in constraint {}.",
                            //name.0,
                            //value.to_polar(),
                            //term.to_polar()
                        //);
                    //}
                //}
            //}
            //(_, _) => panic!(
                //"At least one side of a constraint expression must be a variable. This is {} {}",
                //left.to_polar(),
                //right.to_polar()
            //),
        //}

        //Ok(())
    //}

    // TODO (dhatch) This is still called from the VM, breaks followers.
    pub fn constrain(&mut self, o: &Operation) -> PolarResult<()> {
        assert_eq!(o.operator, Operator::And, "bad constraint {}", o.to_polar());
        for var in o.variables() {
            match self.variable_state(&var) {
                VariableState::Bound(_) => (),
                _ => self.add_binding(&var, o.clone().into_term()),
            }
        }
        Ok(())
    }

    /// Reset the state of `BindingManager` to what it was at `to`.
    pub fn backtrack(&mut self, to: Bsp) {
        self.do_followers(|follower_bsp, follower| {
            let follower_backtrack_to = to.saturating_sub(follower_bsp);
            follower.backtrack(follower_backtrack_to);
            Ok(())
        }).unwrap();

        self.bindings.truncate(to)
    }

    /// Return all variables used in this binding manager.
    pub fn variables(&self) -> HashSet<Symbol> {
        self.bindings
            .iter()
            .map(|Binding(v, _)| v.clone())
            .collect()
    }

    /// Retrieve an opaque value representing the current state of `BindingManager`.
    /// Can be used to reset state with `backtrack`.
    pub fn bsp(&self) -> Bsp {
        self.bindings.len()
    }

    pub fn bindings(&self, include_temps: bool) -> Bindings {
        self.bindings_after(include_temps, 0)
    }

    pub fn bindings_after(&self, include_temps: bool, after: Bsp) -> Bindings {
        let mut bindings = HashMap::new();
        for Binding(var, value) in &self.bindings[after..] {
            if !include_temps && var.is_temporary_var() {
                continue;
            }
            bindings.insert(var.clone(), self.deep_deref(value));
        }
        bindings
    }

    // TODO rename to deep_deref_batch
    pub fn variable_bindings(&self, variables: &HashSet<Symbol>) -> Bindings {
        let mut bindings = HashMap::new();
        for var in variables.iter() {
            let value = self.value(var, self.bsp());
            if let Some(value) = value {
                bindings.insert(var.clone(), self.deep_deref(value));
            }
        }
        bindings
    }

    /// Get the bindings stack *for debugging purposes only*.
    pub fn bindings_debug(&self) -> &BindingStack {
        &self.bindings
    }

    pub fn add_follower(&mut self, follower: BindingManager) -> FollowerId {
        let follower_id = self.next_follower_id;
        self.followers.insert(follower_id, follower);
        self.follower_bsps.insert(follower_id, self.bsp());
        self.next_follower_id += 1;

        follower_id
    }

    pub fn remove_follower(&mut self, follower_id: &FollowerId) -> Option<BindingManager> {
        self.followers.remove(follower_id)
    }

    fn do_followers<F>(&mut self, func: F) -> PolarResult<()>
    where
        F: Fn(Bsp, &mut BindingManager) -> PolarResult<()>,
    {
        for (id, follower) in self.followers.iter_mut() {
            let bsp = self.follower_bsps.get(id).unwrap();
            func(*bsp, follower)?
        }

        Ok(())
    }

    // TODO maybe port from VM:
    // relevant_bindings
    // variable_bindings
    // bindings
}

/// Get variable out of a term for ``add_constraint`` to determine where the
/// constraint is stored.
///
/// If the term is a variable, the variable is returned.
/// If the term is a dot expression, the VAR from VAR.field is returned.
/// Otherwise, the term is returned.
fn extract_variable(value: &Value) -> &Value {
    match value {
        Value::Variable(_) => value,
        Value::Expression(expr) if expr.operator == Operator::Dot => {
            extract_variable(expr.args[0].value())
        }
        _ => value,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn variable_state() {
        let mut bindings = BindingManager::new();

        let x = sym!("x");
        let y = sym!("y");
        let z = sym!("z");

        // Unbound.
        assert_eq!(bindings.variable_state(&x), VariableState::Unbound);

        // Bound.
        bindings.add_binding(&x, term!(1));
        assert_eq!(bindings.variable_state(&x), VariableState::Bound(term!(1)));

        bindings.add_binding(&x, term!(x.clone()));
        assert_eq!(
            bindings.variable_state(&x),
            VariableState::Cycle(vec![x.clone()])
        );

        // 2-cycle.
        bindings.add_binding(&x, term!(y.clone()));
        bindings.add_binding(&y, term!(x.clone()));
        assert_eq!(
            bindings.variable_state(&x),
            VariableState::Cycle(vec![x.clone(), y.clone()])
        );
        assert_eq!(
            bindings.variable_state(&y),
            VariableState::Cycle(vec![y.clone(), x.clone()])
        );

        // 3-cycle.
        bindings.add_binding(&x, term!(y.clone()));
        bindings.add_binding(&y, term!(z.clone()));
        bindings.add_binding(&z, term!(x.clone()));
        assert_eq!(
            bindings.variable_state(&x),
            VariableState::Cycle(vec![x.clone(), y.clone(), z.clone()])
        );
        assert_eq!(
            bindings.variable_state(&y),
            VariableState::Cycle(vec![y.clone(), z.clone(), x.clone()])
        );
        assert_eq!(
            bindings.variable_state(&z),
            VariableState::Cycle(vec![z.clone(), x.clone(), y])
        );

        // Expression.
        bindings.add_binding(&x, term!(op!(And)));
        assert_eq!(
            bindings.variable_state(&x),
            VariableState::Partial(op!(And))
        );
    }

    // Fails for now. See note in bind.
    #[test]
    #[ignore]
    /// Test creating a group of variables bound together, and rebinding them.
    fn rebind_variable_group() {
        let mut bindings = BindingManager::new();
        bindings.bind(&sym!("x"), term!(sym!("y")));
        bindings.bind(&sym!("y"), term!(sym!("z")));

        bindings.bind(&sym!("z"), term!(1));

        // All have value 1.
        assert_eq!(
            bindings.variable_state(&sym!("x")),
            VariableState::Bound(term!(1))
        );
        assert_eq!(
            bindings.variable_state(&sym!("y")),
            VariableState::Bound(term!(1))
        );
        assert_eq!(
            bindings.variable_state(&sym!("z")),
            VariableState::Bound(term!(1))
        );

        bindings.bind(&sym!("x"), term!(2));

        // This doesn't always change all variables, and sometimes changes more than one variable.
        // What should happen here?
        // If we don't support rebinding, it's easier, but some parts of the VM subtly
        // require rebinding.
        assert_eq!(
            bindings.variable_state(&sym!("x")),
            VariableState::Bound(term!(2))
        );
        assert_eq!(
            bindings.variable_state(&sym!("y")),
            VariableState::Bound(term!(1))
        );
        assert_eq!(
            bindings.variable_state(&sym!("z")),
            VariableState::Bound(term!(1))
        );
    }

    #[test]
    fn test_followers() {
        // Regular bindings
        let mut b1 = BindingManager::new();
        b1.bind(&sym!("x"), term!(1));
        b1.bind(&sym!("y"), term!(2));

        assert_eq!(b1.variable_state(&sym!("x")), VariableState::Bound(term!(1)));
        assert_eq!(b1.variable_state(&sym!("y")), VariableState::Bound(term!(2)));

        let b2 = BindingManager::new();
        let b2_id = b1.add_follower(b2);

        b1.bind(&sym!("z"), term!(3));

        assert_eq!(b1.variable_state(&sym!("x")), VariableState::Bound(term!(1)));
        assert_eq!(b1.variable_state(&sym!("y")), VariableState::Bound(term!(2)));
        assert_eq!(b1.variable_state(&sym!("z")), VariableState::Bound(term!(3)));

        let b2 = b1.remove_follower(&b2_id).unwrap();
        assert_eq!(b2.variable_state(&sym!("x")), VariableState::Unbound);
        assert_eq!(b2.variable_state(&sym!("y")), VariableState::Unbound);
        assert_eq!(b2.variable_state(&sym!("z")), VariableState::Bound(term!(3)));

        // Extending cycle.
        let mut b1 = BindingManager::new();
        b1.bind(&sym!("x"), term!(sym!("y")));
        b1.bind(&sym!("x"), term!(sym!("z")));

        let b2 = BindingManager::new();
        let b2_id = b1.add_follower(b2);

        assert!(matches!(b1.variable_state(&sym!("x")), VariableState::Cycle(_)));
        assert!(matches!(b1.variable_state(&sym!("y")), VariableState::Cycle(_)));
        assert!(matches!(b1.variable_state(&sym!("z")), VariableState::Cycle(_)));

        b1.bind(&sym!("x"), term!(sym!("a")));
        if let VariableState::Cycle(c) = b1.variable_state(&sym!("a")) {
            assert_eq!(c, vec![sym!("a"), sym!("x"), sym!("y"), sym!("z")], "c was {:?}", c);
        }

        let b2 = b1.remove_follower(&b2_id).unwrap();
        if let VariableState::Cycle(c) = b2.variable_state(&sym!("a")) {
            assert_eq!(c, vec![sym!("a"), sym!("x")], "c was {:?}", c);
        } else {
            panic!("unexpected");
        }
        if let VariableState::Cycle(c) = b2.variable_state(&sym!("x")) {
            assert_eq!(c, vec![sym!("x"), sym!("a")], "c was {:?}", c);
        } else {
            panic!("unexpected");
        }

        // Adding constraints to cycles.
        let mut b1 = BindingManager::new();
        b1.bind(&sym!("x"), term!(sym!("y")));
        b1.bind(&sym!("x"), term!(sym!("z")));

        let b2 = BindingManager::new();
        let b2_id = b1.add_follower(b2);

        assert!(matches!(b1.variable_state(&sym!("x")), VariableState::Cycle(_)));
        assert!(matches!(b1.variable_state(&sym!("y")), VariableState::Cycle(_)));
        assert!(matches!(b1.variable_state(&sym!("z")), VariableState::Cycle(_)));

        b1.add_constraint(&term!(op!(Gt, term!(sym!("x")), term!(sym!("y"))))).unwrap();

        let b2 = b1.remove_follower(&b2_id).unwrap();

        if let VariableState::Partial(p) = b1.variable_state(&sym!("x")) {
            assert_eq!(p.to_polar(), "x = y and y = z and y = z and z = x and x > y");
        } else {
            panic!("unexpected");
        }

        if let VariableState::Partial(p) = b2.variable_state(&sym!("x")) {
            assert_eq!(p.to_polar(), "x > y");
        } else {
            panic!("unexpected");
        }
    }

    // TODO (dhatch): Test backtrack with followers.
}
