/// Manage binding state in the VM.
///
/// Bindings associate variables in the VM with constraints or values.
use std::collections::{HashMap, HashSet};

use crate::error::{PolarResult, RuntimeError};
use crate::folder::{fold_term, Folder};
use crate::formatting::ToPolarString;
use crate::terms::{has_rest_var, Operation, Operator, Symbol, Term, Value};

#[derive(Clone, Debug)]
pub struct Binding(pub Symbol, pub Term);

// TODO This is only public for debugger and inverter.
// Eventually this should be an internal interface.
pub type BindingStack = Vec<Binding>;
pub type Bindings = HashMap<Symbol, Term>;

pub type Bsp = Bsps;
pub type FollowerId = usize;

/// Bsps represents bsps of a binding manager and its followers as a tree.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bsps {
    /// Index into `bindings` array
    bindings_index: usize,
    /// Store bsps of followers (and their followers) by follower id.
    followers: HashMap<FollowerId, Bsps>,
}

/// Variable binding state.
///
/// A variable is Unbound if it is not bound to a concrete value.
/// A variable is Bound if it is bound to a ground value (not another variable).
/// A variable is Partial if it is bound to other variables, or constrained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariableState {
    Unbound,
    Bound(Term),
    Partial,
}

/// Represent each binding in a cycle as a unification constraint.
// TODO(gj): put this in an impl block on VariableState?
fn cycle_constraints(cycle: Vec<Symbol>) -> Operation {
    let mut constraints = op!(And);
    for (x, y) in cycle.iter().zip(cycle.iter().skip(1)) {
        constraints.add_constraint(op!(Unify, term!(x.clone()), term!(y.clone())));
    }
    constraints
}

impl From<BindingManagerVariableState<'_>> for VariableState {
    fn from(other: BindingManagerVariableState) -> Self {
        // We represent Cycles as a Partial VariableState. This information is not
        // needed in the VM, so unbound could be an acceptable representation as well.
        // The partial representation does not slow down the VM since grounding happens
        // within BindingManager::bind. The fast path of `bind_variables` is still taken
        // instead of running Operation::ground.
        match other {
            BindingManagerVariableState::Unbound => VariableState::Unbound,
            BindingManagerVariableState::Bound(b) => VariableState::Bound(b),
            BindingManagerVariableState::Cycle(_) => VariableState::Partial,
            BindingManagerVariableState::Partial(_) => VariableState::Partial,
        }
    }
}

