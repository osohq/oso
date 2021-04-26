use std::fmt;
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

use crate::bindings::Bindings;
use crate::folder::{fold_term, Folder};
use crate::formatting::ToPolarString;
use crate::terms::{Operation, Operator, Partial, Symbol, Term, TermList, Value};

use super::partial::{invert_operation, FALSE, TRUE};

/// Set to `true` to debug performance in simplifier by turning on
/// performance counters.
const TRACK_PERF: bool = false;

/// Set to `true` to turn on simplify debug logging.
const SIMPLIFY_DEBUG: bool = true;

macro_rules! if_debug {
    ($($e:tt)*) => {
        if SIMPLIFY_DEBUG {
            $($e)*
        }
    }
}

fn hash<H: Hash>(h: &H) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    h.hash(&mut hasher);
    hasher.finish()
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
fn simplify_trivial_constraint(this: Symbol, partial: Partial) -> Term {
    if partial.constraints.is_empty() {
        return term!(sym!(this));
    }
    for o in &partial.constraints {
        if o.operator == Operator::Unify {
            let left = &o.args[0];
            let right = &o.args[1];
            match (left.value(), right.value()) {
                (Value::Variable(v), Value::Variable(w))
                | (Value::Variable(v), Value::RestVariable(w))
                | (Value::RestVariable(v), Value::Variable(w))
                | (Value::RestVariable(v), Value::RestVariable(w))
                    if v == &this && w == &this =>
                {
                    return term!(sym!(this));
                }
                (Value::Variable(l), _) | (Value::RestVariable(l), _)
                    if l == &this && right.is_ground() =>
                {
                    // right.clone()
                    panic!("this should have been ground earlier")
                }
                (_, Value::Variable(r)) | (_, Value::RestVariable(r))
                    if r == &this && left.is_ground() =>
                {
                    panic!("this should have been ground earlier")
                    // left.clone()
                }
                _ => {}
            }
        }
    }
    partial.into_term()
}

