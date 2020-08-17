mod mock_externals;

use indoc::indoc;
use maplit::btreemap;
use permute::permute;

use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::FromIterator;

use polar_core::{error::*, polar::Polar, polar::Query, sym, term, types::*, value};

type QueryResults = Vec<(HashMap<Symbol, Value>, Option<TraceResult>)>;
use mock_externals::MockExternal;

fn no_results(_: u64, _: Option<Term>, _: Symbol, _: Vec<Term>) -> Option<Term> {
    None
}

fn no_externals(_: u64, _: Term) {}

fn no_debug(_: &str) -> String {
    "".to_string()
}

fn no_isa(_: Term, _: Symbol) -> bool {
    true
}

fn no_is_subspecializer(_: u64, _: Symbol, _: Symbol) -> bool {
    false
}

fn query_results<F, G, H, I, J>(
    mut query: Query,
    mut external_call_handler: F,
    mut make_external_handler: H,
    mut external_isa_handler: I,
    mut external_is_subspecializer_handler: J,
    mut debug_handler: G,
) -> QueryResults
where
    F: FnMut(u64, Option<Term>, Symbol, Vec<Term>) -> Option<Term>,
    G: FnMut(&str) -> String,
    H: FnMut(u64, Term),
    I: FnMut(Term, Symbol) -> bool,
    J: FnMut(u64, Symbol, Symbol) -> bool,
{
    let mut results = vec![];
    loop {
        let event = query.next_event().unwrap();
        match event {
            QueryEvent::Done => break,
            QueryEvent::Result { bindings, trace } => {
                results.push((
                    bindings
                        .into_iter()
                        .map(|(k, v)| (k, v.value().clone()))
                        .collect(),
                    trace,
                ));
            }
            QueryEvent::ExternalCall {
                call_id,
                instance,
                attribute,
                args,
            } => {
                query
                    .call_result(
                        call_id,
                        external_call_handler(call_id, instance, attribute, args),
                    )
                    .unwrap();
            }
            QueryEvent::MakeExternal {
                instance_id,
                constructor,
            } => make_external_handler(instance_id, constructor),
            QueryEvent::ExternalIsa {
                call_id,
                instance,
                class_tag,
            } => query.question_result(call_id, external_isa_handler(instance, class_tag)),
            QueryEvent::ExternalIsSubSpecializer {
                call_id,
                instance_id,
                left_class_tag,
                right_class_tag,
            } => query.question_result(
                call_id,
                external_is_subspecializer_handler(instance_id, left_class_tag, right_class_tag),
            ),
            QueryEvent::Debug { ref message } => {
                query.debug_command(&debug_handler(message)).unwrap();
            }
            _ => {}
        }
    }
    results
}

macro_rules! query_results {
    ($query:expr) => {
        query_results(
            $query,
            no_results,
            no_externals,
            no_isa,
            no_is_subspecializer,
            no_debug,
        )
    };
    ($query:expr, $external_call_handler:expr, $make_external_handler:expr, $debug_handler:expr) => {
        query_results(
            $query,
            $external_call_handler,
            $make_external_handler,
            no_isa,
            no_is_subspecializer,
            $debug_handler,
        )
    };
    ($query:expr, $external_call_handler:expr) => {
        query_results(
            $query,
            $external_call_handler,
            no_externals,
            no_isa,
            no_is_subspecializer,
            no_debug,
        )
    };
}

fn query_results_with_externals(query: Query) -> (QueryResults, MockExternal) {
    let mock = RefCell::new(MockExternal::new());
    (
        query_results(
            query,
            |a, b, c, d| mock.borrow_mut().external_call(a, b, c, d),
            |a, b| mock.borrow_mut().make_external(a, b),
            |a, b| mock.borrow_mut().external_isa(a, b),
            |a, b, c| mock.borrow_mut().external_is_subspecializer(a, b, c),
            no_debug,
        ),
        mock.into_inner(),
    )
}

fn qeval(polar: &mut Polar, query_str: &str) -> bool {
    let query = polar.new_query(query_str, false).unwrap();
    !query_results!(query).is_empty()
}

fn qnull(polar: &mut Polar, query_str: &str) -> bool {
    let query = polar.new_query(query_str, false).unwrap();
    query_results!(query).is_empty()
}

fn qext(polar: &mut Polar, query_str: &str, external_results: Vec<Value>) -> QueryResults {
    let mut external_results: Vec<Term> = external_results
        .into_iter()
        .map(Term::new_from_test)
        .rev()
        .collect();
    let query = polar.new_query(query_str, false).unwrap();
    query_results!(query, |_, _, _, _| external_results.pop())
}

fn qvar(polar: &mut Polar, query_str: &str, var: &str) -> Vec<Value> {
    let query = polar.new_query(query_str, false).unwrap();
    query_results!(query)
        .iter()
        .map(|bindings| bindings.0.get(&Symbol(var.to_string())).unwrap().clone())
        .collect()
}

fn qvars(polar: &mut Polar, query_str: &str, vars: &[&str]) -> Vec<Vec<Value>> {
    let query = polar.new_query(query_str, false).unwrap();

    query_results!(query)
        .iter()
        .map(|bindings| {
            vars.iter()
                .map(|&var| bindings.0.get(&Symbol(var.to_string())).unwrap().clone())
                .collect()
        })
        .collect()
}

/// Adapted from <http://web.cse.ohio-state.edu/~stiff.4/cse3521/prolog-resolution.html>
#[test]
fn test_functions() {
    let mut polar = Polar::new(None);
    polar
        .load("f(1); f(2); g(1); g(2); h(2); k(x) if f(x) and h(x) and g(x);")
        .unwrap();

    assert!(qnull(&mut polar, "k(1)"));
    assert!(qeval(&mut polar, "k(2)"));
    assert!(qnull(&mut polar, "k(3)"));
    assert_eq!(qvar(&mut polar, "k(a)", "a"), vec![value!(2)]);
}

