/// Manage binding state in the VM.
///
/// Bindings associate variables in the VM with constraints or values.
use std::collections::{HashMap, HashSet};

use crate::{
    error::{PolarResult, RuntimeError},
    folder::{fold_list, fold_term, Folder},
    terms::{has_rest_var, Operation, Operator, Symbol, Term, Value},
    vm::Goal,
};

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

struct Derefer<'a> {
    binding_manager: &'a BindingManager,
    seen: HashSet<u64>,
}

impl<'a> Derefer<'a> {
    fn new(binding_manager: &'a BindingManager) -> Self {
        Self {
            binding_manager,
            seen: HashSet::new(),
        }
    }
}

impl<'a> Folder for Derefer<'a> {
    fn fold_list(&mut self, list: Vec<Term>) -> Vec<Term> {
        let has_rest = has_rest_var(&list);
        let mut list = fold_list(list, self);
        if has_rest {
            let last = list.pop().unwrap();
            if let Value::List(rest) = last.value() {
                list.append(&mut rest.clone());
            } else {
                list.push(last);
            }
        }
        list
    }

    fn fold_term(&mut self, t: Term) -> Term {
        match t.value() {
            Value::Expression(_) => t,
            Value::Variable(v) | Value::RestVariable(v) => {
                let hash = t.hash_value();
                if self.seen.contains(&hash) {
                    t
                } else {
                    self.seen.insert(hash);
                    let t = self.binding_manager.lookup(v).unwrap_or(t);
                    let t = fold_term(t, self);
                    self.seen.remove(&hash);
                    t
                }
            }
            _ => fold_term(t, self),
        }
    }
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
        use BindingManagerVariableState::*;
        // We represent Cycles as a Partial VariableState. This information is not
        // needed in the VM, so unbound could be an acceptable representation as well.
        // The partial representation does not slow down the VM since grounding happens
        // within BindingManager::bind. The fast path of `bind_variables` is still taken
        // instead of running Operation::ground.
        match other {
            Unbound => Self::Unbound,
            Bound(b) => Self::Bound(b),
            Cycle(_) => Self::Partial,
            Partial(_) => Self::Partial,
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

    /// Bind `var` to `val` in the expression `partial`.
    ///
    /// If the binding succeeds, the new expression is returned as a goal. Otherwise,
    /// an error is returned.
    fn partial_bind(&mut self, partial: Operation, var: &Symbol, val: Term) -> PolarResult<Goal> {
        match partial.ground(var, val.clone()) {
            None => Err(RuntimeError::IncompatibleBindings {
                msg: "Grounding failed A".into(),
            }
            .into()),
            Some(grounded) => {
                self.add_binding(var, val);
                Ok(Goal::Query {
                    term: grounded.into(),
                })
            }
        }
    }

    // **** State Mutation ***

    /// Bind `var` to `val`.
    ///
    /// If the binding succeeds, Ok with an optional goal is returned. The goal will be
    /// present if the binding replaces a partial, which then needs to be reevaluated
    /// to ensure compatibility.
    ///
    /// If the binding is *incompatible* an error is returned. A binding is considered
    /// *incompatible* if either:
    ///
    /// 1. `var` is already bound to some value (rebindings are not allowed, even if the
    ///    rebinding is to the same value).
    /// 2. `var` is constrained, and the new binding of `val` is not compatible with those
    ///    constraints (as determined by `Operation::ground()`)
    ///
    /// If a binding is compatible, it is recorded. If the binding was to a ground value,
    /// subsequent calls to `variable_state` or `deep_deref` will return that value.
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
    pub fn bind(&mut self, var: &Symbol, val: Term) -> PolarResult<Option<Goal>> {
        use BindingManagerVariableState::*;
        let mut goal = None;
        if let Ok(symbol) = val.value().as_symbol() {
            goal = self.bind_variables(var, symbol)?;
        } else {
            match self._variable_state(var) {
                Partial(p) => {
                    let p = p.clone();
                    let val = val.clone();
                    goal = Some(self.partial_bind(p, var, val)?)
                }

                Bound(_) => {
                    return Err(RuntimeError::IncompatibleBindings {
                        msg: format!("Cannot rebind {:?}", var),
                    }
                    .into())
                }
                _ => self.add_binding(var, val.clone()),
            }
        }

        // If the main binding succeeded, the follower binding must succeed.
        self.do_followers(|_, follower| {
            follower.bind(var, val.clone())?;
            Ok(())
        })
        .unwrap();

        Ok(goal)
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
        use BindingManagerVariableState::*;
        assert!(matches!(self._variable_state(var), Unbound | Bound(_)));
        self.add_binding(var, val);
    }

    /// Add a constraint. Constraints are represented as term expressions.
    ///
    /// `term` must be an expression`.
    ///
    /// An error is returned if the constraint is incompatible with existing constraints.
    pub fn add_constraint(&mut self, term: &Term) -> PolarResult<()> {
        use BindingManagerVariableState::*;
        self.do_followers(|_, follower| follower.add_constraint(term))?;

        assert!(term.value().as_expression().is_ok());
        let mut op = op!(And, term.clone());

        // include all constraints applying to any of its variables.
        for var in op.variables().iter().rev() {
            match self._variable_state(var) {
                Cycle(c) => op = cycle_constraints(c).merge_constraints(op),
                Partial(e) => op = e.clone().merge_constraints(op),
                _ => {}
            }
        }

        let vars = op.variables();
        let mut varset = vars.iter().collect::<HashSet<_>>();

        // replace any bound variables with their values.
        for var in vars.iter() {
            if let Bound(val) = self._variable_state(var) {
                varset.remove(var);
                match op.ground(var, val) {
                    Some(o) => op = o,
                    None => {
                        return Err(RuntimeError::IncompatibleBindings {
                            msg: "Grounding failed B".into(),
                        }
                        .into())
                    }
                }
            }
        }

        // apply the new constraint to every remaining variable.
        for var in varset {
            self.add_binding(var, op.clone().into())
        }
        Ok(())
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
    /// Dereference all variables in term, including within nested structures like
    /// lists and dictionaries.
    pub fn deep_deref(&self, term: &Term) -> Term {
        Derefer::new(self).fold_term(term.clone())
    }

    /// Get constraints on variable `variable`. If the variable is in a cycle,
    /// the cycle is expressed as a partial.
    pub fn get_constraints(&self, variable: &Symbol) -> Operation {
        use BindingManagerVariableState::*;
        match self._variable_state(variable) {
            Unbound => op!(And),
            Bound(val) => op!(And, term!(op!(Unify, term!(variable.clone()), val))),
            Partial(expr) => expr.clone(),
            Cycle(c) => cycle_constraints(c),
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
    fn bind_variables(&mut self, left: &Symbol, right: &Symbol) -> PolarResult<Option<Goal>> {
        use BindingManagerVariableState::*;
        match (
            // rebinding the variable with its state makes it possible
            // to handle symmetric cases in one branch
            (left, self._variable_state(left)),
            (right, self._variable_state(right)),
        ) {
            // same variables, do nothing
            _ if left == right => Ok(None),

            // free / cycle cases -- variables are unbound or bound only bound
            // to other variables

            // free x free --  create a pair of bindings var -> var
            ((_, Unbound), (_, Unbound)) => {
                self.add_binding(left, term!(right.clone()));
                self.add_binding(right, term!(left.clone()));
                Ok(None)
            }

            // free x cycle, cycle x free -- create a pair of bindings var -> var
            ((var, Unbound), (cvar, Cycle(cycle))) | ((cvar, Cycle(cycle)), (var, Unbound)) => {
                let last = cycle.last().unwrap();
                assert_ne!(last, cvar);
                self.add_binding(last, term!(var.clone()));
                self.add_binding(var, term!(cvar.clone()));
                Ok(None)
            }

            // cycle x cycle -- two cases
            ((_, Cycle(left_cycle)), (_, Cycle(right_cycle))) => {
                let iter_left = left_cycle.iter().collect::<HashSet<&Symbol>>();
                let iter_right = right_cycle.iter().collect::<HashSet<&Symbol>>();

                // already the same cycle? then do nothing
                if iter_left.intersection(&iter_right).next().is_some() {
                    assert_eq!(iter_left, iter_right);
                // else join them with a pair of bindings var -> var
                } else {
                    let last_left = left_cycle.last().unwrap();
                    let last_right = right_cycle.last().unwrap();
                    assert_ne!(last_left, left);
                    assert_ne!(last_right, right);
                    self.add_binding(last_left, term!(right.clone()));
                    self.add_binding(last_right, term!(left.clone()));
                }
                Ok(None)
            }

            // bound / partial cases -- at least one variable has a value
            // or constraint
            //
            // bound x free , free x bound, bound x cycle , cycle x bound --
            // create a binding var -> val
            ((var, Unbound), (_, Bound(val)))
            | ((_, Bound(val)), (var, Unbound))
            | ((var, Cycle(_)), (_, Bound(val)))
            | ((_, Bound(val)), (var, Cycle(_))) => {
                self.add_binding(var, val);
                Ok(None)
            }

            // partial x free, free x partial, partial x cycle, cycle x partial --
            // extend partials
            ((_, Partial(_)), (_, Unbound))
            | ((_, Unbound), (_, Partial(_)))
            | ((_, Partial(_)), (_, Cycle(_)))
            | ((_, Cycle(_)), (_, Partial(_))) => {
                self.add_constraint(&op!(Unify, term!(left.clone()), term!(right.clone())).into())?;
                Ok(None)
            }

            // bound x bound to different values : binding fails
            // (this error usually gets caught and turned into a backtrack)
            ((_, Bound(l)), (_, Bound(r))) => {
                if l == r {
                    Ok(None)
                } else {
                    Err(RuntimeError::IncompatibleBindings {
                        msg: format!("{} and {} are both bound", left, right),
                    }
                    .into())
                }
            }

            // bound x partial , partial x bound -- ground and requery
            ((_, Bound(val)), (var, Partial(p))) | ((var, Partial(p)), (_, Bound(val))) => {
                let p = p.clone();
                Ok(Some(self.partial_bind(p, var, val)?))
            }

            // partial x partial -- if they already overlap, do nothing.
            // else rebind vars as a 2-cycle & requery
            ((lv, Partial(lp)), (rv, Partial(rp))) => {
                if rp.variables().contains(left) {
                    Ok(None)
                } else {
                    // Merge the two partials.
                    let merged = lp.clone().merge_constraints(rp.clone());

                    // Express the partial in terms of lv (bind rv to lv, replacing all rv in partial with lv).
                    let goal = self.partial_bind(merged, rv, term!(lv.clone()))?;

                    // Unification from lv = rv (remember that vars are equal so that the
                    // simplifier can later choose the correct one). We do this
                    // after the partial bind so that we don't recursively query the unification.
                    let unify = term!(op!(Unify, term!(lv.clone()), term!(rv.clone())));
                    self.add_constraint(&unify)?;
                    Ok(Some(goal))
                }
            }
        }
    }

    fn add_binding(&mut self, var: &Symbol, val: Term) {
        self.bindings.push(Binding(var.clone(), val));
    }

    fn lookup(&self, var: &Symbol) -> Option<Term> {
        match self.variable_state(var) {
            VariableState::Bound(val) => Some(val),
            _ => None,
        }
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
        use BindingManagerVariableState::*;
        let index = bsp.bindings_index;
        let mut path = vec![variable];
        while let Some(value) = self.value(path.last().unwrap(), index) {
            match value.value() {
                Value::Expression(e) => return Partial(e),
                Value::Variable(v) | Value::RestVariable(v) => {
                    if v == variable {
                        return Cycle(path.into_iter().cloned().collect());
                    } else {
                        path.push(v);
                    }
                }
                _ => return Bound(value.clone()),
            }
        }
        Unbound
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
            assert_eq!(p.to_string(), "x = y and y = z and z = x and x > y");
        } else {
            panic!("unexpected");
        }

        if let BindingManagerVariableState::Partial(p) = b2._variable_state(&sym!("x")) {
            assert_eq!(p.to_string(), "x > y");
        } else {
            panic!("unexpected");
        }
    }

    #[test]
    fn old_deref() {
        let mut bm = BindingManager::default();
        let value = term!(1);
        let x = sym!("x");
        let y = sym!("y");
        let term_x = term!(x.clone());
        let term_y = term!(y.clone());

        // unbound var
        assert_eq!(bm.deep_deref(&term_x), term_x);

        // unbound var -> unbound var
        bm.bind(&x, term_y.clone()).unwrap();
        assert_eq!(bm.deep_deref(&term_x), term_x);

        // value
        assert_eq!(bm.deep_deref(&value), value.clone());

        // unbound var -> value
        let mut bm = BindingManager::default();
        bm.bind(&x, value.clone()).unwrap();
        assert_eq!(bm.deep_deref(&term_x), value);

        // unbound var -> unbound var -> value
        let mut bm = BindingManager::default();
        bm.bind(&x, term_y).unwrap();
        bm.bind(&y, value.clone()).unwrap();
        assert_eq!(bm.deep_deref(&term_x), value);
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
