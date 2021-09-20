use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::bindings::Bindings;
use crate::folder::{fold_term, Folder};
use crate::formatting::ToPolarString;
use crate::terms::*;

use super::partial::{invert_operation, FALSE, TRUE};

/// Set to `true` to debug performance in simplifier by turning on
/// performance counters.
const TRACK_PERF: bool = false;

/// Set to `true` to turn on simplify debug logging.
const SIMPLIFY_DEBUG: bool = false;

macro_rules! if_debug {
    ($($e:tt)*) => {
        if SIMPLIFY_DEBUG {
            $($e)*
        }
    }
}

macro_rules! simplify_debug {
    ($($e:tt)*) => {
        if_debug!(eprintln!($($e)*))
    }
}

enum MaybeDrop {
    Keep,
    Drop,
    Bind(Symbol, Term),
    Check(Symbol, Term),
}

struct VariableSubber {
    this_var: Symbol,
}

impl VariableSubber {
    pub fn new(this_var: Symbol) -> Self {
        Self { this_var }
    }
}

impl Folder for VariableSubber {
    fn fold_variable(&mut self, v: Symbol) -> Symbol {
        if v == self.this_var {
            sym!("_this")
        } else {
            v
        }
    }

    fn fold_rest_variable(&mut self, v: Symbol) -> Symbol {
        if v == self.this_var {
            sym!("_this")
        } else {
            v
        }
    }
}

/// Substitute `sym!("_this")` for a variable in a partial.
pub fn sub_this(this: Symbol, term: Term) -> Term {
    if term
        .value()
        .as_symbol()
        .map(|s| s == &this)
        .unwrap_or(false)
    {
        return term;
    }
    fold_term(term, &mut VariableSubber::new(this))
}

/// Turn `_this = x` into `x` when it's ground.
fn simplify_trivial_constraint(this: Symbol, term: Term) -> Term {
    match term.value() {
        Value::Expression(o) if o.operator == Operator::Unify => {
            let left = &o.args[0];
            let right = &o.args[1];
            match (left.value(), right.value()) {
                (Value::Variable(v), Value::Variable(w))
                | (Value::Variable(v), Value::RestVariable(w))
                | (Value::RestVariable(v), Value::Variable(w))
                | (Value::RestVariable(v), Value::RestVariable(w))
                    if v == &this && w == &this =>
                {
                    TRUE.into()
                }
                (Value::Variable(l), _) | (Value::RestVariable(l), _)
                    if l == &this && right.is_ground() =>
                {
                    right.clone()
                }
                (_, Value::Variable(r)) | (_, Value::RestVariable(r))
                    if r == &this && left.is_ground() =>
                {
                    left.clone()
                }
                _ => term,
            }
        }
        _ => term,
    }
}

pub fn simplify_partial(
    var: &Symbol,
    mut term: Term,
    output_vars: HashSet<Symbol>,
    track_performance: bool,
) -> (Term, Option<PerfCounters>) {
    let mut simplifier = Simplifier::new(output_vars, track_performance);
    simplify_debug!("*** simplify partial {:?}", var);
    simplifier.simplify_partial(&mut term);
    term = simplify_trivial_constraint(var.clone(), term);
    simplify_debug!("simplify partial done {:?}, {:?}", var, term.to_polar());
    if matches!(term.value(), Value::Expression(e) if e.operator != Operator::And) {
        (op!(And, term).into(), simplifier.perf_counters())
    } else {
        (term, simplifier.perf_counters())
    }
}

