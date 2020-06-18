/// Utils for mocking externals in tests.
use std::collections::{HashMap, HashSet};

use polar::types::{InstanceLiteral, Symbol, Term};

#[derive(Default)]
/// Mock external that keeps track of instance literals and allows
/// lookups of attributes on them.
pub struct MockExternal {
    externals: HashMap<u64, InstanceLiteral>,
    calls: HashSet<u64>,
}

impl MockExternal {
    pub fn new() -> Self {
        MockExternal::default()
    }

    pub fn external_call(
        &mut self,
        attribute: Symbol,
        args: Vec<Term>,
        call_id: u64,
        instance_id: u64,
    ) -> Option<Term> {
        assert_eq!(args.len(), 0, "Only support field lookups.");

        if self.calls.remove(&call_id) {
            // Calls only return one result, so we have none if the call is in progress.
            return None;
        }

        self.calls.insert(call_id);
        self.externals
            .get(&instance_id)
            .expect("Instance not constructed")
            .fields
            .fields
            .get(&attribute)
            .cloned()
    }

    pub fn make_external(&mut self, instance_id: u64, literal: InstanceLiteral) {
        assert!(self.externals.insert(instance_id, literal).is_none());
    }

    pub fn external_isa(&mut self, instance_id: u64, class_tag: Symbol) -> bool {
        // True if class tags match
        self.externals
            .get(&instance_id)
            .expect("Instance to be created")
            .tag
            == class_tag
    }

    pub fn external_is_subspecializer(
        &mut self,
        instance_id: u64,
        left_class_tag: Symbol,
        right_class_tag: Symbol,
    ) -> bool {
        self.externals.get(&instance_id).expect("Instance").tag == left_class_tag
            || self.externals.get(&instance_id).expect("Instance").tag == right_class_tag
    }
}
