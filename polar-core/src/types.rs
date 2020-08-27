//! # Types
//!
//! Polar types

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::sources::*;
use super::terms::*;

pub use super::{error, formatting::ToPolarString};

/// A map of bindings: variable name → value. The VM uses a stack internally,
/// but can translate to and from this type.
pub type Bindings = HashMap<Symbol, Term>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Parameter {
    pub parameter: Term,
    pub specializer: Option<Term>,
}

impl Parameter {
    pub fn map_replace<F>(&mut self, f: &mut F)
    where
        F: FnMut(&Term) -> Term,
    {
        self.parameter.map_replace(f);
        self.specializer.iter_mut().for_each(|p| p.map_replace(f));
    }

    pub fn is_ground(&self) -> bool {
        self.specializer.is_none() && self.parameter.value().is_ground()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    pub name: Symbol,
    pub params: Vec<Parameter>,
    pub body: Term,
}

impl Rule {
    pub fn map_replace<F>(&mut self, f: &mut F)
    where
        F: FnMut(&Term) -> Term,
    {
        self.params.iter_mut().for_each(|p| p.map_replace(f));
        self.body.map_replace(f);
    }

    pub fn is_ground(&self) -> bool {
        self.params.iter().all(|p| p.is_ground())
    }
}

pub type Rules = Vec<Arc<Rule>>;

type RuleSet = BTreeSet<u64>;

#[derive(Clone, Default, Debug)]
struct RuleIndex {
    rules: RuleSet,
    index: HashMap<Option<Value>, RuleIndex>,
}

impl RuleIndex {
    pub fn index_rule(&mut self, rule_id: u64, params: &[Parameter], i: usize) {
        if i < params.len() {
            self.index
                .entry({
                    if params[i].is_ground() {
                        Some(params[i].parameter.value().clone())
                    } else {
                        None
                    }
                })
                .or_insert_with(RuleIndex::default)
                .index_rule(rule_id, params, i + 1);
        } else {
            self.rules.insert(rule_id);
        }
    }

    #[allow(clippy::comparison_chain)]
    pub fn get_applicable_rules(&self, args: &[Term], i: usize) -> RuleSet {
        if i < args.len() {
            // Check this argument and recurse on the rest.
            let filter_next_args =
                |index: &RuleIndex| -> RuleSet { index.get_applicable_rules(args, i + 1) };
            let arg = args[i].value();
            if arg.is_ground() {
                // Check the index for a ground argument.
                let mut ruleset = self
                    .index
                    .get(&Some(arg.clone()))
                    .map(|index| filter_next_args(index))
                    .unwrap_or_else(RuleSet::default);

                // Extend for a variable parameter.
                if let Some(index) = self.index.get(&None) {
                    ruleset.extend(filter_next_args(index));
                }
                ruleset
            } else {
                // Accumulate all indexed arguments.
                self.index.values().fold(
                    RuleSet::default(),
                    |mut result: RuleSet, index: &RuleIndex| {
                        result.extend(filter_next_args(index).into_iter());
                        result
                    },
                )
            }
        } else {
            // No more arguments.
            self.rules.clone()
        }
    }
}

#[derive(Clone)]
pub struct GenericRule {
    pub name: Symbol,
    rules: HashMap<u64, Arc<Rule>>,
    index: RuleIndex,
    next_rule_id: u64,
}

impl GenericRule {
    pub fn new(name: Symbol, rules: Rules) -> Self {
        let mut generic_rule = Self {
            name,
            rules: Default::default(),
            index: Default::default(),
            next_rule_id: 0,
        };

        for rule in rules {
            generic_rule.add_rule(rule);
        }

        generic_rule
    }

    pub fn add_rule(&mut self, rule: Arc<Rule>) {
        let rule_id = self.next_rule_id();

        assert!(
            self.rules.insert(rule_id, rule.clone()).is_none(),
            "Rule id already used."
        );
        self.index.index_rule(rule_id, &rule.params[..], 0);
    }

    #[allow(clippy::ptr_arg)]
    pub fn get_applicable_rules(&self, args: &TermList) -> Rules {
        self.index
            .get_applicable_rules(&args, 0)
            .iter()
            .map(|id| self.rules.get(id).expect("Rule missing"))
            .cloned()
            .collect()
    }