pub fn simplify_partial(
    var: &Symbol,
    mut partial: Partial,
    mut bindings: Bindings,
    track_performance: bool,
) -> (Term, Option<PerfCounters>) {
    let _ = bindings.remove(var);
    let mut simplifier = Simplifier::new(bindings, track_performance);
    simplify_debug!("*** simplify partial {:?}", var);
    simplifier.simplify_partial(&mut partial);
    let term = simplify_trivial_constraint(var.clone(), partial);
    simplify_debug!("simplify partial done {:?}, {:?}", var, term.to_polar());
    (term, simplifier.perf_counters())
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

    simplify_debug!("simplify bindings {}", if all { "all" } else { "not all" });

    let mut unsatisfiable = false;
    let mut simplified_bindings = HashMap::new();
    for (var, value) in bindings
        .iter()
        .filter(|(var, _value)| !var.is_temporary_var() || all)
    {
        let simplified = match value.value() {
            Value::Partial(p) => {
                let (simplified, p): (Term, _) =
                    simplify_partial(var, p.clone(), bindings.clone(), TRACK_PERF);
                if let Some(p) = p {
                    perf.merge(p);
                }

                if matches!(simplified.value(), Value::Partial(p) if p.is_false()) {
                    unsatisfiable = true;
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
        simplified_bindings.insert(var.clone(), simplified);
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
    counters: PerfCounters,
}

type TermSimplifier = dyn Fn(&mut Simplifier, &mut Term);

impl Simplifier {
    pub fn new(bindings: Bindings, track_performance: bool) -> Self {
        Self {
            bindings,
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
        let new_value = self.deref(&value);
        if self.is_bound(&var) {
            // We do not allow rebindings.
            return;
        }

        self.bindings.insert(var, new_value);
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

    fn is_trivial_operation(&self, o: &Operation) -> bool {
        match o.operator {
            // unify with self
            Operator::Unify | Operator::Eq if o.args[0] == o.args[1] => true,
            Operator::Unify | Operator::Eq | Operator::Neq => {
                // either left or right is unbound, and the other is a dot op
                matches!(o.args[1].value(), Value::Variable(v) if !self.is_bound(v))
                    && matches!(
                        o.args[0].value(),
                        Value::Expression(Operation {
                            operator: Operator::Dot,
                            ..
                        })
                    )
                    || matches!(o.args[0].value(), Value::Variable(v) if !self.is_bound(v))
                        && matches!(
                            o.args[1].value(),
                            Value::Expression(Operation {
                                operator: Operator::Dot,
                                ..
                            })
                        )
            }
            _ => false,
        }
    }

    /// Drop trivial partials, and flatten any nested conjunctions
    ///
    /// Also inverts negation operations.
    fn reduce_constraints(&mut self, p: &mut Partial) {
        struct PartialReducer<'a> {
            simplifier: &'a Simplifier,
        }

        impl<'a> Folder for PartialReducer<'a> {
            fn fold_operation(&mut self, o: Operation) -> Operation {
                simplify_debug!("fold operation: {}", o.to_polar());
                match o.operator {
                    // Zero-argument conjunctions & disjunctions represent constants
                    // TRUE and FALSE, respectively. We do not simplify them.
                    Operator::Or if o.args.is_empty() => o,

                    // Replace one-argument conjunctions & disjunctions with their argument.
                    Operator::Or if o.args.len() == 1 => {
                        if let Value::Expression(operation) = o.args[0].value() {
                            self.fold_operation(operation.clone())
                        } else {
                            o
                        }
                    }

                    // Negation. Simplify the negated term, saving & restoring the
                    // current bindings because bindings may not leak out of a negation.
                    Operator::Not => {
                        assert_eq!(o.args.len(), 1);
                        simplify_debug!("invert op: {}", o.args[0].to_polar());
                        let inner = o.args[0].value().as_expression().unwrap().clone();
                        self.fold_operation(invert_operation(inner))
                    }

                    // Default case.
                    _ => o,
                }
            }
            fn fold_partial(&mut self, p: Partial) -> Partial {
                let mut p = crate::folder::fold_partial(p, self);
                simplify_debug!("folded: {}", p.to_polar());

                // flatten any nested ands
                let mut initial_len = 0;
                while p.constraints.len() != initial_len {
                    initial_len = p.constraints.len();
                    p.constraints = p
                        .constraints
                        .into_iter()
                        .flat_map(|o| match o.operator {
                            Operator::And => o
                                .args
                                .iter()
                                .map(|t| t.value().as_expression().unwrap().clone())
                                .collect(),
                            _ => vec![o],
                        })
                        .collect();
                }
                simplify_debug!("flattened: {}", p.to_polar());
                // toss trivial unifications
                p.constraints
                    .retain(|o| !self.simplifier.is_trivial_operation(o));
                simplify_debug!("reduced: {}", p.to_polar());

                p
            }
        }

        let mut reducer = PartialReducer { simplifier: &self };
        *p = reducer.fold_partial(p.clone());
    }

    /// Deduplicate a partial by removing terms that are mirrors or duplicates
    /// of other terms.
    fn deduplicate_partial(&mut self, p: &mut Partial) {
        // HashSet of term hash values used to deduplicate. We use hash values
        // to avoid cloning to insert terms.
        struct PartialDedup {
            seen: HashSet<u64>,
        }

        impl Folder for PartialDedup {
            fn fold_operation(&mut self, mut o: Operation) -> Operation {
                if o.operator == Operator::And {
                    o.args.retain(|a| {
                        if let Ok(expr) = a.value().as_expression() {
                            let h = hash(expr);
                            expr != &TRUE // trivial
                                && !self.seen.contains(&h) // reflection
                                && self.seen.insert(h) // duplicate
                        } else {
                            true
                        }
                    });
                }
                crate::folder::fold_operation(o, self)
            }
        }

        let mut folder = PartialDedup {
            seen: HashSet::with_capacity(2 * p.constraints.len()),
        };
        for o in p.constraints.iter_mut() {
            *o = folder.fold_operation(o.clone());
        }
    }

    fn deref_partial_variables(&mut self, p: &mut Partial) {
        // Derefs any variables found in expressions,
        // and moves any partially bound variables into
        // the references field
        struct PartialDeref<'a> {
            simplifier: &'a Simplifier,
            references: HashMap<Symbol, Partial>,
        }

        impl<'a> Folder for PartialDeref<'a> {
            fn fold_operation(&mut self, o: Operation) -> Operation {
                simplify_debug!("fold op: {}", o.to_polar());
                let new_args = o
                    .args
                    .iter()
                    .map(|a| {
                        simplify_debug!("map arg: {:?}", a);
                        match a.value() {
                            Value::Variable(var) | Value::RestVariable(var) => {
                                if let Some(value) = self.simplifier.bindings.get(var) {
                                    if let Value::Partial(p) = value.value() {
                                        simplify_debug!("add ref {} -> {}", var, p.to_polar());
                                        self.references.insert(var.clone(), p.clone());
                                        a.clone()
                                    } else {
                                        simplify_debug!(
                                            "partial deref {} -> {}",
                                            var,
                                            value.to_polar()
                                        );
                                        crate::folder::fold_term(value.clone(), self)
                                    }
                                } else {
                                    simplify_debug!("unbound var {}", var);
                                    a.clone()
                                }
                            }
                            _ => crate::folder::fold_term(a.clone(), self),
                        }
                    })
                    .collect();
                Operation {
                    operator: o.operator,
                    args: new_args,
                }
            }
        }

        let mut folder = PartialDeref {
            references: Default::default(),
            simplifier: &self,
        };
        *p = folder.fold_partial(p.clone());
        p.references.extend(folder.references);
    }

    /// Simplify a partial until quiescence.
    pub fn simplify_partial(&mut self, partial: &mut Partial) {
        simplify_debug!("simplify loop {:?}", partial.to_polar());
        self.counters.simplify_term();

        self.deref_partial_variables(partial);
        self.deduplicate_partial(partial);
        self.reduce_constraints(partial);

        self.counters.finish_acc(partial.clone().into_term());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Ensure that debug flags are false. Do not remove this test. It is here
    /// to ensure we don't release with debug logs or performance tracking enabled.
    #[test]
    fn test_debug_off() {
        assert_eq!(SIMPLIFY_DEBUG, false);
        assert_eq!(TRACK_PERF, false);
    }
}