/// Adapted from <http://web.cse.ohio-state.edu/~stiff.4/cse3521/prolog-resolution.html>
#[test]
fn test_jealous() {
    let polar = Polar::new(None);
    polar
        .load(
            r#"loves("vincent", "mia");
               loves("marcellus", "mia");
               jealous(a, b) if loves(a, c) and loves(b, c);"#,
        )
        .unwrap();

    let query = polar.new_query("jealous(who, of)", false).unwrap();
    let results = query_results!(query);
    let jealous = |who: &str, of: &str| {
        assert!(
            &results.iter().any(|(r, _)| r
                == &HashMap::from_iter(vec![(sym!("who"), value!(who)), (sym!("of"), value!(of))])),
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
fn test_trace() {
    let polar = Polar::new(None);
    polar
        .load("f(x) if x = 1 and x = 1; f(y) if y = 1;")
        .unwrap();
    let query = polar.new_query("f(1)", true).unwrap();
    let results = query_results!(query);
    let trace = results[0].1.as_ref().unwrap();
    let expected = r#"f(1) [
  f(x) if
    x = 1 and x = 1 [
      x = 1 []
      x = 1 []
  ]
]
"#;
    assert_eq!(trace.formatted, expected);
    let trace = results[1].1.as_ref().unwrap();
    let expected = r#"f(1) [
  f(y) if
    y = 1 [
      y = 1 []
  ]
]
"#;
    assert_eq!(trace.formatted, expected);
}

#[test]
fn test_nested_rule() {
    let mut polar = Polar::new(None);
    polar
        .load("f(x) if g(x); g(x) if h(x); h(2); g(x) if j(x); j(4);")
        .unwrap();

    assert!(qeval(&mut polar, "f(2)"));
    assert!(qnull(&mut polar, "f(3)"));
    assert!(qeval(&mut polar, "f(4)"));
    assert!(qeval(&mut polar, "j(4)"));
}

/// A functions permutation that is known to fail.
#[test]
fn test_bad_functions() {
    let mut polar = Polar::new(None);
    polar
        .load("f(2); f(1); g(1); g(2); h(2); k(x) if f(x) and h(x) and g(x);")
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
        "k(x) if f(x) and g(x) and h(x)",
    ];

    for (i, permutation) in permute(parts).into_iter().enumerate() {
        let mut polar = Polar::new(None);

        let mut joined = permutation.join(";");
        joined.push(';');
        polar.load(&joined).unwrap();

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
    let mut polar = Polar::new(None);
    polar.load("foo(1); foo(2); foo(3);").unwrap();
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
        let mut polar = Polar::new(None);
        let (results, rules): (Vec<_>, Vec<_>) = permutation.into_iter().unzip();
        polar.load(&format!("{};", rules.join(";"))).unwrap();
        assert_eq!(
            qvar(&mut polar, "foo(a)", "a"),
            results.into_iter().map(|v| value!(v)).collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_multi_arg_method_ordering() {
    let mut polar = Polar::new(None);
    polar
        .load("bar(2, 1); bar(1, 1); bar(1, 2); bar(2, 2);")
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
    let mut polar = Polar::new(None);
    assert!(qnull(&mut polar, "f()"));

    polar.load("f(_);").unwrap();
    assert!(qnull(&mut polar, "f()"));
}

/// From Aït-Kaci's WAM tutorial (1999), page 34.
#[test]
fn test_ait_kaci_34() {
    let mut polar = Polar::new(None);
    polar
        .load(
            r#"a() if b(x) and c(x);
               b(x) if e(x);
               c(1);
               e(x) if f(x);
               e(x) if g(x);
               f(2);
               g(1);"#,
        )
        .unwrap();
    assert!(qeval(&mut polar, "a()"));
}

#[test]
fn test_constants() {
    let mut polar = Polar::new(None);
    {
        let mut kb = polar.kb.write().unwrap();
        kb.constant(sym!("one"), term!(1));
        kb.constant(sym!("two"), term!(2));
        kb.constant(sym!("three"), term!(3));
    }
    polar
        .load(
            r#"one(x) if one = one and one = x and x < two;
               two(x) if one < x and two = two and two = x and two < three;
               three(x) if three = three and three = x;"#,
        )
        .unwrap();
    assert!(qeval(&mut polar, "one(1)"));
    assert!(qnull(&mut polar, "two(1)"));
    assert!(qeval(&mut polar, "two(2)"));
    assert!(qnull(&mut polar, "three(2)"));
    assert!(qeval(&mut polar, "three(3)"));
}

#[test]
fn test_not() {
    let mut polar = Polar::new(None);
    polar.load("odd(1); even(2);").unwrap();
    assert!(qeval(&mut polar, "odd(1)"));
    assert!(qnull(&mut polar, "not odd(1)"));
    assert!(qnull(&mut polar, "even(1)"));
    assert!(qeval(&mut polar, "not even(1)"));
    assert!(qnull(&mut polar, "odd(2)"));
    assert!(qeval(&mut polar, "not odd(2)"));
    assert!(qeval(&mut polar, "even(2)"));
    assert!(qnull(&mut polar, "not even(2)"));
    assert!(qnull(&mut polar, "even(3)"));
    assert!(qeval(&mut polar, "not even(3)"));

    polar
        .load("f(x) if not a(x); a(1); b(2); g(x) if not (a(x) or b(x));")
        .unwrap();

    assert!(qnull(&mut polar, "f(1)"));
    assert!(qeval(&mut polar, "f(2)"));

    assert!(qnull(&mut polar, "g(1)"));
    assert!(qnull(&mut polar, "g(2)"));
    assert!(qeval(&mut polar, "g(3)"));
    assert!(qnull(&mut polar, "g(x) and x=3")); // this should fail because unbound x means g(x) always fails
    assert!(qeval(&mut polar, "x=3 and g(x)"));

    polar
        .load("h(x) if not (not (x = 1 or x = 3) or x = 3);")
        .unwrap();
    assert!(qeval(&mut polar, "h(1)"));
    assert!(qnull(&mut polar, "h(2)"));
    assert!(qnull(&mut polar, "h(3)"));

    assert!(qeval(
        &mut polar,
        "
        d = {x: 1} and not d.x = 2
    "
    ));
}

#[test]
fn test_and() {
    let mut polar = Polar::new(None);
    polar.load("f(1); f(2);").unwrap();
    assert!(qeval(&mut polar, "f(1) and f(2)"));
    assert!(qnull(&mut polar, "f(1) and f(2) and f(3)"));
}

#[test]
fn test_equality() {
    let mut polar = Polar::new(None);
    assert!(qeval(&mut polar, "1 = 1"));
    assert!(qnull(&mut polar, "1 = 2"));
}

#[test]
fn test_lookup() {
    let mut polar = Polar::new(None);
    assert!(qeval(&mut polar, "{x: 1}.x = 1"));
}

#[test]
fn test_instance_lookup() {
    let mut polar = Polar::new(None);
    // Q: Not sure if this should be allowed? I can't get (new a{x: 1}).x to parse, but that might
    // be the only thing we should permit
    assert_eq!(
        qext(&mut polar, "new a{x: 1}.x = 1", vec![value!(1)]).len(),
        1
    );
}

/// Adapted from <http://web.cse.ohio-state.edu/~stiff.4/cse3521/prolog-resolution.html>
#[test]
fn test_retries() {
    let mut polar = Polar::new(None);
    polar
        .load("f(1); f(2); g(1); g(2); h(2); k(x) if f(x) and h(x) and g(x); k(3);")
        .unwrap();

    assert!(qnull(&mut polar, "k(1)"));
    assert!(qeval(&mut polar, "k(2)"));
    assert_eq!(qvar(&mut polar, "k(a)", "a"), vec![value!(2), value!(3)]);
    assert!(qeval(&mut polar, "k(3)"));
}

#[test]
fn test_two_rule_bodies_not_nested() {
    let mut polar = Polar::new(None);
    polar.load("f(x) if a(x); f(1);").unwrap();
    assert_eq!(qvar(&mut polar, "f(x)", "x"), vec![value!(1)]);
}

#[test]
fn test_two_rule_bodies_nested() {
    let mut polar = Polar::new(None);
    polar.load("f(x) if a(x); f(1); a(x) if g(x);").unwrap();
    assert_eq!(qvar(&mut polar, "f(x)", "x"), vec![value!(1)]);
}

#[test]
fn test_unify_and() {
    let mut polar = Polar::new(None);
    polar
        .load("f(x, y) if a(x) and y = 2; a(1); a(3);")
        .unwrap();
    assert_eq!(qvar(&mut polar, "f(x, y)", "x"), vec![value!(1), value!(3)]);
    assert_eq!(qvar(&mut polar, "f(x, y)", "y"), vec![value!(2), value!(2)]);
}

#[test]
fn test_symbol_lookup() {
    let mut polar = Polar::new(None);
    assert_eq!(
        qvar(&mut polar, "{x: 1}.x = result", "result"),
        vec![value!(1)]
    );
    assert_eq!(
        qvar(&mut polar, "{x: 1} = dict and dict.x = result", "result"),
        vec![value!(1)]
    );
}

#[test]
fn test_or() {
    let mut polar = Polar::new(None);
    polar.load("f(x) if a(x) or b(x); a(1); b(3);").unwrap();

    assert_eq!(qvar(&mut polar, "f(x)", "x"), vec![value!(1), value!(3)]);
    assert!(qeval(&mut polar, "f(1)"));
    assert!(qnull(&mut polar, "f(2)"));
    assert!(qeval(&mut polar, "f(3)"));

    polar.load("g(x) if a(x) or b(x) or c(x); c(5);").unwrap();
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
    let mut polar = Polar::new(None);
    polar.load("f({x: 1});").unwrap();
    polar.load("g(_: {x: 1});").unwrap();

    // Test unifying dicts against our dict head.
    assert!(qeval(&mut polar, "f({x: 1})"));
    assert!(qnull(&mut polar, "f({x: 1, y: 2})"));
    assert!(qnull(&mut polar, "f(1)"));
    assert!(qnull(&mut polar, "f({})"));
    assert!(qnull(&mut polar, "f({x: 2})"));
    assert!(qnull(&mut polar, "f({y: 1})"));

    assert!(qeval(&mut polar, "g({x: 1})"));
    assert!(qeval(&mut polar, "g({x: 1, y: 2})"));
    assert!(qnull(&mut polar, "g(1)"));
    assert!(qnull(&mut polar, "g({})"));
    assert!(qnull(&mut polar, "g({x: 2})"));
    assert!(qnull(&mut polar, "g({y: 1})"));

    // Test unifying & isa-ing instances against our rules.
    assert!(qnull(&mut polar, "f(new a{x: 1})"));
    assert_eq!(qext(&mut polar, "g(new a{x: 1})", vec![value!(1)]).len(), 1);
    assert!(qnull(&mut polar, "f(new a{})"));
    assert!(qnull(&mut polar, "f(new a{x: {}})"));
    assert!(qext(&mut polar, "g(new a{x: 2})", vec![value!(2)]).is_empty());
    assert_eq!(
        qext(&mut polar, "g(new a{y: 2, x: 1})", vec![value!(1)]).len(),
        1
    );
}

#[test]
fn test_non_instance_specializers() {
    let mut polar = Polar::new(None);
    polar.load("f(x: 1) if x = 1;").unwrap();
    assert!(qeval(&mut polar, "f(1)"));
    assert!(qnull(&mut polar, "f(2)"));

    polar.load("g(x: 1, y: [x]) if y = [1];").unwrap();
    assert!(qeval(&mut polar, "g(1, [1])"));
    assert!(qnull(&mut polar, "g(1, [2])"));

    polar.load("h(x: {y: y}, x.y) if y = 1;").unwrap();
    assert!(qeval(&mut polar, "h({y: 1}, 1)"));
    assert!(qnull(&mut polar, "h({y: 1}, 2)"));
}

#[test]
fn test_bindings() {
    let mut polar = Polar::new(None);
    assert_eq!(qvar(&mut polar, "x=1", "x"), vec![value!(1)]);
    assert_eq!(qvar(&mut polar, "x=x", "x"), vec![value!(sym!("x"))]);
    assert_eq!(
        qvar(&mut polar, "x=y and y=x", "x"),
        vec![value!(sym!("y"))]
    );

    polar
        .load("f(x) if x = y and g(y); g(y) if y = 1;")
        .unwrap();
    assert_eq!(qvar(&mut polar, "f(x)", "x"), vec![value!(1)]);
}

#[test]
fn test_lookup_derefs() {
    let polar = Polar::new(None);
    polar
        .load("f(x) if x = y and g(y); g(y) if new Foo{}.get(y) = y;")
        .unwrap();
    let query = polar.new_query("f(1)", false).unwrap();
    let mut foo_lookups = vec![term!(1)];
    let mock_foo = |_, _, _, args: Vec<Term>| {
        // check the argument is bound to an integer
        assert!(matches!(args[0].value(), Value::Number(_)));
        foo_lookups.pop()
    };

    let results = query_results!(query, mock_foo);
    assert_eq!(results.len(), 1);

    let mut foo_lookups = vec![term!(1)];
    let mock_foo = |_, _, _, args: Vec<Term>| {
        assert!(matches!(args[0].value(), Value::Number(_)));
        foo_lookups.pop()
    };
    let query = polar.new_query("f(2)", false).unwrap();
    let results = query_results!(query, mock_foo);
    assert!(results.is_empty());
}

#[test]
fn unify_predicates() {
    let mut polar = Polar::new(None);
    polar
        .load("f(g(_x)); k(x) if h(g(x), g(x)); h(g(1), g(1));")
        .unwrap();

    assert!(qeval(&mut polar, "f(g(1))"));
    assert!(qnull(&mut polar, "f(1)"));
    assert!(qeval(&mut polar, "k(1)"));
}

/// Test that rules are executed in the correct order.
#[test]
fn test_rule_order() {
    let mut polar = Polar::new(None);
    polar.load("a(\"foo\");").unwrap();
    polar.load("a(\"bar\");").unwrap();
    polar.load("a(\"baz\");").unwrap();

    assert_eq!(
        qvar(&mut polar, "a(x)", "x"),
        vec![value!("foo"), value!("bar"), value!("baz")]
    );
}

#[test]
fn test_load_with_query() {
    let polar = Polar::new(None);
    let src = "f(1); f(2); ?= f(1); ?= not f(3);";
    polar.load(src).expect("load failed");

    while let Some(query) = polar.next_inline_query(false) {
        assert_eq!(query_results!(query).len(), 1);
    }
}

#[test]
fn test_externals_instantiated() {
    let mut polar = Polar::new(None);
    polar.register_constant(sym!("Foo"), term!(true));
    polar
        .load("f(x, foo: Foo) if foo.bar(new Bar{x: x}) = 1;")
        .unwrap();

    let mut foo_lookups = vec![term!(1)];
    let mock_foo = |_, _, _, args: Vec<Term>| {
        // make sure that what we get as input is an external instance
        // with the fields set correctly
        match &args[0].value() {
            Value::ExternalInstance(ExternalInstance {
                constructor: Some(ref term),
                ..
            }) => assert!(
                matches!(term.value(), Value::InstanceLiteral(InstanceLiteral {
                ref tag, ref fields
            }) if tag.0 == "Bar" && fields.fields == btreemap!{sym!("x") => term!(1)}),
                "expected external instance Bar {{ x: 1 }}, found: {:?}",
                args[0].value()
            ),
            _ => panic!("Expected external instance"),
        }
        foo_lookups.pop()
    };
    let query = polar.new_query("f(1, new Foo{})", false).unwrap();
    let results = query_results!(query, mock_foo);
    assert_eq!(results.len(), 1);
}

#[test]
#[ignore] // ignore because this take a LONG time (could consider lowering the goal limit)
#[should_panic(expected = "Goal count exceeded! MAX_EXECUTED_GOALS = 10000")]
fn test_infinite_loop() {
    let mut polar = Polar::new(None);
    polar.load("f(x) if f(x);").unwrap();
    qeval(&mut polar, "f(1)");
}

#[test]
fn test_comparisons() {
    let mut polar = Polar::new(None);

    // <
    polar.load("lt(x, y) if x < y;").unwrap();
    assert!(qnull(&mut polar, "lt(1,1)"));
    assert!(qeval(&mut polar, "lt(1,2)"));
    assert!(qnull(&mut polar, "lt(2,1)"));
    assert!(qnull(&mut polar, "lt(+1,-1)"));
    assert!(qeval(&mut polar, "lt(-1,+1)"));
    assert!(qnull(&mut polar, "lt(-1,-1)"));
    assert!(qeval(&mut polar, "lt(-2,-1)"));
    assert!(qeval(&mut polar, "lt(1019,1e19)"));
    assert!(qnull(&mut polar, "lt(1e19,1019)"));
    assert!(qnull(&mut polar, "lt(9007199254740992,9007199254740992)")); // identical
    assert!(qnull(&mut polar, "lt(9007199254740992,9007199254740992.0)")); // equal
    assert!(qnull(&mut polar, "lt(9007199254740992,9007199254740993.0)")); // indistinguishable
    assert!(qeval(&mut polar, "lt(9007199254740992,9007199254740994.0)")); // distinguishable
    assert!(qeval(&mut polar, "lt(\"aa\",\"ab\")"));
    assert!(qnull(&mut polar, "lt(\"aa\",\"aa\")"));

    // <=
    polar.load("leq(x, y) if x <= y;").unwrap();
    assert!(qeval(&mut polar, "leq(1,1)"));
    assert!(qeval(&mut polar, "leq(1,2)"));
    assert!(qnull(&mut polar, "leq(2,1)"));
    assert!(qnull(&mut polar, "leq(+1,-1)"));
    assert!(qeval(&mut polar, "leq(-1,+1)"));
    assert!(qeval(&mut polar, "leq(-1,-1)"));
    assert!(qeval(&mut polar, "leq(-2,-1)"));
    assert!(qeval(&mut polar, "leq(\"aa\",\"aa\")"));
    assert!(qeval(&mut polar, "leq(\"aa\",\"ab\")"));
    assert!(qnull(&mut polar, "leq(\"ab\",\"aa\")"));

    // >
    polar.load("gt(x, y) if x > y;").unwrap();
    assert!(qnull(&mut polar, "gt(1,1)"));
    assert!(qnull(&mut polar, "gt(1,2)"));
    assert!(qeval(&mut polar, "gt(2,1)"));
    assert!(qeval(&mut polar, "gt(+1,-1)"));
    assert!(qnull(&mut polar, "gt(-1,+1)"));
    assert!(qnull(&mut polar, "gt(-1,-1)"));
    assert!(qeval(&mut polar, "gt(-1,-2)"));
    assert!(qeval(&mut polar, "gt(\"ab\",\"aa\")"));
    assert!(qnull(&mut polar, "gt(\"aa\",\"aa\")"));

    // >=
    polar.load("geq(x, y) if x >= y;").unwrap();
    assert!(qeval(&mut polar, "geq(1,1)"));
    assert!(qnull(&mut polar, "geq(1,2)"));
    assert!(qeval(&mut polar, "geq(2,1)"));
    assert!(qeval(&mut polar, "geq(2,1)"));
    assert!(qeval(&mut polar, "geq(+1,-1)"));
    assert!(qnull(&mut polar, "geq(-1,+1)"));
    assert!(qeval(&mut polar, "geq(-1,-1)"));
    assert!(qeval(&mut polar, "geq(-1,-1.0)"));
    assert!(qeval(&mut polar, "geq(\"ab\",\"aa\")"));
    assert!(qeval(&mut polar, "geq(\"aa\",\"aa\")"));

    // ==
    polar.load("eq(x, y) if x == y;").unwrap();
    assert!(qeval(&mut polar, "eq(1,1)"));
    assert!(qnull(&mut polar, "eq(1,2)"));
    assert!(qnull(&mut polar, "eq(2,1)"));
    assert!(qnull(&mut polar, "eq(-1,+1)"));
    assert!(qeval(&mut polar, "eq(-1,-1)"));
    assert!(qeval(&mut polar, "eq(-1,-1.0)"));
    assert!(qnull(&mut polar, "eq(1019,1e19)"));
    assert!(qnull(&mut polar, "eq(1e19,1019)"));
    assert!(qeval(&mut polar, "eq(9007199254740992,9007199254740992)")); // identical
    assert!(qeval(&mut polar, "eq(9007199254740992,9007199254740992.0)")); // equal
    assert!(qeval(&mut polar, "eq(9007199254740992,9007199254740993.0)")); // indistinguishable
    assert!(qnull(&mut polar, "eq(9007199254740992,9007199254740994.0)")); // distinguishable
    assert!(qeval(&mut polar, "eq(\"aa\", \"aa\")"));
    assert!(qnull(&mut polar, "eq(\"ab\", \"aa\")"));

    // !=
    polar.load("neq(x, y) if x != y;").unwrap();
    assert!(qnull(&mut polar, "neq(1,1)"));
    assert!(qeval(&mut polar, "neq(1,2)"));
    assert!(qeval(&mut polar, "neq(2,1)"));
    assert!(qeval(&mut polar, "neq(-1,+1)"));
    assert!(qnull(&mut polar, "neq(-1,-1)"));
    assert!(qnull(&mut polar, "neq(-1,-1.0)"));
    assert!(qnull(&mut polar, "neq(\"aa\", \"aa\")"));
    assert!(qeval(&mut polar, "neq(\"ab\", \"aa\")"));

    let mut query = polar.new_query("eq(bob, bob)", false).unwrap();
    query
        .next_event()
        .expect_err("can't compare unbound variables");

    assert!(qeval(&mut polar, "1.0 == 1"));
    assert!(qeval(&mut polar, "0.99 < 1"));
    assert!(qeval(&mut polar, "1.0 <= 1"));
    assert!(qeval(&mut polar, "1 == 1"));
    assert!(qeval(&mut polar, "0.0 == 0"));
}

#[test]
fn test_arithmetic() {
    let mut polar = Polar::new(None);
    assert!(qeval(&mut polar, "1 + 1 == 2"));
    assert!(qeval(&mut polar, "1 + 1 < 3 and 1 + 1 > 1"));
    assert!(qeval(&mut polar, "2 - 1 == 1"));
    assert!(qeval(&mut polar, "1 - 2 == -1"));
    assert!(qeval(&mut polar, "1.23 - 3.21 == -1.98"));
    assert!(qeval(&mut polar, "2 * 3 == 6"));
    assert!(qeval(&mut polar, "6 / 2 == 3"));
    assert!(qeval(&mut polar, "2 / 6 == 0.3333333333333333"));

    polar
        .load(
            r#"even(0) if cut;
               even(x) if x > 0 and odd(x - 1);
               odd(1) if cut;
               odd(x) if x > 0 and even(x - 1);"#,
        )
        .unwrap();

    assert!(qeval(&mut polar, "even(0)"));
    assert!(qnull(&mut polar, "even(1)"));
    assert!(qeval(&mut polar, "even(2)"));
    assert!(qnull(&mut polar, "even(3)"));
    assert!(qeval(&mut polar, "even(4)"));

    assert!(qnull(&mut polar, "odd(0)"));
    assert!(qeval(&mut polar, "odd(1)"));
    assert!(qnull(&mut polar, "odd(2)"));
    assert!(qeval(&mut polar, "odd(3)"));
    assert!(qnull(&mut polar, "odd(4)"));

    let check_arithmetic_error = |query: &str| {
        let mut query = polar.new_query(query, false).unwrap();
        let error = query.next_event().unwrap_err();
        assert!(matches!(
            error.kind,
            ErrorKind::Runtime(RuntimeError::ArithmeticError { .. })
        ));
    };
    check_arithmetic_error("9223372036854775807 + 1 > 0");
    check_arithmetic_error("-9223372036854775807 - 2 < 0");

    // x / 0 = ∞
    assert_eq!(qvar(&mut polar, "x=1/0", "x"), vec![value!(f64::INFINITY)]);
    assert!(qeval(&mut polar, "1/0 = 2/0"));
    assert!(qnull(&mut polar, "1/0 < 0"));
    assert!(qeval(&mut polar, "1/0 > 0"));
    assert!(qeval(&mut polar, "1/0 > 1e100"));
}

#[test]
fn test_debug() {
    let polar = Polar::new(None);
    polar
        .load("a() if debug(\"a\") and b() and c() and d();\nb();\nc() if debug(\"c\");\nd();\n")
        .unwrap();

    let mut call_num = 0;
    let debug_handler = |s: &str| {
        let rt = match call_num {
            0 => {
                assert_eq!(s, "Welcome to the debugger!\ndebug(\"a\")");
                "over"
            }
            1 => {
                let expected = indoc!(
                    r#"
                001: a() if debug("a") and b() and c() and d();
                                           ^"#
                );
                assert_eq!(s, expected);
                "over"
            }
            2 => {
                let expected = indoc!(
                    r#"
                    001: a() if debug("a") and b() and c() and d();
                                                       ^"#
                );
                assert_eq!(s, expected);
                "over"
            }
            3 => {
                assert_eq!(s, "Welcome to the debugger!\ndebug(\"c\")");
                "over"
            }
            4 => {
                let expected = indoc!(
                    r#"
                    001: a() if debug("a") and b() and c() and d();
                                                               ^"#
                );
                assert_eq!(s, expected);
                "over"
            }
            _ => panic!("Too many calls!"),
        };
        call_num += 1;
        rt.to_string()
    };

    let query = polar.new_query("a()", false).unwrap();
    let _results = query_results!(query, no_results, no_externals, debug_handler);

    let mut call_num = 0;
    let debug_handler = |s: &str| {
        let rt = match call_num {
            0 => {
                assert_eq!(s, "Welcome to the debugger!\ndebug(\"a\")");
                "out"
            }
            1 => {
                assert_eq!(s, "Welcome to the debugger!\ndebug(\"c\")");
                "out"
            }
            2 => {
                let expected = indoc!(
                    r#"
                001: a() if debug("a") and b() and c() and d();
                                                           ^"#
                );
                assert_eq!(s, expected);
                "out"
            }
            _ => panic!("Too many calls: {}", s),
        };
        call_num += 1;
        rt.to_string()
    };
    let query = polar.new_query("a()", false).unwrap();
    let _results = query_results!(query, no_results, no_externals, debug_handler);
}

#[test]
fn test_anonymous_vars() {
    let mut polar = Polar::new(None);
    assert!(qeval(&mut polar, "[1,2,3] = [_,_,_]"));
    assert!(qnull(&mut polar, "[1,2,3] = [__,__,__]"));
}

#[test]
fn test_singleton_vars() {
    let messages = MessageQueue::new();
    let mut polar = Polar::new(Some(messages.clone()));
    polar.register_constant(sym!("X"), term!(true));
    polar.register_constant(sym!("Y"), term!(true));
    polar.load("f(x:X,y:Y,z:Z) if z = z;").unwrap();
    let output = messages.next().unwrap();
    assert!(matches!(&output.kind, MessageKind::Warning));
    // @TODO: How does this work?
    // assert_eq!(
    //     &out,
    //     indoc!(
    //         r#"Singleton variable x is unused or undefined, see <https://docs.oso.dev/using/polar-syntax.html#variables>
    //         001: f(x:X,y:Y,z:Z) if z = z;
    //                   ^
    //            Singleton variable y is unused or undefined, see <https://docs.oso.dev/using/polar-syntax.html#variables>
    //            001: f(x:X,y:Y,z:Z) if z = z;
    //                       ^
    //            Unknown specializer Z
    //            001: f(x:X,y:Y,z:Z) if z = z;
    //                             ^
    //            "#
    //     )
    // );
}

#[test]
fn test_print() {
    let messages = MessageQueue::new();
    let mut polar = Polar::new(Some(messages.clone()));
    polar.load("f(x,y,z) if print(x, y, z);").unwrap();
    assert!(qeval(&mut polar, "f(1, 2, 3)"));
    let output = messages.next().unwrap();
    assert!(matches!(&output.kind, MessageKind::Print));
    assert_eq!(&output.msg, "1, 2, 3");
}

#[test]
fn test_rest_vars() {
    let mut polar = Polar::new(None);

    assert_eq!(
        qvar(&mut polar, "[1,2,3] = [*rest]", "rest"),
        vec![value!([value!(1), value!(2), value!(3)])]
    );
    assert_eq!(
        qvar(&mut polar, "[1,2,3] = [1, *rest]", "rest"),
        vec![value!([value!(2), value!(3)])]
    );
    assert_eq!(
        qvar(&mut polar, "[1,2,3] = [1,2, *rest]", "rest"),
        vec![value!([value!(3)])]
    );
    assert_eq!(
        qvar(&mut polar, "([1,2,3] = [1,2,3, *rest])", "rest"),
        vec![value!([])]
    );
    assert!(qnull(&mut polar, "[1,2,3] = [1,2,3,4, *_rest]"));

    polar
        .load(
            r#"member(x, [x, *_rest]);
               member(x, [_first, *rest]) if member(x, rest);"#,
        )
        .unwrap();
    assert!(qeval(&mut polar, "member(1, [1,2,3])"));
    assert!(qeval(&mut polar, "member(3, [1,2,3])"));
    assert!(qeval(&mut polar, "not member(4, [1,2,3])"));
    assert_eq!(
        qvar(&mut polar, "member(x, [1,2,3])", "x"),
        vec![value!(1), value!(2), value!(3)]
    );

    polar
        .load(
            r#"append([], x, x);
               append([first, *rest], x, [first, *tail]) if append(rest, x, tail);"#,
        )
        .unwrap();
    assert!(qeval(&mut polar, "append([], [], [])"));
    assert!(qeval(&mut polar, "append([], [1,2,3], [1,2,3])"));
    assert!(qeval(&mut polar, "append([1], [2,3], [1,2,3])"));
    assert!(qeval(&mut polar, "append([1,2], [3], [1,2,3])"));
    assert!(qeval(&mut polar, "append([1,2,3], [], [1,2,3])"));
    assert!(qeval(&mut polar, "not append([1,2,3], [4], [1,2,3])"));
}

#[test]
fn test_in() {
    let mut polar = Polar::new(None);
    polar.load("f(x, y) if x in y;").unwrap();
    assert!(qeval(&mut polar, "f(1, [1,2,3])"));
    assert_eq!(
        qvar(&mut polar, "f(x, [1,2,3])", "x"),
        vec![value!(1), value!(2), value!(3)]
    );
    assert!(qeval(&mut polar, "4 in [1,2,3] or 1 in [1,2,3]"));

    // strange test case but it's important to note that this returns
    // 3 results, with 1 binding each
    let query = polar.new_query("f(1, [x,y,z])", false).unwrap();
    let results = query_results!(query);
    assert_eq!(results.len(), 3);
    assert_eq!(
        results[0].0.get(&Symbol("x".to_string())).unwrap().clone(),
        value!(1)
    );
    assert_eq!(
        results[1].0.get(&Symbol("y".to_string())).unwrap().clone(),
        value!(1)
    );
    assert_eq!(
        results[2].0.get(&Symbol("z".to_string())).unwrap().clone(),
        value!(1)
    );

    assert!(qeval(&mut polar, "f({a:1}, [{a:1}, b, c])"));

    let mut query = polar.new_query("a in {a:1}", false).unwrap();
    let e = query.next_event().unwrap_err();
    assert!(matches!(
        e.kind,
        ErrorKind::Runtime(RuntimeError::TypeError { .. })
    ));

    // negation
    assert!(qeval(&mut polar, "not (4 in [1,2,3])"));
    assert!(qnull(&mut polar, "not (1 in [1,2,3])"));
    assert!(qnull(&mut polar, "not (2 in [1,2,3])"));
    assert!(qnull(&mut polar, "not (3 in [1,2,3])"));

    // empty lists
    assert!(qnull(&mut polar, "x in []"));
    assert!(qnull(&mut polar, "1 in []"));
    assert!(qnull(&mut polar, "\"foo\" in []"));
    assert!(qnull(&mut polar, "[] in []"));
    assert!(qeval(&mut polar, "not x in []"));
    assert!(qeval(&mut polar, "not 1 in []"));
    assert!(qeval(&mut polar, "not \"foo\" in []"));
    assert!(qeval(&mut polar, "not [] in []"));
}

#[test]
fn test_matches() {
    let mut polar = Polar::new(None);
    qnull(&mut polar, "x = 1 and y = 2 and x matches y");
    qeval(&mut polar, "x = 1 and y = 1 and x matches y");

    qeval(&mut polar, "x = {foo: 1} and x matches {foo: 1}");
    qnull(&mut polar, "x = {foo: 1} and x matches {foo: 1, bar: 2}");
    qnull(&mut polar, "x = {foo: 1} and x matches {foo: 2}");
}

#[test]
fn test_keyword_bug() {
    let polar = Polar::new(None);
    let result = polar.load("g(a) if a.new(b);").unwrap_err();
    assert!(matches!(
        result.kind,
        ErrorKind::Parse(ParseError::ReservedWord { .. })
    ));

    let result = polar.load("f(a) if a.in(b);").unwrap_err();
    assert!(matches!(
        result.kind,
        ErrorKind::Parse(ParseError::ReservedWord { .. })
    ));

    let result = polar.load("cut(a) if a;").unwrap_err();
    assert!(matches!(
        result.kind,
        ErrorKind::Parse(ParseError::ReservedWord { .. })
    ));

    let result = polar.load("debug(a) if a;").unwrap_err();
    assert!(matches!(
        result.kind,
        ErrorKind::Parse(ParseError::ReservedWord { .. })
    ));
}

/// Test that rule heads work correctly when unification or specializers are used.
#[test]
fn test_unify_rule_head() {
    let mut polar = Polar::new(None);
    assert!(matches!(
        polar
            .load("f(Foo{a: 1});")
            .expect_err("Must have a parser error"),
        PolarError { kind: ErrorKind::Parse(_), .. }
    ));

    assert!(matches!(
        polar
            .load("f(new Foo{a: Foo{a: 1}});")
            .expect_err("Must have a parser error"),
        PolarError { kind: ErrorKind::Parse(_), .. }
    ));

    assert!(matches!(
        polar
            .load("f(x: new Foo{a: 1});")
            .expect_err("Must have a parser error"),
        PolarError { kind: ErrorKind::Parse(_), .. }
    ));

    assert!(matches!(
        polar
            .load("f(x: Foo{a: new Foo{a: 1}});")
            .expect_err("Must have a parser error"),
        PolarError { kind: ErrorKind::Parse(_), .. }
    ));

    polar.register_constant(sym!("Foo"), term!(true));
    polar.load("f(_: Foo{a: 1}, x) if x = 1;").unwrap();
    polar.load("g(_: Foo{a: Foo{a: 1}}, x) if x = 1;").unwrap();

    let query = polar.new_query("f(new Foo{a: 1}, x)", false).unwrap();
    let (results, _externals) = query_results_with_externals(query);
    assert_eq!(results[0].0.get(&sym!("x")).unwrap(), &value!(1));

    let query = polar
        .new_query("g(new Foo{a: new Foo{a: 1}}, x)", false)
        .unwrap();
    let (results, _externals) = query_results_with_externals(query);
    assert_eq!(results[0].0.get(&sym!("x")).unwrap(), &value!(1));
}

/// Test that cut commits to all choice points before the cut, not just the last.
#[test]
fn test_cut() {
    let mut polar = Polar::new(None);
    polar.load("a(x) if x = 1 or x = 2;").unwrap();
    polar.load("b(x) if x = 3 or x = 4;").unwrap();
    polar.load("bcut(x) if x = 3 or x = 4 and cut;").unwrap();

    polar.load("c(a, b) if a(a) and b(b) and cut;").unwrap();
    polar.load("c_no_cut(a, b) if a(a) and b(b);").unwrap();
    polar
        .load("c_partial_cut(a, b) if a(a) and bcut(b);")
        .unwrap();
    polar
        .load("c_another_partial_cut(a, b) if a(a) and cut and b(b);")
        .unwrap();

    // Ensure we return multiple results without a cut.
    assert!(qvars(&mut polar, "c_no_cut(a, b)", &["a", "b"]).len() > 1);

    // Ensure that only one result is returned when cut is at the end.
    assert_eq!(
        qvars(&mut polar, "c(a, b)", &["a", "b"]),
        vec![vec![value!(1), value!(3)]]
    );

    // Make sure that cut in `bcut` does not affect `c_partial_cut`.
    // If it did, only one result would be returned, [1, 3].
    assert_eq!(
        qvars(&mut polar, "c_partial_cut(a, b)", &["a", "b"]),
        vec![vec![value!(1), value!(3)], vec![value!(2), value!(3)]]
    );

    // Make sure cut only affects choice points before it.
    assert_eq!(
        qvars(&mut polar, "c_another_partial_cut(a, b)", &["a", "b"]),
        vec![vec![value!(1), value!(3)], vec![value!(1), value!(4)]]
    );

    polar.load("f(x) if (x = 1 and cut) or x = 2;").unwrap();
    assert_eq!(qvar(&mut polar, "f(x)", "x"), vec![value!(1)]);
    assert!(qeval(&mut polar, "f(1)"));
    assert!(qeval(&mut polar, "f(2)"));
}

#[test]
fn test_forall() {
    let mut polar = Polar::new(None);
    polar
        .load("all_ones(l) if forall(item in l, item = 1);")
        .unwrap();

    assert!(qeval(&mut polar, "all_ones([1])"));
    assert!(qeval(&mut polar, "all_ones([1, 1, 1])"));
    assert!(qnull(&mut polar, "all_ones([1, 2, 1])"));

    polar
        .load("not_ones(l) if forall(item in l, item != 1);")
        .unwrap();
    assert!(qnull(&mut polar, "not_ones([1])"));
    assert!(qeval(&mut polar, "not_ones([2, 3, 4])"));

    assert!(qnull(&mut polar, "forall(x = 2 or x = 3, x != 2)"));
    assert!(qnull(&mut polar, "forall(x = 2 or x = 3, x != 3)"));
    assert!(qeval(&mut polar, "forall(x = 2 or x = 3, x = 2 or x = 3)"));
    assert!(qeval(&mut polar, "forall(x = 1, x = 1)"));
    assert!(qeval(&mut polar, "forall(x in [2, 3, 4], x > 1)"));

    polar.load("g(1);").unwrap();
    polar.load("g(2);").unwrap();
    polar.load("g(3);").unwrap();

    assert!(qeval(&mut polar, "forall(g(x), x in [1, 2, 3])"));

    polar.load("allow(_: {x: 1}, y) if y = 1;").unwrap();
    polar.load("allow(_: {y: 1}, y) if y = 2;").unwrap();
    polar.load("allow(_: {z: 1}, y) if y = 3;").unwrap();

    assert!(qeval(
        &mut polar,
        "forall(allow({x: 1, y: 1, z: 1}, y), y in [1, 2, 3])"
    ));
}

#[test]
fn test_emoji_policy() {
    let mut polar = Polar::new(None);
    polar
        .load(
            r#"
                    👩‍🔧("👩‍🦰");
                    allow(👩, "🛠", "🚙") if 👩‍🔧(👩);
                "#,
        )
        .unwrap();
    assert!(qeval(&mut polar, r#"allow("👩‍🦰","🛠","🚙")"#));
    assert!(qnull(&mut polar, r#"allow("🧟","🛠","🚙")"#));
}

#[test]
/// Check that boolean expressions evaluate without requiring "= true".
fn test_boolean_expression() {
    let mut polar = Polar::new(None);

    // Succeeds because t is true.
    assert!(qeval(&mut polar, "a = {t: true, f: false} and a.t"));
    // Fails because `f` is not true.
    assert!(qnull(&mut polar, "a = {t: true, f: false} and a.f"));
    // Fails because `f` is not true.
    assert!(qnull(&mut polar, "a = {t: true, f: false} and a.f and a.t"));
    // Succeeds because `t` is true.
    assert!(qeval(
        &mut polar,
        "a = {t: true, f: false} and (a.f or a.t)"
    ));

    assert!(qeval(&mut polar, "true"));
    assert!(qnull(&mut polar, "false"));
    assert!(qeval(&mut polar, "a = true and a"));
    assert!(qnull(&mut polar, "a = false and a"));
}

#[test]
fn test_float_parsing() {
    let mut polar = Polar::new(None);
    assert_eq!(qvar(&mut polar, "x=1+1", "x"), vec![value!(2)]);
    assert_eq!(qvar(&mut polar, "x=1+1.5", "x"), vec![value!(2.5)]);
    assert_eq!(qvar(&mut polar, "x=1.e+5", "x"), vec![value!(1e5)]);
    assert_eq!(qvar(&mut polar, "x=1e+5", "x"), vec![value!(1e5)]);
    assert_eq!(qvar(&mut polar, "x=1e5", "x"), vec![value!(1e5)]);
    assert_eq!(qvar(&mut polar, "x=1e-5", "x"), vec![value!(1e-5)]);
    assert_eq!(qvar(&mut polar, "x=1.e-5", "x"), vec![value!(1e-5)]);
    assert_eq!(qvar(&mut polar, "x=1.0e+15", "x"), vec![value!(1e15)]);
    assert_eq!(qvar(&mut polar, "x=1.0E+15", "x"), vec![value!(1e15)]);
    assert_eq!(qvar(&mut polar, "x=1.0e-15", "x"), vec![value!(1e-15)]);
}
#[test]
fn test_assignment() {
    let mut polar = Polar::new(None);
    assert!(qeval(&mut polar, "x := 5 and x == 5"));
    let mut query = polar.new_query("x := 5 and x := 6", false).unwrap();
    let e = query.next_event().unwrap_err();
    assert!(matches!(
        e.kind,
        ErrorKind::Runtime(RuntimeError::TypeError {
            msg: s,
            ..
        }) if s == "Can only assign to unbound variables, x is bound to value 5."
    ));
    assert!(qnull(&mut polar, "x := 5 and x > 6"));
    assert!(qeval(&mut polar, "x := y and y = 6 and x = 6"));

    // confirm old syntax -> parse error
    let e = polar.load("f(x) := g(x)").unwrap_err();
    assert!(matches!(
        e.kind,
        ErrorKind::Parse(ParseError::UnrecognizedToken { .. })
    ));
}
