use maplit::btreemap;
use permute::permute;

use std::collections::HashMap;
use std::iter::FromIterator;

use polar::{sym, term, types::*, value, Polar, Query};

type QueryResults = Vec<HashMap<Symbol, Value>>;

fn no_results(_: Symbol, _: Vec<Term>) -> Option<Term> {
    None
}

fn query_results<F>(polar: &mut Polar, mut query: Query, mut external_handler: F) -> QueryResults
where
    F: FnMut(Symbol, Vec<Term>) -> Option<Term>,
{
    let mut results = vec![];
    loop {
        let event = polar.query(&mut query).unwrap();
        match event {
            QueryEvent::Done => break,
            QueryEvent::Result { bindings } => {
                results.push(bindings.into_iter().map(|(k, v)| (k, v.value)).collect());
            }
            QueryEvent::ExternalCall {
                call_id,
                attribute,
                args,
                ..
            } => {
                polar
                    .external_call_result(&mut query, call_id, external_handler(attribute, args))
                    .unwrap();
            }
            _ => {}
        }
    }
    results
}

fn qeval(polar: &mut Polar, query_str: &str) -> bool {
    let query = polar.new_query(query_str).unwrap();
    query_results(polar, query, no_results).len() == 1
}

fn qnull(polar: &mut Polar, query_str: &str) -> bool {
    let query = polar.new_query(query_str).unwrap();
    query_results(polar, query, no_results).is_empty()
}

fn qext(polar: &mut Polar, query_str: &str, external_results: Vec<Value>) -> QueryResults {
    let mut external_results: Vec<Term> =
        external_results.into_iter().map(Term::new).rev().collect();
    let query = polar.new_query(query_str).unwrap();
    query_results(polar, query, |_, _| external_results.pop())
}

fn qvar(polar: &mut Polar, query_str: &str, var: &str) -> Vec<Value> {
    let query = polar.new_query(query_str).unwrap();
    query_results(polar, query, no_results)
        .iter()
        .map(|bindings| bindings.get(&Symbol(var.to_string())).unwrap().clone())
        .collect()
}

fn qvars(polar: &mut Polar, query_str: &str, vars: &[&str]) -> Vec<Vec<Value>> {
    let query = polar.new_query(query_str).unwrap();

    query_results(polar, query, no_results)
        .iter()
        .map(|bindings| {
            vars.iter()
                .map(|&var| bindings.get(&Symbol(var.to_string())).unwrap().clone())
                .collect()
        })
        .collect()
}

/// Adapted from <http://web.cse.ohio-state.edu/~stiff.4/cse3521/prolog-resolution.html>
#[test]
fn test_functions() {
    let mut polar = Polar::new();
    polar
        .load_str("f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);")
        .unwrap();

    assert!(qnull(&mut polar, "k(1)"));
    assert!(qeval(&mut polar, "k(2)"));
    assert!(qnull(&mut polar, "k(3)"));
    assert_eq!(qvar(&mut polar, "k(a)", "a"), vec![value!(2)]);
}

/// Adapted from <http://web.cse.ohio-state.edu/~stiff.4/cse3521/prolog-resolution.html>
#[test]
fn test_jealous() {
    let mut polar = Polar::new();
    polar
        .load_str(
            r#"loves("vincent", "mia");
               loves("marcellus", "mia");
               jealous(a, b) := loves(a, c), loves(b, c);"#,
        )
        .unwrap();

    let query = polar.new_query("jealous(who, of)").unwrap();
    let results = query_results(&mut polar, query, no_results);
    let jealous = |who: &str, of: &str| {
        assert!(
            &results.contains(&HashMap::from_iter(vec![
                (sym!("who"), value!(who)),
                (sym!("of"), value!(of))
            ])),
            "{} is not jealous of {} (but should be)",
            who,
            of
        );
    };
    assert_eq!(results.len(), 4);
    jealous("vincent", "vincent");
    jealous("vincent", "marcellus");
    jealous("marcellus", "vincent");
    jealous("marcellus", "marcellus");
}

