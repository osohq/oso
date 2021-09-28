use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use crate::sources::SourceInfo;

use super::terms::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Parameter {
    pub parameter: Term,
    pub specializer: Option<Term>,
}

impl Parameter {
    pub fn is_ground(&self) -> bool {
        self.specializer.is_none() && self.parameter.value().is_ground()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub name: Symbol,
    pub params: Vec<Parameter>,
    pub body: Term,
    #[serde(skip, default = "SourceInfo::ffi")]
    pub source_info: SourceInfo,
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.params.len() == other.params.len()
            && self.params == other.params
            && self.body == other.body
    }
}

impl Rule {
    pub fn is_ground(&self) -> bool {
        self.params.iter().all(|p| p.is_ground())
    }

    pub fn span(&self) -> Option<(usize, usize)> {
        if let SourceInfo::Parser { left, right, .. } = self.source_info {
            Some((left, right))
        } else {
            None
        }
    }

    pub fn new_from_test(name: Symbol, params: Vec<Parameter>, body: Term) -> Self {
        Self {
            name,
            params,
            body,
            source_info: SourceInfo::Test,
        }
    }

    /// Creates a new term from the parser
    pub fn new_from_parser(
        src_id: u64,
        left: usize,
        right: usize,
        name: Symbol,
        params: Vec<Parameter>,
        body: Term,
    ) -> Self {
        Self {
            name,
            params,
            body,
            source_info: SourceInfo::Parser {
                src_id,
                left,
                right,
            },
        }
    }
}

// TODO: should this be a Set of Rules? Do we currently check for duplicate rules?
pub struct RuleTypes(HashMap<Symbol, Vec<Rule>>);

impl Default for RuleTypes {
    fn default() -> Self {
        let mut rule_types = Self(HashMap::new());
        rule_types.add_default_rule_types();
        rule_types
    }
}

impl RuleTypes {
    fn add_default_rule_types(&mut self) {
        // type has_permission(actor: Actor, permission: String, resource: Resource);
        self.add(rule!("has_permission", ["actor"; instance!(sym!("Actor")), "_permission"; instance!(sym!("String")), "resource"; instance!(sym!("Resource"))]));
        // type has_permission(actor: Actor, permission: String, resource: Actor);
        self.add(rule!("has_permission", ["actor"; instance!(sym!("Actor")), "_permission"; instance!(sym!("String")), "resource"; instance!(sym!("Actor"))]));
        // type has_role(actor: Actor, role: String, resource: Resource);
        self.add(rule!("has_role", ["actor"; instance!(sym!("Actor")), "_role"; instance!(sym!("String")), "resource"; instance!(sym!("Resource"))]));
        // type has_role(actor: Actor, role: String, resource: Actor);
        self.add(rule!("has_role", ["actor"; instance!(sym!("Actor")), "_role"; instance!(sym!("String")), "resource"; instance!(sym!("Actor"))]));

        // TODO: revisit this when working on extension guides. This rule currently lets users define any relation they would like, but we may want to restrict that a bit more.
        // type has_relation(_subject: Resource, relation: String, _object: Resource);
        self.add(rule!("has_relation", ["_subject"; instance!(sym!("Resource")), "_relation"; instance!(sym!("String")), "_object"; instance!(sym!("Resource"))]));
        // type has_relation(_subject: Resource, relation: String, _object: Actor);
        self.add(rule!("has_relation", ["_subject"; instance!(sym!("Resource")), "_relation"; instance!(sym!("String")), "_object"; instance!(sym!("Actor"))]));
        // type has_relation(_subject: Actor, relation: String, _object: Actor);
        self.add(rule!("has_relation", ["_subject"; instance!(sym!("Actor")), "_relation"; instance!(sym!("String")), "_object"; instance!(sym!("Actor"))]));
        // type has_relation(_subject: Actor, relation: String, _object: Resource);
        self.add(rule!("has_relation", ["_subject"; instance!(sym!("Actor")), "_relation"; instance!(sym!("String")), "_object"; instance!(sym!("Resource"))]));

        // type allow(actor, action, resource);
        self.add(rule!(
            "allow",
            [sym!("actor"), sym!("_action"), sym!("resource")]
        ));
        // type allow_field(actor, action, resource, field);
        self.add(rule!(
            "allow_field",
            [
                sym!("actor"),
                sym!("action"),
                sym!("resource"),
                sym!("field")
            ]
        ));
        // type allow_request(actor, request);"#;
        self.add(rule!("allow_request", [sym!("actor"), sym!("request")]));
    }

    pub fn get(&self, name: &Symbol) -> Option<&Vec<Rule>> {
        self.0.get(name)
    }

    pub fn add(&mut self, rule_type: Rule) {
        let name = rule_type.name.clone();
        // get rule types with this rule name
        let rule_types = self.0.entry(name).or_insert_with(Vec::new);
        rule_types.push(rule_type);
    }

    pub fn reset(&mut self) {
        self.0.clear();
        self.add_default_rule_types()
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

    pub fn remove_rule(&mut self, rule_id: u64) {
        self.rules.remove(&rule_id);
        self.index
            .iter_mut()
            .for_each(|(_, index)| index.remove_rule(rule_id));
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
    pub rules: HashMap<u64, Arc<Rule>>,
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

    pub fn remove_rule(&mut self, rule_id: u64) {
        self.rules.remove(&rule_id);
        self.index.remove_rule(rule_id);
    }

    #[allow(clippy::ptr_arg)]
    pub fn get_applicable_rules(&self, args: &TermList) -> Rules {
        self.index
            .get_applicable_rules(args, 0)
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::polar::Polar;

    #[test]
    fn test_rule_index() {
        let polar = Polar::new();
        polar
            .load_str(
                r#"
            f(1, 1, "x");
            f(1, 1, "y");
            f(1, x, "y") if x = 2;
            f(1, 2, {b: "y"});
            f(1, 3, {c: "z"});
        "#,
            )
            .unwrap();

        let kb = polar.kb.read().unwrap();
        let generic_rule = kb.get_generic_rule(&sym!("f")).unwrap();
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