/// Simplify the values of the bindings to be returned to the host language.
///
/// - For partials, simplify the constraint expressions.
/// - For non-partials, deep deref. TODO(ap/gj): deep deref.
pub fn simplify_bindings(bindings: Bindings, all: bool) -> Option<Bindings> {
    let mut perf = PerfCounters::new(TRACK_PERF);
    simplify_debug!("simplify bindings");

    if_debug! {
        eprintln!("before simplified");
        for (k, v) in bindings.iter() {
            eprintln!("{:?} {:?}", k, v.to_polar());
        }
    }

    let mut unsatisfiable = false;
    let mut simplify_var = |bindings: &Bindings, var: &Symbol, value: &Term| match value.value() {
        Value::Expression(o) => {
            assert_eq!(o.operator, Operator::And);
            let output_vars = if all {
                let mut hs = HashSet::with_capacity(1);
                hs.insert(var.clone());
                hs
            } else {
                bindings
                    .keys()
                    .filter(|v| !v.is_temporary_var())
                    .cloned()
                    .collect::<HashSet<_>>()
            };

            let (simplified, p) = simplify_partial(var, value.clone(), output_vars, TRACK_PERF);
            if let Some(p) = p {
                perf.merge(p);
            }

            match simplified.value().as_expression() {
                Ok(o) if o == &FALSE => unsatisfiable = true,
                _ => (),
            }
            simplified
        }
        Value::Variable(v) | Value::RestVariable(v)
            if v.is_temporary_var()
                && bindings.contains_key(v)
                && matches!(
                    bindings[v].value(),
                    Value::Variable(_) | Value::RestVariable(_)
                ) =>
        {
            bindings[v].clone()
        }
        _ => value.clone(),
    };

    simplify_debug!("simplify bindings {}", if all { "all" } else { "not all" });

    let mut simplified_bindings = HashMap::new();
    for (var, value) in &bindings {
        if !var.is_temporary_var() || all {
            let simplified = simplify_var(&bindings, var, value);
            simplified_bindings.insert(var.clone(), simplified);
        }
    }

    if unsatisfiable {
        None
    } else {
        if_debug! {
            eprintln!("after simplified");
            for (k, v) in simplified_bindings.iter() {
                eprintln!("{:?} {:?}", k, v.to_polar());
            }
        }

        Some(simplified_bindings)
    }
}

#[derive(Clone, Default)]
pub struct PerfCounters {
    enabled: bool,

    // Map of number simplifier loops by term to simplify.
    simplify_term: HashMap<Term, u64>,
    preprocess_and: HashMap<Term, u64>,

    acc_simplify_term: u64,
    acc_preprocess_and: u64,
}

impl fmt::Display for PerfCounters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "perf {{")?;
        writeln!(f, "simplify term")?;
        for (term, ncalls) in self.simplify_term.iter() {
            writeln!(f, "\t{}: {}", term.to_polar(), ncalls)?;
        }

        writeln!(f, "preprocess and")?;

        for (term, ncalls) in self.preprocess_and.iter() {
            writeln!(f, "\t{}: {}", term.to_polar(), ncalls)?;
        }

        writeln!(f, "}}")
    }
}

impl PerfCounters {
    fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Default::default()
        }
    }

    fn preprocess_and(&mut self) {
        if !self.enabled {
            return;
        }

        self.acc_preprocess_and += 1;
    }

    fn simplify_term(&mut self) {
        if !self.enabled {
            return;
        }

        self.acc_simplify_term += 1;
    }

    fn finish_acc(&mut self, term: Term) {
        if !self.enabled {
            return;
        }

        self.simplify_term
            .insert(term.clone(), self.acc_simplify_term);
        self.preprocess_and.insert(term, self.acc_preprocess_and);
        self.acc_preprocess_and = 0;
        self.acc_simplify_term = 0;
    }

    fn merge(&mut self, other: PerfCounters) {
        if !self.enabled {
            return;
        }

        self.simplify_term.extend(other.simplify_term.into_iter());
        self.preprocess_and.extend(other.preprocess_and.into_iter());
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone)]
pub struct Simplifier {
    bindings: Bindings,
    output_vars: HashSet<Symbol>,
    seen: HashSet<Term>,

    counters: PerfCounters,
}

type TermSimplifier = dyn Fn(&mut Simplifier, &mut Term);

impl Simplifier {
    pub fn new(output_vars: HashSet<Symbol>, track_performance: bool) -> Self {
        Self {
            bindings: Bindings::new(),
            output_vars,
            seen: HashSet::new(),
            counters: PerfCounters::new(track_performance),
        }
    }