/// Internal variable binding state.
///
/// Includes the Cycle representation in addition to VariableState.
#[derive(Clone, Debug, PartialEq, Eq)]
enum BindingManagerVariableState<'a> {
    Unbound,
    Bound(Term),
    Cycle(Vec<Symbol>),
    Partial(&'a Operation),
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
#[derive(Clone, Debug, Default)]
pub struct BindingManager {
    bindings: BindingStack,
    followers: HashMap<FollowerId, BindingManager>,
    next_follower_id: FollowerId,
}

// Public interface.
impl BindingManager {
    pub fn new() -> Self {
        Self::default()
    }

    // **** State Mutation ***

    /// Bind `var` to `val`.
    ///
    /// If the binding succeeds, Ok is returned. If the binding is *incompatible*
    /// an error is returned.
    ///
    /// A binding is considered *incompatible* if either:
    ///
    /// 1. `var` is already bound to some value (rebindings are not allowed, even if the
    ///    rebinding is to the same value).
    /// 2. `var` is constrained, and the new binding of `val` is not compatible with those
    ///    constraints.
    ///
    /// If a binding is compatible, it is recorded. If the binding was to a ground value,
    /// subsequent calls to `variable_state` or `deref` will return that value.
    ///
    /// If the binding was between two variables, the two will always have the same value
    /// or constraints going forward. Further, a unification constraint is recorded between
    /// the two variables.
    ///
    /// If either variable is bound in the future, both will be bound to that value
    /// (`variable_state` and `deref` will return the same value).
    ///
    /// If a binding between two variables is made, and one is bound and the other unbound, the
    /// unbound variable will take the value of the bound one.
    pub fn bind(&mut self, var: &Symbol, val: Term) -> PolarResult<()> {
        if let Ok(symbol) = val.value().as_symbol() {
            self.bind_variables(var, symbol)?;
        } else if let BindingManagerVariableState::Partial(p) = self._variable_state(var) {
            if let Some(grounded) = p.ground(var.clone(), val.clone()) {
                self.add_binding(var, val.clone());
                self.constrain(&grounded)?;
            } else {
                return Err(RuntimeError::IncompatibleBindings {
                    msg: "Grounding failed".into(),
                }
                .into());
            }
        } else {
            if let BindingManagerVariableState::Bound(_) = self._variable_state(var) {
                return Err(RuntimeError::IncompatibleBindings {
                    msg: format!("Cannot rebind {:?}", var),
                }
                .into());
            }

            self.add_binding(var, val.clone());
        }

        // If the main binding succeeded, the follower binding must succeed.
        self.do_followers(|_, follower| follower.bind(var, val.clone()))
            .unwrap();

        Ok(())
    }

    /// Rebind `var` to `val`, regardless of compatibility.
    ///
    /// A rebinding is only allowed if a variable is unbound, or already bound.
    ///
    /// Constrained variables, or variables that have been bound with other variables
    /// cannot be rebound.
    ///
    /// Note: Rebinding a variable that has been previously bound to other variables will place the
    /// BindingManager in an invalid state. For this reason, rebinding should be used with care.
    ///
    /// (The only current usage is for replacing default values with call ids).
    pub fn unsafe_rebind(&mut self, var: &Symbol, val: Term) {
        assert!(matches!(
            self._variable_state(var),
            BindingManagerVariableState::Unbound | BindingManagerVariableState::Bound(_)
        ));
        self.add_binding(var, val);
    }

    /// Add a constraint. Constraints are represented as term expressions.
    ///
    /// `term` must be an expression`.
    ///
    /// An error is returned if the constraint is incompatible with existing constraints.
    ///
    /// (Currently all constraints are considered compatible).
    pub fn add_constraint(&mut self, term: &Term) -> PolarResult<()> {
        self.do_followers(|_, follower| follower.add_constraint(term))?;

        assert!(term.value().as_expression().is_ok());
        let mut op = op!(And, term.clone());
        for var in op.variables().iter().rev() {
            match self._variable_state(&var) {
                BindingManagerVariableState::Unbound => {}
                BindingManagerVariableState::Cycle(c) => {
                    let mut cycle = cycle_constraints(c);
                    cycle.merge_constraints(op.clone());
                    op = cycle;
                }
                BindingManagerVariableState::Partial(e) => {
                    let mut e = e.clone();
                    e.merge_constraints(op);
                    op = e;
                }
                BindingManagerVariableState::Bound(v) => {
                    panic!(
                        "Unexpected bound variable {var} in constraint. {var} = {val}",
                        var = var,
                        val = v
                    );
                }
            }
        }

        self.constrain(&op)
    }

    /// Reset the state of `BindingManager` to what it was at `to`.
    pub fn backtrack(&mut self, to: &Bsp) {
        self.do_followers(|follower_id, follower| {
            if let Some(follower_to) = to.followers.get(&follower_id) {
                follower.backtrack(follower_to);
            } else {
                follower.backtrack(&Bsp::default());
            }
            Ok(())
        })
        .unwrap();

        self.bindings.truncate(to.bindings_index)
    }

    // *** Binding Inspection ***

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
                    Value::Expression(_) => t,
                    Value::List(_) => fold_term(self.binding_manager.deref(&t), self),
                    Value::Variable(_) | Value::RestVariable(_) => {
                        let derefed = self.binding_manager.deref(&t);
                        fold_term(derefed, self)
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
        match self._variable_state(variable) {
            BindingManagerVariableState::Unbound => op!(And),
            BindingManagerVariableState::Bound(val) => {
                op!(And, term!(op!(Unify, term!(variable.clone()), val)))
            }
            BindingManagerVariableState::Partial(expr) => expr.clone(),
            BindingManagerVariableState::Cycle(c) => cycle_constraints(c),
        }
    }

    pub fn variable_state(&self, variable: &Symbol) -> VariableState {
        self.variable_state_at_point(variable, &self.bsp())
    }

    pub fn variable_state_at_point(&self, variable: &Symbol, bsp: &Bsp) -> VariableState {
        let index = bsp.bindings_index;
        let mut next = variable;
        while let Some(value) = self.value(next, index) {
            match value.value() {
                Value::Expression(_) => return VariableState::Partial,
                Value::Variable(v) | Value::RestVariable(v) => {
                    if v == variable {
                        return VariableState::Partial;
                    } else {
                        next = v;
                    }
                }
                _ => return VariableState::Bound(value.clone()),
            }
        }
        VariableState::Unbound
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
        let follower_bsps = self
            .followers
            .iter()
            .map(|(id, f)| (*id, f.bsp()))
            .collect::<HashMap<_, _>>();

        Bsps {
            bindings_index: self.bindings.len(),
            followers: follower_bsps,
        }
    }

    pub fn bindings(&self, include_temps: bool) -> Bindings {
        self.bindings_after(include_temps, &Bsp::default())
    }

    pub fn bindings_after(&self, include_temps: bool, after: &Bsp) -> Bindings {
        let mut bindings = HashMap::new();
        for Binding(var, value) in &self.bindings[after.bindings_index..] {
            if !include_temps && var.is_temporary_var() {
                continue;
            }
            bindings.insert(var.clone(), self.deep_deref(value));
        }
        bindings
    }

    pub fn variable_bindings(&self, variables: &HashSet<Symbol>) -> Bindings {
        let mut bindings = HashMap::new();
        for var in variables.iter() {
            let value = self.value(var, self.bsp().bindings_index);
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

    // *** Followers ***

    pub fn add_follower(&mut self, follower: BindingManager) -> FollowerId {
        let follower_id = self.next_follower_id;
        self.followers.insert(follower_id, follower);
        self.next_follower_id += 1;

        follower_id
    }

    pub fn remove_follower(&mut self, follower_id: &FollowerId) -> Option<BindingManager> {
        self.followers.remove(follower_id)
    }
}

// Private impls.
impl BindingManager {
    /// Bind two variables together.
    fn bind_variables(&mut self, left: &Symbol, right: &Symbol) -> PolarResult<()> {
        match (self._variable_state(left), self._variable_state(right)) {
            (
                BindingManagerVariableState::Bound(left_value),
                BindingManagerVariableState::Unbound,
            ) => {
                self.add_binding(right, left_value);
            }
            (
                BindingManagerVariableState::Unbound,
                BindingManagerVariableState::Bound(right_value),
            ) => {
                self.add_binding(left, right_value);
            }

            // Cycles: one or more variables are bound together.
            (BindingManagerVariableState::Unbound, BindingManagerVariableState::Unbound) => {
                // Both variables are unbound. Bind them in a new cycle,
                // but do not create 1-cycles.
                if left != right {
                    self.add_binding(left, term!(right.clone()));
                    self.add_binding(right, term!(left.clone()));
                }
            }
            (BindingManagerVariableState::Cycle(cycle), BindingManagerVariableState::Unbound) => {
                // Left is in a cycle. Extend it to include right.
                let last = cycle.last().unwrap();
                assert_ne!(last, left);
                self.add_binding(last, term!(right.clone()));
                self.add_binding(right, term!(left.clone()));
            }
            (BindingManagerVariableState::Unbound, BindingManagerVariableState::Cycle(cycle)) => {
                // Right is in a cycle. Extend it to include left.
                let last = cycle.last().unwrap();
                assert_ne!(last, right);
                self.add_binding(last, term!(left.clone()));
                self.add_binding(left, term!(right.clone()));
            }
            (
                BindingManagerVariableState::Cycle(left_cycle),
                BindingManagerVariableState::Cycle(right_cycle),
            ) => {
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
            (
                BindingManagerVariableState::Cycle(_),
                BindingManagerVariableState::Bound(right_value),
            ) => {
                // Ground out the cycle.
                self.add_binding(left, right_value);
            }
            (
                BindingManagerVariableState::Bound(left_value),
                BindingManagerVariableState::Cycle(_),
            ) => {
                // Left is currently bound. Ground right cycle.
                self.add_binding(right, left_value);
            }
            (BindingManagerVariableState::Bound(_), BindingManagerVariableState::Bound(_)) => {
                return Err(RuntimeError::IncompatibleBindings {
                    msg: format!("{} and {} are both bound", left, right),
                }
                .into());
            }
            (
                BindingManagerVariableState::Bound(left_value),
                BindingManagerVariableState::Partial(_),
            ) => {
                // Left is bound, right has constraints.
                // TODO (dhatch): No unwrap.
                self.add_constraint(&op!(Unify, left_value, term!(right.clone())).into_term())?;
            }
            (
                BindingManagerVariableState::Partial(_),
                BindingManagerVariableState::Bound(right_value),
            ) => {
                self.add_constraint(&op!(Unify, term!(left.clone()), right_value).into_term())?;
            }
            (BindingManagerVariableState::Partial(_), _)
            | (_, BindingManagerVariableState::Partial(_)) => {
                self.add_constraint(
                    &op!(Unify, term!(left.clone()), term!(right.clone())).into_term(),
                )?;
            }
        }

        Ok(())
    }

    fn add_binding(&mut self, var: &Symbol, val: Term) {
        self.bindings.push(Binding(var.clone(), val));
    }

    /// Look up a variable in the bindings stack and return
    /// a reference to its value if it's bound.
    fn value(&self, variable: &Symbol, bsp: usize) -> Option<&Term> {
        self.bindings[..bsp]
            .iter()
            .rev()
            .find(|Binding(var, _)| var == variable)
            .map(|Binding(_, val)| val)
    }

    fn _variable_state(&self, variable: &Symbol) -> BindingManagerVariableState {
        self._variable_state_at_point(variable, &self.bsp())
    }

    /// Check the state of `variable` at `bsp`.
    fn _variable_state_at_point(
        &self,
        variable: &Symbol,
        bsp: &Bsp,
    ) -> BindingManagerVariableState {
        let index = bsp.bindings_index;
        let mut path = vec![variable];
        while let Some(value) = self.value(path.last().unwrap(), index) {
            match value.value() {
                Value::Expression(e) => return BindingManagerVariableState::Partial(e),
                Value::Variable(v) | Value::RestVariable(v) => {
                    if v == variable {
                        return BindingManagerVariableState::Cycle(
                            path.into_iter().cloned().collect(),
                        );
                    } else {
                        path.push(v);
                    }
                }
                _ => return BindingManagerVariableState::Bound(value.clone()),
            }
        }
        BindingManagerVariableState::Unbound
    }

    #[allow(clippy::unnecessary_wraps)]
    fn constrain(&mut self, o: &Operation) -> PolarResult<()> {
        assert_eq!(o.operator, Operator::And, "bad constraint {}", o.to_polar());
        for var in o.variables() {
            match self._variable_state(&var) {
                // A constraint should not contain a bound variable, it should have been removed in
                // add_constraint by calling ground.
                BindingManagerVariableState::Bound(_) => {
                    panic!("Unexpected bound variable in constraint.")
                }
                _ => self.add_binding(&var, o.clone().into_term()),
            }
        }
        Ok(())
    }

    fn do_followers<F>(&mut self, mut func: F) -> PolarResult<()>
    where
        F: FnMut(FollowerId, &mut BindingManager) -> PolarResult<()>,
    {
        for (id, follower) in self.followers.iter_mut() {
            func(*id, follower)?
        }

        Ok(())
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
        assert_eq!(
            bindings._variable_state(&x),
            BindingManagerVariableState::Unbound
        );

        // Bound.
        bindings.add_binding(&x, term!(1));
        assert_eq!(
            bindings._variable_state(&x),
            BindingManagerVariableState::Bound(term!(1))
        );

        bindings.add_binding(&x, term!(x.clone()));
        assert_eq!(
            bindings._variable_state(&x),
            BindingManagerVariableState::Cycle(vec![x.clone()])
        );

        // 2-cycle.
        bindings.add_binding(&x, term!(y.clone()));
        bindings.add_binding(&y, term!(x.clone()));
        assert_eq!(
            bindings._variable_state(&x),
            BindingManagerVariableState::Cycle(vec![x.clone(), y.clone()])
        );
        assert_eq!(
            bindings._variable_state(&y),
            BindingManagerVariableState::Cycle(vec![y.clone(), x.clone()])
        );

        // 3-cycle.
        bindings.add_binding(&x, term!(y.clone()));
        bindings.add_binding(&y, term!(z.clone()));
        bindings.add_binding(&z, term!(x.clone()));
        assert_eq!(
            bindings._variable_state(&x),
            BindingManagerVariableState::Cycle(vec![x.clone(), y.clone(), z.clone()])
        );
        assert_eq!(
            bindings._variable_state(&y),
            BindingManagerVariableState::Cycle(vec![y.clone(), z.clone(), x.clone()])
        );
        assert_eq!(
            bindings._variable_state(&z),
            BindingManagerVariableState::Cycle(vec![z.clone(), x.clone(), y])
        );

        // Expression.
        bindings.add_binding(&x, term!(op!(And)));
        assert_eq!(
            bindings._variable_state(&x),
            BindingManagerVariableState::Partial(&op!(And))
        );
    }

    #[test]
    fn test_followers() {
        // Regular bindings
        let mut b1 = BindingManager::new();
        b1.bind(&sym!("x"), term!(1)).unwrap();
        b1.bind(&sym!("y"), term!(2)).unwrap();

        assert_eq!(
            b1._variable_state(&sym!("x")),
            BindingManagerVariableState::Bound(term!(1))
        );
        assert_eq!(
            b1._variable_state(&sym!("y")),
            BindingManagerVariableState::Bound(term!(2))
        );

        let b2 = BindingManager::new();
        let b2_id = b1.add_follower(b2);

        b1.bind(&sym!("z"), term!(3)).unwrap();

        assert_eq!(
            b1._variable_state(&sym!("x")),
            BindingManagerVariableState::Bound(term!(1))
        );
        assert_eq!(
            b1._variable_state(&sym!("y")),
            BindingManagerVariableState::Bound(term!(2))
        );
        assert_eq!(
            b1._variable_state(&sym!("z")),
            BindingManagerVariableState::Bound(term!(3))
        );

        let b2 = b1.remove_follower(&b2_id).unwrap();
        assert_eq!(
            b2._variable_state(&sym!("x")),
            BindingManagerVariableState::Unbound
        );
        assert_eq!(
            b2._variable_state(&sym!("y")),
            BindingManagerVariableState::Unbound
        );
        assert_eq!(
            b2._variable_state(&sym!("z")),
            BindingManagerVariableState::Bound(term!(3))
        );

        // Extending cycle.
        let mut b1 = BindingManager::new();
        b1.bind(&sym!("x"), term!(sym!("y"))).unwrap();
        b1.bind(&sym!("x"), term!(sym!("z"))).unwrap();

        let b2 = BindingManager::new();
        let b2_id = b1.add_follower(b2);

        assert!(matches!(
            b1._variable_state(&sym!("x")),
            BindingManagerVariableState::Cycle(_)
        ));
        assert!(matches!(
            b1._variable_state(&sym!("y")),
            BindingManagerVariableState::Cycle(_)
        ));
        assert!(matches!(
            b1._variable_state(&sym!("z")),
            BindingManagerVariableState::Cycle(_)
        ));

        b1.bind(&sym!("x"), term!(sym!("a"))).unwrap();
        if let BindingManagerVariableState::Cycle(c) = b1._variable_state(&sym!("a")) {
            assert_eq!(
                c,
                vec![sym!("a"), sym!("x"), sym!("y"), sym!("z")],
                "c was {:?}",
                c
            );
        }

        let b2 = b1.remove_follower(&b2_id).unwrap();
        if let BindingManagerVariableState::Cycle(c) = b2._variable_state(&sym!("a")) {
            assert_eq!(c, vec![sym!("a"), sym!("x")], "c was {:?}", c);
        } else {
            panic!("unexpected");
        }
        if let BindingManagerVariableState::Cycle(c) = b2._variable_state(&sym!("x")) {
            assert_eq!(c, vec![sym!("x"), sym!("a")], "c was {:?}", c);
        } else {
            panic!("unexpected");
        }

        // Adding constraints to cycles.
        let mut b1 = BindingManager::new();
        b1.bind(&sym!("x"), term!(sym!("y"))).unwrap();
        b1.bind(&sym!("x"), term!(sym!("z"))).unwrap();

        let b2 = BindingManager::new();
        let b2_id = b1.add_follower(b2);

        assert!(matches!(
            b1._variable_state(&sym!("x")),
            BindingManagerVariableState::Cycle(_)
        ));
        assert!(matches!(
            b1._variable_state(&sym!("y")),
            BindingManagerVariableState::Cycle(_)
        ));
        assert!(matches!(
            b1._variable_state(&sym!("z")),
            BindingManagerVariableState::Cycle(_)
        ));

        b1.add_constraint(&term!(op!(Gt, term!(sym!("x")), term!(sym!("y")))))
            .unwrap();

        let b2 = b1.remove_follower(&b2_id).unwrap();

        if let BindingManagerVariableState::Partial(p) = b1._variable_state(&sym!("x")) {
            assert_eq!(
                p.to_polar(),
                "x = y and y = z and y = z and z = x and x > y"
            );
        } else {
            panic!("unexpected");
        }

        if let BindingManagerVariableState::Partial(p) = b2._variable_state(&sym!("x")) {
            assert_eq!(p.to_polar(), "x > y");
        } else {
            panic!("unexpected");
        }
    }

    #[test]
    fn deref() {
        let mut bm = BindingManager::default();
        let value = term!(1);
        let x = sym!("x");
        let y = sym!("y");
        let term_x = term!(x.clone());
        let term_y = term!(y.clone());

        // unbound var
        assert_eq!(bm.deref(&term_x), term_x);

        // unbound var -> unbound var
        bm.bind(&x, term_y.clone()).unwrap();
        assert_eq!(bm.deref(&term_x), term_x);

        // value
        assert_eq!(bm.deref(&value), value.clone());

        // unbound var -> value
        let mut bm = BindingManager::default();
        bm.bind(&x, value.clone()).unwrap();
        assert_eq!(bm.deref(&term_x), value);

        // unbound var -> unbound var -> value
        let mut bm = BindingManager::default();
        bm.bind(&x, term_y).unwrap();
        bm.bind(&y, value.clone()).unwrap();
        assert_eq!(bm.deref(&term_x), value);
    }

    #[test]
    fn deep_deref() {
        let mut bm = BindingManager::default();
        let one = term!(1);
        let two = term!(1);
        let one_var = sym!("one");
        let two_var = sym!("two");
        bm.bind(&one_var, one.clone()).unwrap();
        bm.bind(&two_var, two.clone()).unwrap();
        let dict = btreemap! {
            sym!("x") => term!(one_var),
            sym!("y") => term!(two_var),
        };
        let list = term!([dict]);
        assert_eq!(
            bm.deep_deref(&list).value().clone(),
            Value::List(vec![term!(btreemap! {
                sym!("x") => one,
                sym!("y") => two,
            })])
        );
    }

    #[test]
    fn bind() {
        let x = sym!("x");
        let y = sym!("y");
        let zero = term!(0);
        let mut bm = BindingManager::default();
        bm.bind(&x, zero.clone()).unwrap();
        assert_eq!(bm.variable_state(&x), VariableState::Bound(zero));
        assert_eq!(bm.variable_state(&y), VariableState::Unbound);
    }

    #[test]
    fn test_backtrack_followers() {
        // Regular bindings
        let mut b1 = BindingManager::new();
        b1.bind(&sym!("x"), term!(sym!("y"))).unwrap();
        b1.bind(&sym!("z"), term!(sym!("x"))).unwrap();

        let b2 = BindingManager::new();
        let b2_id = b1.add_follower(b2);

        b1.add_constraint(&term!(op!(Gt, term!(sym!("x")), term!(1))))
            .unwrap();

        let bsp = b1.bsp();

        b1.bind(&sym!("a"), term!(sym!("x"))).unwrap();
        assert!(matches!(
            b1.variable_state(&sym!("a")),
            VariableState::Partial
        ));

        b1.backtrack(&bsp);
        let b2 = b1.remove_follower(&b2_id).unwrap();
        assert!(matches!(
            b2.variable_state(&sym!("a")),
            VariableState::Unbound
        ));
    }
}
