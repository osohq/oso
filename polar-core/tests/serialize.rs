#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use polar_core::{error::*, events::*, rules::*, term, terms::*, value};

    #[test]
    fn serialize_test() {
        let mut kwargs = BTreeMap::new();
        kwargs.insert(Symbol::new("bar"), term!(1));
        let pred = Call {
            name: Symbol("foo".to_owned()),
            args: vec![Term::new_from_test(value!(0))],
            kwargs: Some(kwargs),
        };
        assert_eq!(
            serde_json::to_string(&pred).unwrap(),
            r#"{"name":"foo","args":[{"value":{"Number":{"Integer":0}}}],"kwargs":{"bar":{"value":{"Number":{"Integer":1}}}}}"#
        );
        let event = QueryEvent::ExternalCall {
            call_id: 2,
            instance: Term::new_from_test(Value::String("abc".to_string())),
            attribute: Symbol::new("foo"),
            args: Some(vec![
                Term::new_from_test(value!(0)),
                Term::new_from_test(value!("hello")),
            ]),
            kwargs: None,
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
        let constructor = Call {
            name: Symbol::new("Foo"),
            args: vec![
                term!(1234),
                Term::new_from_test(Value::String("something".to_owned())),
            ],
            kwargs: Some(fields),
        };
        let event = QueryEvent::MakeExternal {
            instance_id: 12345,
            constructor: Term::new_from_test(Value::Call(constructor)),
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
        let e = ParseError::InvalidTokenCharacter {
            token: "Integer".to_owned(),
            c: 'x',
            loc: 99,
        };
        let err: PolarError = e.into();
        eprintln!("{}", serde_json::to_string(&err).unwrap());
        let rule = Rule::new_from_test(
            Symbol::new("foo"),
            vec![],
            Term::new_temporary(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![dict.clone(), dict.clone(), dict],
            })),
        );
        eprintln!("{}", rule);
    }
}