    fn perf_counters(&mut self) -> Option<PerfCounters> {
        if !self.counters.is_enabled() {
            return None;
        }

        let mut counter = PerfCounters::new(true);
        std::mem::swap(&mut self.counters, &mut counter);
        Some(counter)
    }

    pub fn bind(&mut self, var: Symbol, value: Term) {
        // We do not allow rebindings.
        if !self.is_bound(&var) {
            self.bindings.insert(var, self.deref(&value));
        }
    }

    pub fn deref(&self, term: &Term) -> Term {
        match term.value() {
            Value::Variable(var) | Value::RestVariable(var) => {
                self.bindings.get(var).unwrap_or(term).clone()
            }
            _ => term.clone(),
        }
    }

    fn is_bound(&self, var: &Symbol) -> bool {
        self.bindings.contains_key(var)
    }

    fn is_output(&self, t: &Term) -> bool {
        match t.value() {
            Value::Variable(v) | Value::RestVariable(v) => self.output_vars.contains(v),
            _ => false,
        }
    }

    /// Determine whether to keep, drop, bind or conditionally bind a unification operation.
    ///
    /// Returns:
    /// - Keep: to indicate that the operation should not be removed
    /// - Drop: to indicate the operation should be removed with no new bindings
    /// - Bind(var, val) to indicate that the operation should be removed, and var should be
    ///                  bound to val.
    /// - Check(var, val) To indicate that the operation should be removed and var should
    ///                   be bound to val *if* var is referenced elsewhere in the expression.
    ///
    /// Params:
    ///     constraint: The constraint to consider removing from its parent.
    fn maybe_bind_constraint(&mut self, constraint: &Operation) -> MaybeDrop {
        match constraint.operator {
            // X and X is always true, so drop.
            Operator::And if constraint.args.is_empty() => MaybeDrop::Drop,

            // Choose a unification to maybe drop.
            Operator::Unify | Operator::Eq => {
                let left = &constraint.args[0];
                let right = &constraint.args[1];

                if left == right {
                    // The sides are exactly equal, so drop.
                    MaybeDrop::Drop
                } else {
                    // Maybe bind one side to the other.
                    match (left.value(), right.value()) {
                        // Always keep unifications of two output variables (x = y).
                        (Value::Variable(_), Value::Variable(_))
                            if self.is_output(left) && self.is_output(right) =>
                        {
                            MaybeDrop::Keep
                        }
                        // Replace non-output variable l with right.
                        (Value::Variable(l), _) if !self.is_bound(l) && !self.is_output(left) => {
                            simplify_debug!("*** 1");
                            MaybeDrop::Bind(l.clone(), right.clone())
                        }
                        // Replace non-output variable r with left.
                        (_, Value::Variable(r)) if !self.is_bound(r) && !self.is_output(right) => {
                            simplify_debug!("*** 2");
                            MaybeDrop::Bind(r.clone(), left.clone())
                        }
                        // Replace unbound variable with ground value.
                        (Value::Variable(var), val) if val.is_ground() && !self.is_bound(var) => {
                            simplify_debug!("*** 3");
                            MaybeDrop::Check(var.clone(), right.clone())
                        }
                        // Replace unbound variable with ground value.
                        (val, Value::Variable(var)) if val.is_ground() && !self.is_bound(var) => {
                            simplify_debug!("*** 4");
                            MaybeDrop::Check(var.clone(), left.clone())
                        }
                        // Keep everything else.
                        _ => MaybeDrop::Keep,
                    }
                }
            }
            _ => MaybeDrop::Keep,
        }
    }

