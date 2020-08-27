#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};

    use crate::polar::Polar;

    use crate::events::*;
    use crate::kb::*;
    use crate::rules::*;
    use crate::terms::*;

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
