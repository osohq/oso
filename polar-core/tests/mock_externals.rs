/// Utils for mocking externals in tests.
use std::collections::{HashMap, HashSet};

use polar_core::types::{ExternalInstance, Symbol, Term, Value};

#[derive(Default)]
/// Mock external that keeps track of instance literals and allows
/// lookups of attributes on them.
pub struct MockExternal {
    externals: HashMap<u64, Term>,
    calls: HashSet<u64>,
}

impl MockExternal {
    pub fn new() -> Self {
        MockExternal::default()
    }

    fn get_external(&self, instance_id: u64) -> &Value {
        self.externals
            .get(&instance_id)
            .expect("Instance not constructed")
            .value()
    }

    pub fn external_call(
        &mut self,
        call_id: u64,
        instance: Option<Term>,
        attribute: Symbol,
        args: Option<Vec<Term>>,
    ) -> Option<Term> {
        assert!(args.is_none(), "Only support field lookups.");

        if self.calls.remove(&call_id) {
            // Calls only return one result, so we have none if the call is in progress.
            return None;
        }

        self.calls.insert(call_id);
        let instance_id = match instance.unwrap().value() {
            Value::ExternalInstance(ExternalInstance { instance_id, .. }) => *instance_id,
            _ => panic!("expected external instance"),
        };
        match self.get_external(instance_id) {
            Value::InstanceLiteral(literal) => literal.fields.fields.get(&attribute).cloned(),
            _ => panic!("expected instance literal"),
        }
    }

    pub fn make_external(&mut self, instance_id: u64, constructor: Term) {
        assert!(self.externals.insert(instance_id, constructor).is_none());
    }

    pub fn external_isa(&mut self, instance: Term, class_tag: Symbol) -> bool {
        // True if class tags match
        if let Value::ExternalInstance(ExternalInstance { instance_id, .. }) = instance.value() {
            match self.get_external(*instance_id) {
                Value::InstanceLiteral(literal) => literal.tag == class_tag,
                _ => panic!("expected instance literal"),
            }
        } else {
            false
        }
    }

    pub fn external_is_subspecializer(
        &mut self,
        instance_id: u64,
        left_class_tag: Symbol,
        right_class_tag: Symbol,
    ) -> bool {
        match self.get_external(instance_id) {
            Value::InstanceLiteral(literal) => {
                literal.tag == left_class_tag || literal.tag == right_class_tag
            }
            _ => panic!("expected instance literal"),
        }
    }
}