    fn next_rule_id(&mut self) -> u64 {
        let v = self.next_rule_id;
        self.next_rule_id += 1;
        v
    }
}

#[derive(Clone)]
pub enum Type {
    Class { name: Symbol },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Rule(Arc<Rule>),
    Term(Term),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    pub node: Node,
    pub children: Vec<Rc<Trace>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceResult {
    pub trace: Rc<Trace>,
    pub formatted: String,
}

#[derive(Default)]
pub struct KnowledgeBase {
    pub constants: Bindings,
    pub types: HashMap<Symbol, Type>,
    pub rules: HashMap<Symbol, GenericRule>,
    pub sources: Sources,
    // For symbols returned from gensym
    gensym_counter: AtomicU64,
    // For call IDs, instance IDs, symbols, etc.
    id_counter: AtomicU64,
    pub inline_queries: Vec<Term>,
}

const MAX_ID: u64 = (1 << 53) - 1;

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
            types: HashMap::new(),
            rules: HashMap::new(),
            sources: Sources::default(),
            id_counter: AtomicU64::new(1),
            gensym_counter: AtomicU64::new(1),
            inline_queries: vec![],
        }
    }

    /// Return a monotonically increasing integer ID.
    ///
    /// Wraps around at 52 bits of precision so that it can be safely coerced to an IEEE-754
    /// double-float (f64).
    pub fn new_id(&self) -> u64 {
        if self
            .id_counter
            .compare_and_swap(MAX_ID, 1, Ordering::SeqCst)
            == MAX_ID
        {
            MAX_ID
        } else {
            self.id_counter.fetch_add(1, Ordering::SeqCst)
        }
    }

    /// Generate a new symbol.
    pub fn gensym(&self, prefix: &str) -> Symbol {
        let next = self.gensym_counter.fetch_add(1, Ordering::SeqCst);
        if prefix == "_" {
            Symbol(format!("_{}", next))
        } else if prefix.starts_with('_') {
            Symbol(format!("{}_{}", prefix, next))
        } else {
            Symbol(format!("_{}_{}", prefix, next))
        }
    }

    /// Add a generic rule to the knowledge base.
    #[cfg(test)]
    pub fn add_generic_rule(&mut self, rule: GenericRule) {
        self.rules.insert(rule.name.clone(), rule);
    }

    /// Define a constant variable.
    pub fn constant(&mut self, name: Symbol, value: Term) {
        self.constants.insert(name, value);
    }

    /// Return true if a constant with the given name has been defined.
    pub fn is_constant(&self, name: &Symbol) -> bool {
        self.constants.contains_key(name)
    }
}

#[allow(clippy::large_enum_variant)]
#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryEvent {
    None,
    Debug {
        message: String,
    },

    Done,

    MakeExternal {
        instance_id: u64,
        constructor: Term,
    },

    ExternalCall {
        /// Persistent id across all requests for results from the same external call.
        call_id: u64,
        /// The external instance to make this call on. None for functions or constructors.
        instance: Option<Term>,
        /// Field name to lookup or method name to call. A class name indicates a constructor
        /// should be called.
        attribute: Symbol,
        /// List of arguments to use if this is a method call.
        args: Option<Vec<Term>>,
    },

    /// Checks if the instance is an instance of (a subclass of) the class_tag.
    ExternalIsa {
        call_id: u64,
        instance: Term,
        class_tag: Symbol,
    },

    /// Checks if the instance is more specifically and instance/subclass of A than B.
    ExternalIsSubSpecializer {
        call_id: u64,
        instance_id: u64,
        left_class_tag: Symbol,
        right_class_tag: Symbol,
    },

    /// Unifies two external instances.
    ExternalUnify {
        call_id: u64,
        left_instance_id: u64,
        right_instance_id: u64,
    },

    Result {
        bindings: Bindings,
        trace: Option<TraceResult>,
    },

    ExternalOp {
        call_id: u64,
        operator: Operator,
        args: TermList,
    },
}