#[test]
fn test_nested_rule() {
    let mut polar = Polar::new();
    polar
        .load_str("f(x) := g(x); g(x) := h(x); h(2); g(x) := j(x); j(4);")
        .unwrap();

    assert!(qeval(&mut polar, "f(2)"));
    assert!(qnull(&mut polar, "f(3)"));
    assert!(qeval(&mut polar, "f(4)"));
    assert!(qeval(&mut polar, "j(4)"));
}

#[test]
/// A functions permutation that is known to fail.
fn test_bad_functions() {
    let mut polar = Polar::new();
    polar
        .load_str("f(2); f(1); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);")
        .unwrap();
    assert_eq!(qvar(&mut polar, "k(a)", "a"), vec![value!(2)]);
}

#[test]
fn test_functions_reorder() {
    // TODO (dhatch): Reorder f(x), h(x), g(x)
    let parts = vec![
        "f(1)",
        "f(2)",
        "g(1)",
        "g(2)",
        "h(2)",
        "k(x) := f(x), g(x), h(x)",
    ];

    for (i, permutation) in permute(parts).into_iter().enumerate() {
        let mut polar = Polar::new();

        let mut joined = permutation.join(";");
        joined.push(';');
        polar.load_str(&joined).unwrap();

        assert!(
            qnull(&mut polar, "k(1)"),
            "k(1) was true for permutation {:?}",
            &permutation
        );
        assert!(
            qeval(&mut polar, "k(2)"),
            "k(2) failed for permutation {:?}",
            &permutation
        );

        assert_eq!(
            qvar(&mut polar, "k(a)", "a"),
            vec![value!(2)],
            "k(a) failed for permutation {:?}",
            &permutation
        );

        println!("permute: {}", i);
    }
}

#[test]
fn test_results() {
    let mut polar = Polar::new();
    polar.load_str("foo(1); foo(2); foo(3);").unwrap();
    assert_eq!(
        qvar(&mut polar, "foo(a)", "a"),
        vec![value!(1), value!(2), value!(3)]
    );
}

