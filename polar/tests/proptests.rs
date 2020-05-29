use polar::types::*;

use proptest::prelude::*;

prop_compose! {
    fn arb_symbol()(name in r"[a-zA-Z_]+") -> Symbol {
        Symbol(name)
    }
    // fn arb_symbol()(name in r"[\w&&[^\d]]\w*") -> Symbol {
    //     Symbol(name)
    // }
}

fn arbitrary_term() -> impl Strategy<Value = Term> {
    // TODO: unsupported operators commented out here
    let op = prop_oneof![
        Just(Operator::Make),
        // Just(Operator::Dot),
        Just(Operator::Not),
        Just(Operator::Mul),
        Just(Operator::Div),
        Just(Operator::Add),
        Just(Operator::Sub),
        // Just(Operator::Eq),
        // Just(Operator::Geq),
        // Just(Operator::Leq),
        // Just(Operator::Neq),
        // Just(Operator::Gt),
        // Just(Operator::Lt),
        Just(Operator::Unify),
        Just(Operator::Or),
        Just(Operator::And),
    ];

    let leaf = prop_oneof![
        // any::<i64>().prop_map(Value::Integer),
        r#"(\\.|[^"\\\n])*"#.prop_map(Value::String), // TODO: better string test
        arb_symbol().prop_map(Value::Symbol),
        any::<bool>().prop_map(Value::Boolean),
        // any::<u64>()
        //     .prop_map(|instance_id| Value::ExternalInstance(ExternalInstance { instance_id })),
    ]
    .prop_map(Term::new);
    let term = leaf.prop_recursive(
        4,  // 4 levels deep
        16, // Shoot for maximum size of 16 nodes
        3,  // We put up to 3 items per collection
        move |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..3).prop_map(Value::List),
                (arb_symbol(), prop::collection::vec(inner.clone(), 0..3))
                    .prop_map(|(name, args)| { Value::Call(Predicate { name, args }) }),
                prop::collection::btree_map(arb_symbol(), inner.clone(), 0..3)
                    .prop_map(|fields| Value::Dictionary(Dictionary { fields })),
                (
                    arb_symbol(),
                    prop::collection::btree_map(arb_symbol(), inner.clone(), 0..3)
                )
                    .prop_map(|(tag, fields)| {
                        Value::InstanceLiteral(InstanceLiteral {
                            tag,
                            fields: Dictionary { fields },
                        })
                    }),
                // (
                //     any::<u64>(),
                //     arb_symbol(),
                //     prop::collection::btree_map(arb_symbol(), inner.clone(), 0..3)
                // )
                //     .prop_map(|(id, tag, fields)| Value::ExternalInstance(
                //         ExternalInstance {
                //             instance_id: id,
                //             literal: Some(InstanceLiteral {
                //                 tag,
                //                 fields: Dictionary { fields },
                //             }),
                //         }
                //     )),
            ]
            .prop_map(Term::new)
        },
    );
    // final term is made up of some composition of terms with operators
    (op.clone(), prop::collection::vec(term.clone(), 2))
        .prop_map(|(operator, args)| Value::Expression(Operation { operator, args }))
        .prop_map(Term::new)
}

use polar::ToPolarString;

proptest! {
    #[test]
    fn test_parse_term(t in arbitrary_term()) {
        let s = t.to_polar();
        polar::parser::parse_query(&s).expect(&format!("Failed to parse: {}", s));
    }
}