pub fn unwrap_and(term: Term) -> TermList {
    match term.value() {
        Value::Expression(Operation {
            operator: Operator::And,
            args,
        }) => args.clone(),
        _ => vec![term.clone()],
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::polar::Polar;

    #[test]
    fn serialize_test() {
        let pred = Predicate {
            name: Symbol("foo".to_owned()),
            args: vec![Term::new_from_test(value!(0))],
        };
        assert_eq!(
            serde_json::to_string(&pred).unwrap(),
            r#"{"name":"foo","args":[{"value":{"Number":{"Integer":0}}}]}"#
        );
        let event = QueryEvent::ExternalCall {
            call_id: 2,
            instance: None,
            attribute: Symbol::new("foo"),
            args: Some(vec![
                Term::new_from_test(value!(0)),
                Term::new_from_test(value!("hello")),
            ]),
        };
        eprintln!("{}", serde_json::to_string(&event).unwrap());
        let term = Term::new_from_test(value!(1));
        eprintln!("{}", serde_json::to_string(&term).unwrap());
        let mut fields = BTreeMap::new();
        fields.insert(Symbol::new("hello"), term!(1234));
        fields.insert(
            Symbol::new("world"),
            Term::new_from_test(Value::String("something".to_owned())),
        );
        let literal = InstanceLiteral {
            tag: Symbol::new("Foo"),
            fields: Dictionary { fields },
        };
        let event = QueryEvent::MakeExternal {
            instance_id: 12345,
            constructor: Term::new_from_test(Value::InstanceLiteral(literal)),
        };
        eprintln!("{}", serde_json::to_string(&event).unwrap());
        let external = Term::new_from_test(Value::ExternalInstance(ExternalInstance {
            instance_id: 12345,
            constructor: None,
            repr: None,
        }));
        let list_of = Term::new_from_test(Value::List(vec![external]));
        eprintln!("{}", serde_json::to_string(&list_of).unwrap());
        let mut fields = BTreeMap::new();
        fields.insert(Symbol::new("foo"), list_of);
        let dict = Term::new_from_test(Value::Dictionary(Dictionary { fields }));
        eprintln!("{}", serde_json::to_string(&dict).unwrap());
        let e = error::ParseError::InvalidTokenCharacter {
            token: "Integer".to_owned(),
            c: 'x',
            loc: 99,
        };
        let err: crate::error::PolarError = e.into();
        eprintln!("{}", serde_json::to_string(&err).unwrap());
        let rule = Rule {
            name: Symbol::new("foo"),
            params: vec![],
            body: Term::new_temporary(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![dict.clone(), dict.clone(), dict],
            })),
        };
        eprintln!("{}", rule);
    }

    #[test]
    fn test_id_wrapping() {
        let kb = KnowledgeBase::new();
        kb.id_counter.store(MAX_ID - 1, Ordering::SeqCst);
        assert_eq!(MAX_ID - 1, kb.new_id());
        assert_eq!(MAX_ID, kb.new_id());
        assert_eq!(1, kb.new_id());
        assert_eq!(2, kb.new_id());
    }

    #[test]
    fn test_value_hash() {
        let mut table = HashMap::new();
        table.insert(value!(0), "0");
        table.insert(value!(1), "1");
        table.insert(value!("one"), "one");
        table.insert(value!(btreemap! {sym!("a") => term!(1)}), "a:1");
        table.insert(value!(btreemap! {sym!("b") => term!(2)}), "b:2");
        assert_eq!(*table.get(&value!(0)).unwrap(), "0");
        assert_eq!(*table.get(&value!(1)).unwrap(), "1");
        assert_eq!(*table.get(&value!(1.0)).unwrap(), "1");
        assert_eq!(*table.get(&value!("one")).unwrap(), "one");
        assert_eq!(
            *table
                .get(&value!(btreemap! {sym!("a") => term!(1)}))
                .unwrap(),
            "a:1"
        );
        assert_eq!(
            *table
                .get(&value!(btreemap! {sym!("b") => term!(2)}))
                .unwrap(),
            "b:2"
        );
    }

    #[test]
    fn test_rule_index() {
        let polar = Polar::new();
        polar.load(r#"f(1, 1, "x");"#).unwrap();
        polar.load(r#"f(1, 1, "y");"#).unwrap();
        polar.load(r#"f(1, x, "y") if x = 2;"#).unwrap();
        polar.load(r#"f(1, 2, {b: "y"});"#).unwrap();
        polar.load(r#"f(1, 3, {c: "z"});"#).unwrap();

        // Test the index itself.
        let kb = polar.kb.read().unwrap();
        let generic_rule = kb.rules.get(&sym!("f")).unwrap();
        let index = &generic_rule.index;
        assert!(index.rules.is_empty());

        fn keys(index: &RuleIndex) -> HashSet<Option<Value>> {
            index.index.keys().cloned().collect()
        }

        let mut args = HashSet::<Option<Value>>::new();

        args.clear();
        args.insert(Some(value!(1)));
        assert_eq!(args, keys(index));

        args.clear();
        args.insert(None); // x
        args.insert(Some(value!(1)));
        args.insert(Some(value!(2)));
        args.insert(Some(value!(3)));
        let index1 = index.index.get(&Some(value!(1))).unwrap();
        assert_eq!(args, keys(index1));

        args.clear();
        args.insert(Some(value!("x")));
        args.insert(Some(value!("y")));
        let index11 = index1.index.get(&Some(value!(1))).unwrap();
        assert_eq!(args, keys(index11));

        args.remove(&Some(value!("x")));
        let index1_ = index1.index.get(&None).unwrap();
        assert_eq!(args, keys(index1_));

        args.clear();
        args.insert(Some(value!(btreemap! {sym!("b") => term!("y")})));
        let index12 = index1.index.get(&Some(value!(2))).unwrap();
        assert_eq!(args, keys(index12));

        args.clear();
        args.insert(Some(value!(btreemap! {sym!("c") => term!("z")})));
        let index13 = index1.index.get(&Some(value!(3))).unwrap();
        assert_eq!(args, keys(index13));
    }
}