#[test]
fn test_result_permutations() {
    let parts = vec![
        (1, "foo(1)"),
        (2, "foo(2)"),
        (3, "foo(3)"),
        (4, "foo(4)"),
        (5, "foo(5)"),
    ];
    for permutation in permute(parts).into_iter() {
        eprintln!("{:?}", permutation);
        let mut polar = Polar::new();
        let (results, rules): (Vec<_>, Vec<_>) = permutation.into_iter().unzip();
        polar.load_str(&format!("{};", rules.join(";"))).unwrap();
        assert_eq!(
            qvar(&mut polar, "foo(a)", "a"),
            results.into_iter().map(|v| value!(v)).collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_multi_arg_method_ordering() {
    let mut polar = Polar::new();
    polar
        .load_str("bar(2, 1); bar(1, 1); bar(1, 2); bar(2, 2);")
        .unwrap();
    assert_eq!(
        qvars(&mut polar, "bar(a, b)", &["a", "b"]),
        vec![
            vec![value!(2), value!(1)],
            vec![value!(1), value!(1)],
            vec![value!(1), value!(2)],
            vec![value!(2), value!(2)],
        ]
    );
}

#[test]
fn test_no_applicable_rules() {
    let mut polar = Polar::new();
    assert!(qnull(&mut polar, "f()"));

    polar.load_str("f(x);").unwrap();
    assert!(qnull(&mut polar, "f()"));
}

#[test]
/// From Aït-Kaci's WAM tutorial (1999), page 34.
fn test_ait_kaci_34() {
    let mut polar = Polar::new();
    polar
        .load_str(
            r#"a() := b(x), c(x);
               b(x) := e(x);
               c(1);
               e(x) := f(x);
               e(x) := g(x);
               f(2);
               g(1);"#,
        )
        .unwrap();
    assert!(qeval(&mut polar, "a()"));
}

#[test]
fn test_not() {
    let mut polar = Polar::new();
    polar.load_str("odd(1); even(2);").unwrap();
    assert!(qeval(&mut polar, "odd(1)"));
    assert!(qnull(&mut polar, "!odd(1)"));
    assert!(qnull(&mut polar, "even(1)"));
    assert!(qeval(&mut polar, "!even(1)"));
    assert!(qnull(&mut polar, "odd(2)"));
    assert!(qeval(&mut polar, "!odd(2)"));
    assert!(qeval(&mut polar, "even(2)"));
    assert!(qnull(&mut polar, "!even(2)"));
    assert!(qnull(&mut polar, "even(3)"));
    assert!(qeval(&mut polar, "!even(3)"));

    polar
        .load_str("f(x) := !a(x); a(1); b(2); g(x) := !(a(x) | b(x)), x = 3;")
        .unwrap();

    assert!(qnull(&mut polar, "f(1)"));
    assert!(qeval(&mut polar, "f(2)"));

    assert!(qnull(&mut polar, "g(1)"));
    assert!(qnull(&mut polar, "g(2)"));
    assert!(qeval(&mut polar, "g(3)"));
    assert_eq!(qvar(&mut polar, "g(x)", "x"), vec![value!(3)]);
}

#[test]
fn test_and() {
    let mut polar = Polar::new();
    polar.load_str("f(1); f(2);").unwrap();
    assert!(qeval(&mut polar, "f(1), f(2)"));
    assert!(qnull(&mut polar, "f(1), f(2), f(3)"));
}

#[test]
fn test_equality() {
    let mut polar = Polar::new();
    assert!(qeval(&mut polar, "1 = 1"));
    assert!(qnull(&mut polar, "1 = 2"));
}

#[test]
fn test_lookup() {
    let mut polar = Polar::new();
    assert!(qeval(&mut polar, "{x: 1}.x = 1"));
}

#[test]
fn test_instance_lookup() {
    let mut polar = Polar::new();
    assert_eq!(qext(&mut polar, "a{x: 1}.x = 1", vec![value!(1)]).len(), 1);
}

/// Adapted from <http://web.cse.ohio-state.edu/~stiff.4/cse3521/prolog-resolution.html>
#[test]
fn test_retries() {
    let mut polar = Polar::new();
    polar
        .load_str("f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x); k(3);")
        .unwrap();

    assert!(qnull(&mut polar, "k(1)"));
    assert!(qeval(&mut polar, "k(2)"));
    assert_eq!(qvar(&mut polar, "k(a)", "a"), vec![value!(2), value!(3)]);
    assert!(qeval(&mut polar, "k(3)"));
}

#[test]
fn test_two_rule_bodies_not_nested() {
    let mut polar = Polar::new();
    polar.load_str("f(x) := a(x); f(1);").unwrap();
    assert_eq!(qvar(&mut polar, "f(x)", "x"), vec![value!(1)]);
}

#[test]
fn test_two_rule_bodies_nested() {
    let mut polar = Polar::new();
    polar.load_str("f(x) := a(x); f(1); a(x) := g(x);").unwrap();
    assert_eq!(qvar(&mut polar, "f(x)", "x"), vec![value!(1)]);
}

#[test]
fn test_unify_and() {
    let mut polar = Polar::new();
    polar
        .load_str("f(x, y) := a(x), y = 2; a(1); a(3);")
        .unwrap();
    assert_eq!(qvar(&mut polar, "f(x, y)", "x"), vec![value!(1), value!(3)]);
    assert_eq!(qvar(&mut polar, "f(x, y)", "y"), vec![value!(2), value!(2)]);
}

#[test]
fn test_symbol_lookup() {
    let mut polar = Polar::new();
    assert_eq!(
        qvar(&mut polar, "{x: 1}.x = result", "result"),
        vec![value!(1)]
    );
    assert_eq!(
        qvar(&mut polar, "{x: 1} = dict, dict.x = result", "result"),
        vec![value!(1)]
    );
}

#[test]
fn test_or() {
    let mut polar = Polar::new();
    polar.load_str("f(x) := a(x) | b(x); a(1); b(3);").unwrap();

    assert_eq!(qvar(&mut polar, "f(x)", "x"), vec![value!(1), value!(3)]);
    assert!(qeval(&mut polar, "f(1)"));
    assert!(qnull(&mut polar, "f(2)"));
    assert!(qeval(&mut polar, "f(3)"));

    polar.load_str("g(x) := a(x) | b(x) | c(x); c(5);").unwrap();
    assert_eq!(
        qvar(&mut polar, "g(x)", "x"),
        vec![value!(1), value!(3), value!(5)]
    );
    assert!(qeval(&mut polar, "g(1)"));
    assert!(qnull(&mut polar, "g(2)"));
    assert!(qeval(&mut polar, "g(3)"));
    assert!(qeval(&mut polar, "g(5)"));
}

#[test]
fn test_dict_head() {
    let mut polar = Polar::new();
    polar.load_str("f({x: 1});").unwrap();

    // Test isa-ing dicts against our dict head.
    assert!(qeval(&mut polar, "f({x: 1})"));
    assert!(qeval(&mut polar, "f({x: 1, y: 2})"));
    assert!(qnull(&mut polar, "f(1)"));
    assert!(qnull(&mut polar, "f({})"));
    assert!(qnull(&mut polar, "f({x: 2})"));
    assert!(qnull(&mut polar, "f({y: 1})"));

    // Test isa-ing instances against our dict head.
    assert_eq!(qext(&mut polar, "f(a{x: 1})", vec![value!(1)]).len(), 1);
    assert!(qnull(&mut polar, "f(a{})"));
    assert!(qnull(&mut polar, "f(a{x: {}})"));
    assert!(qext(&mut polar, "f(a{x: 2})", vec![value!(2)]).is_empty());
    assert_eq!(
        qext(&mut polar, "f(a{y: 2, x: 1})", vec![value!(1)]).len(),
        1
    );
}

#[test]
fn test_non_instance_specializers() {
    let mut polar = Polar::new();
    polar.load_str("f(x: 1) := x = 1;").unwrap();
    assert!(qeval(&mut polar, "f(1)"));
    assert!(qnull(&mut polar, "f(2)"));

    polar.load_str("g(x: 1, y: [x]) := y = [1];").unwrap();
    assert!(qeval(&mut polar, "g(1, [1])"));
    assert!(qnull(&mut polar, "g(1, [2])"));

    polar.load_str("h(x: {y: y}, x.y) := y = 1;").unwrap();
    assert!(qeval(&mut polar, "h({y: 1}, 1)"));
    assert!(qnull(&mut polar, "h({y: 1}, 2)"));
}

#[test]
fn test_bindings() {
    let mut polar = Polar::new();
    polar
        .load_str("f(x) := x = y, g(y); g(y) := y = 1;")
        .unwrap();
    assert_eq!(qvar(&mut polar, "f(x)", "x"), vec![value!(1)]);
}

#[test]
fn test_lookup_derefs() {
    let mut polar = Polar::new();
    polar
        .load_str("f(x) := x = y, g(y); g(y) := Foo{}.get(y) = y;")
        .unwrap();
    let query = polar.new_query("f(1)").unwrap();
    let mut foo_lookups = vec![term!(1)];
    let mock_foo = |_, args: Vec<Term>| {
        // check the argument is bound to an integer
        assert!(matches!(args[0].value, Value::Integer(_)));
        foo_lookups.pop()
    };
    let results = query_results(&mut polar, query, mock_foo);
    assert_eq!(results.len(), 1);

    let mut foo_lookups = vec![term!(1)];
    let mock_foo = |_, args: Vec<Term>| {
        assert!(matches!(args[0].value, Value::Integer(_)));
        foo_lookups.pop()
    };
    let query = polar.new_query("f(2)").unwrap();
    let results = query_results(&mut polar, query, mock_foo);
    assert!(results.is_empty());
}

#[test]
fn unify_predicates() {
    let mut polar = Polar::new();
    polar
        .load_str("f(g(x)); k(x) := h(g(x), g(x)); h(g(1), g(1));")
        .unwrap();

    assert!(qeval(&mut polar, "f(g(1))"));
    assert!(qnull(&mut polar, "f(1)"));
    assert!(qeval(&mut polar, "k(1)"));
}

#[test]
fn test_isa_predicate() {
    let mut polar = Polar::new();
    polar
        .load_str("isa(x, y, x: (y)); isa(x, y) := isa(x, y, x);")
        .unwrap();
    assert!(qeval(&mut polar, "isa(1, 1)"));
    assert!(qnull(&mut polar, "isa(1, 2)"));
    assert!(qeval(&mut polar, "isa({x: 1, y: 2}, {y: 2})"));
    assert!(qnull(&mut polar, "isa({x: 1, y: 2}, {x: 2})"));
}

/// Test that rules are executed in the correct order.
#[test]
fn test_rule_order() {
    let mut polar = Polar::new();
    polar.load_str("a(\"foo\");").unwrap();
    polar.load_str("a(\"bar\");").unwrap();
    polar.load_str("a(\"baz\");").unwrap();

    assert_eq!(
        qvar(&mut polar, "a(x)", "x"),
        vec![value!("foo"), value!("bar"), value!("baz")]
    );
}

#[test]
fn test_load_with_query() {
    let mut polar = Polar::new();
    let mut load = polar
        .new_load("f(1); f(2); ?= f(1); ?= !f(3);")
        .expect("new_load failed");

    while let Some(query) = polar.load(&mut load).expect("load failed") {
        assert_eq!(query_results(&mut polar, query, no_results).len(), 1);
    }
}
#[test]
fn test_externals_instantiated() {
    let mut polar = Polar::new();
    polar
        .load_str("f(x, foo: Foo) := foo.bar(Bar{x: x}) = 1;")
        .unwrap();

    let mut foo_lookups = vec![term!(1)];
    let mock_foo = |_, args: Vec<Term>| {
        // make sure that what we get as input is an external instance
        // with the fields set correctly
        assert!(
            matches!(&args[0].value,
                Value::ExternalInstance(ExternalInstance {
                    literal: Some(InstanceLiteral {
                        ref tag, ref fields
                    }),
                    ..
                }) if tag.0 == "Bar" && fields.fields == btreemap!{sym!("x") => term!(1)}),
            "expected external instance Bar {{ x: 1 }}, found: {:?}",
            args[0].value
        );
        foo_lookups.pop()
    };
    let query = polar.new_query("f(1, Foo{})").unwrap();
    let results = query_results(&mut polar, query, mock_foo);
    assert_eq!(results.len(), 1);
}

#[test]
#[ignore] // ignore because this take a LONG time (could consider lowering the goal limit)
#[should_panic(expected = "Goal count exceeded! MAX_EXECUTED_GOALS = 10000")]
fn test_infinite_loop() {
    let mut polar = Polar::new();
    polar.load_str("f(x) := f(x);").unwrap();
    qeval(&mut polar, "f(1)");
}

fn test_comparisons() {
    let mut polar = Polar::new();

    // "<"
    polar
        .load_str("lt(x, y) := x < y; f(x) := x = 1; g(x) := x = 2;")
        .unwrap();
    assert!(qeval(&mut polar, "lt(1,2)"));
    assert!(!qeval(&mut polar, "lt(2,2)"));
    assert!(qeval(&mut polar, "lt({a: 1}.a,{a: 2}.a)"));
    assert!(qeval(&mut polar, "f(x), g(y), lt(x,y)"));

    // "<="
    polar.load_str("leq(x, y) := x <= y;").unwrap();
    assert!(qeval(&mut polar, "leq(1,1)"));
    assert!(qeval(&mut polar, "leq(1,2)"));
    assert!(!qeval(&mut polar, "leq(2,1)"));

    // ">"
    polar.load_str("gt(x, y) := x > y;").unwrap();
    assert!(qeval(&mut polar, "gt(2,1)"));
    assert!(!qeval(&mut polar, "gt(2,2)"));

    // ">="
    polar.load_str("geq(x, y) := x >= y;").unwrap();
    assert!(qeval(&mut polar, "geq(1,1)"));
    assert!(qeval(&mut polar, "geq(2,1)"));
    assert!(!qeval(&mut polar, "geq(1,2)"));

    // "=="
    polar.load_str("eq(x, y) := x == y;").unwrap();
    assert!(qeval(&mut polar, "eq(1,1)"));
    assert!(!qeval(&mut polar, "eq(2,1)"));

    // "!="
    polar.load_str("neq(x, y) := x != y;").unwrap();
    assert!(qeval(&mut polar, "neq(1,2)"));
    assert!(!qeval(&mut polar, "neq(1,1)"));

    let mut query = polar.new_query("eq(bob, bob)").unwrap();
    polar
        .query(&mut query)
        .expect_err("Comparison operators should not allow non-integer operands");
}