    /// Perform simplification of variable names in an operation by eliminating unification
    /// operations to express an operation in terms of output variables only.
    ///
    /// Also inverts negation operations.
    ///
    /// May require multiple calls to perform all eliminiations.
    pub fn simplify_operation_variables(
        &mut self,
        o: &mut Operation,
        simplify_term: &TermSimplifier,
    ) {
        fn toss_trivial_unifies(args: &mut TermList) {
            args.retain(|c| {
                let o = c.value().as_expression().unwrap();
                match o.operator {
                    Operator::Unify | Operator::Eq => {
                        assert_eq!(o.args.len(), 2);
                        let left = &o.args[0];
                        let right = &o.args[1];
                        left != right
                    }
                    _ => true,
                }
            });
        }

        if o.operator == Operator::And || o.operator == Operator::Or {
            toss_trivial_unifies(&mut o.args);
        }

        match o.operator {
            // Zero-argument conjunctions & disjunctions represent constants
            // TRUE and FALSE, respectively. We do not simplify them.
            Operator::And | Operator::Or if o.args.is_empty() => (),

            // Replace one-argument conjunctions & disjunctions with their argument.
            Operator::And | Operator::Or if o.args.len() == 1 => {
                if let Value::Expression(operation) = o.args[0].value() {
                    *o = operation.clone();
                    self.simplify_operation_variables(o, simplify_term);
                }
            }

            // Non-trivial conjunctions. Choose unification constraints
            // to make bindings from and throw away; fold the rest.
            Operator::And if o.args.len() > 1 => {
                // Compute which constraints to keep.
                let mut keep = o.args.iter().map(|_| true).collect::<Vec<bool>>();
                let mut references = o.args.iter().map(|_| false).collect::<Vec<bool>>();
                for (i, arg) in o.args.iter().enumerate() {
                    match self.maybe_bind_constraint(arg.value().as_expression().unwrap()) {
                        MaybeDrop::Keep => (),
                        MaybeDrop::Drop => keep[i] = false,
                        MaybeDrop::Bind(var, value) => {
                            keep[i] = false;
                            simplify_debug!("bind {:?}, {:?}", var, value.to_polar());
                            self.bind(var, value);
                        }
                        MaybeDrop::Check(var, value) => {
                            simplify_debug!("check {:?}, {:?}", var.to_polar(), value.to_polar());
                            for (j, arg) in o.args.iter().enumerate() {
                                if j != i && arg.contains_variable(&var) {
                                    simplify_debug!(
                                        "check bind {:?}, {:?} ref: {}",
                                        var.to_polar(),
                                        value.to_polar(),
                                        j
                                    );
                                    self.bind(var, value);
                                    keep[i] = false;

                                    // record that this term references var and must be kept.
                                    references[j] = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                // Drop the rest.
                let mut i = 0;
                o.args.retain(|_| {
                    i += 1;
                    keep[i - 1] || references[i - 1]
                });

                // Simplify the survivors.
                for arg in &mut o.args {
                    simplify_term(self, arg);
                }
            }

            // Negation. Simplify the negated term, saving & restoring the
            // current bindings because bindings may not leak out of a negation.
            Operator::Not => {
                assert_eq!(o.args.len(), 1);
                let mut simplified = o.args[0].clone();
                let mut simplifier = self.clone();
                simplifier.simplify_partial(&mut simplified);
                *o = invert_operation(
                    simplified
                        .value()
                        .as_expression()
                        .expect("a simplified expression")
                        .clone(),
                )
            }

            // Default case.
            _ => {
                for arg in &mut o.args {
                    simplify_term(self, arg);
                }
            }
        }
    }

    /// Deduplicate an operation by removing terms that are mirrors or duplicates
    /// of other terms.
    pub fn deduplicate_operation(&mut self, o: &mut Operation, simplify_term: &TermSimplifier) {
        fn preprocess_and(args: &mut TermList) {
            // HashSet of term hash values used to deduplicate. We use hash values
            // to avoid cloning to insert terms.
            let mut seen: HashSet<u64> = HashSet::with_capacity(args.len());
            args.retain(|a| {
                let o = a.value().as_expression().unwrap();
                o != &TRUE // trivial
                    && !seen.contains(&Term::from(o.mirror()).hash_value()) // reflection
                    && seen.insert(a.hash_value()) // duplicate
            });
        }

        if o.operator == Operator::And {
            self.counters.preprocess_and();
            preprocess_and(&mut o.args);
        }

        match o.operator {
            Operator::And | Operator::Or if o.args.is_empty() => (),

            // Replace one-argument conjunctions & disjunctions with their argument.
            Operator::And | Operator::Or if o.args.len() == 1 => {
                if let Value::Expression(operation) = o.args[0].value() {
                    *o = operation.clone();
                    self.deduplicate_operation(o, simplify_term);
                }
            }

            // Default case.
            _ => {
                for arg in &mut o.args {
                    simplify_term(self, arg);
                }
            }
        }
    }

    /// Simplify a term `term` in place by calling the simplification
    /// function `simplify_operation` on any Expression in that term.
    ///
    /// `simplify_operation` should perform simplification operations in-place
    /// on the operation argument. To recursively simplify sub-terms in that operation,
    /// it must call the passed TermSimplifier.
    pub fn simplify_term<F>(&mut self, term: &mut Term, simplify_operation: F)
    where
        F: Fn(&mut Self, &mut Operation, &TermSimplifier) + 'static + Clone,
    {
        if self.seen.contains(term) {
            return;
        }
        let orig = term.clone();
        self.seen.insert(term.clone());

        let de = self.deref(term);
        *term = de;

        match term.mut_value() {
            Value::Dictionary(dict) => {
                for (_, v) in dict.fields.iter_mut() {
                    self.simplify_term(v, simplify_operation.clone());
                }
            }
            Value::Call(call) => {
                for arg in call.args.iter_mut() {
                    self.simplify_term(arg, simplify_operation.clone());
                }
                if let Some(kwargs) = &mut call.kwargs {
                    for (_, v) in kwargs.iter_mut() {
                        self.simplify_term(v, simplify_operation.clone());
                    }
                }
            }
            Value::List(list) => {
                for elem in list.iter_mut() {
                    self.simplify_term(elem, simplify_operation.clone());
                }
            }
            Value::Expression(operation) => {
                let so = simplify_operation.clone();
                let cont = move |s: &mut Self, term: &mut Term| {
                    s.simplify_term(term, simplify_operation.clone())
                };
                so(self, operation, &cont);
            }
            _ => (),
        }

        if let Ok(sym) = orig.value().as_symbol() {
            if term.contains_variable(sym) {
                *term = orig.clone()
            }
        }
        self.seen.remove(&orig);
    }

    /// Simplify a partial until quiescence.
    pub fn simplify_partial(&mut self, term: &mut Term) {
        // TODO(ap): This does not handle hash collisions.
        let mut last = term.hash_value();
        let mut nbindings = self.bindings.len();
        loop {
            simplify_debug!("simplify loop {:?}", term.to_polar());
            self.counters.simplify_term();

            self.simplify_term(term, Simplifier::simplify_operation_variables);
            let now = term.hash_value();
            if last == now && self.bindings.len() == nbindings {
                break;
            }
            last = now;
            nbindings = self.bindings.len();
        }

        self.simplify_term(term, Simplifier::deduplicate_operation);

        self.counters.finish_acc(term.clone());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Ensure that debug flags are false. Do not remove this test. It is here
    /// to ensure we don't release with debug logs or performance tracking enabled.
    #[test]
    #[allow(clippy::bool_assert_comparison)]
    fn test_debug_off() {
        assert_eq!(SIMPLIFY_DEBUG, false);
        assert_eq!(TRACK_PERF, false);
    }

    #[test]
    fn test_simplify_circular_dot_with_isa() {
        let op = term!(op!(Dot, var!("x"), str!("x")));
        let op = term!(op!(Unify, var!("x"), op));
        let op = term!(op!(
            And,
            op,
            term!(op!(Isa, var!("x"), term!(pattern!(instance!("X")))))
        ));
        let mut vs: HashSet<Symbol> = HashSet::new();
        vs.insert(sym!("x"));
        let (x, _) = simplify_partial(&sym!("x"), op, vs, false);
        assert_eq!(
            x,
            term!(op!(
                And,
                term!(op!(Unify, var!("x"), term!(op!(Dot, var!("x"), str!("x"))))),
                term!(op!(Isa, var!("x"), term!(pattern!(instance!("X")))))
            ))
        );
    }
}
