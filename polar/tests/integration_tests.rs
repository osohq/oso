use ::polar::{polar::*, types::*, value, sym};

use std::collections::HashMap;
use std::iter::FromIterator;

use permute::permute;

fn query_results(polar: &mut Polar, mut query: Query) -> Vec<HashMap<Symbol, Value>> {
    let mut external_results = vec![Term::new(Value::Integer(1))];
    let mut results = vec![];
    loop {
        let event = polar.query(&mut query).unwrap();
        match event {
            QueryEvent::Done => break,
            QueryEvent::Result { bindings } => {
                results.push(bindings.into_iter().map(|(k, v)| (k, v.value)).collect());
            }
            QueryEvent::ExternalCall { call_id, .. } => {
                polar.external_call_result(&mut query, call_id, external_results.pop());
            }
            _ => panic!("unexpected event"),
        }
    }
    results
}

fn qeval(polar: &mut Polar, query_str: &str) -> bool {
    let query = polar.new_query(query_str).unwrap();
    query_results(polar, query).len() == 1
}

fn qnull(polar: &mut Polar, query_str: &str) -> bool {
    let query = polar.new_query(query_str).unwrap();
    query_results(polar, query).is_empty()
}

fn qvar(polar: &mut Polar, query_str: &str, var: &str) -> Vec<Value> {
    let query = polar.new_query(query_str).unwrap();
    query_results(polar, query)
        .iter()
        .map(|bindings| bindings.get(&Symbol(var.to_string())).unwrap().clone())
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
    let results = query_results(&mut polar, query);
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
#[ignore = "not implemented yet"]
fn test_instance_lookup() {
    let mut polar = Polar::new();
    assert!(qeval(&mut polar, "a{x: 1}.x = 1"));
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

