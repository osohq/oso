mod mock_externals;

use indoc::indoc;
use maplit::btreemap;
use permute::permute;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::iter::FromIterator;

use polar_core::{
    error::*,
    events::*,
    messages::*,
    polar::{Polar, Query},
    sym, term,
    terms::*,
    traces::*,
    value, values,
};

type QueryResults = Vec<(HashMap<Symbol, Value>, Option<TraceResult>)>;
use mock_externals::MockExternal;

fn no_results(
    _: u64,
    _: Term,
    _: Symbol,
    _: Option<Vec<Term>>,
    _: Option<BTreeMap<Symbol, Term>>,
) -> Option<Term> {
    None
}

fn print_messages(msg: &Message) {
    eprintln!("[{:?}] {}", msg.kind, msg.msg);
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

fn query_results<F, G, H, I, J, K>(
    mut query: Query,
    mut external_call_handler: F,
    mut make_external_handler: H,
    mut external_isa_handler: I,
    mut external_is_subspecializer_handler: J,
    mut debug_handler: G,
    mut message_handler: K,
) -> QueryResults
where
    F: FnMut(u64, Term, Symbol, Option<Vec<Term>>, Option<BTreeMap<Symbol, Term>>) -> Option<Term>,
    G: FnMut(&str) -> String,
    H: FnMut(u64, Term),
    I: FnMut(Term, Symbol) -> bool,
    J: FnMut(u64, Symbol, Symbol) -> bool,
    K: FnMut(&Message),
{
    let mut results = vec![];
    loop {
        let event = query.next_event().unwrap();
        while let Some(msg) = query.next_message() {
            message_handler(&msg)
        }
        match event {
            QueryEvent::Done { .. } => break,
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
                kwargs,
            } => {
                query
                    .call_result(
                        call_id,
                        external_call_handler(call_id, instance, attribute, args, kwargs),
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
            } => query
                .question_result(call_id, external_isa_handler(instance, class_tag))
                .unwrap(),
            QueryEvent::ExternalIsSubSpecializer {
                call_id,
                instance_id,
                left_class_tag,
                right_class_tag,
            } => query
                .question_result(
                    call_id,
                    external_is_subspecializer_handler(
                        instance_id,
                        left_class_tag,
                        right_class_tag,
                    ),
                )
                .unwrap(),
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
            print_messages,
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
            print_messages,
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
            print_messages,
        )
    };
    ($query:expr, @msgs $message_handler:expr) => {
        query_results(
            $query,
            no_results,
            no_externals,
            no_isa,
            no_is_subspecializer,
            no_debug,
            $message_handler,
        )
    };
}

fn query_results_with_externals(query: Query) -> (QueryResults, MockExternal) {
    let mock = RefCell::new(MockExternal::new());
    (
        query_results(
            query,
            |a, b, c, d, e| mock.borrow_mut().external_call(a, b, c, d, e),
            |a, b| mock.borrow_mut().make_external(a, b),
            |a, b| mock.borrow_mut().external_isa(a, b),
            |a, b, c| mock.borrow_mut().external_is_subspecializer(a, b, c),
            no_debug,
            print_messages,
        ),
        mock.into_inner(),
    )
}

#[track_caller]
#[must_use = "test results need to be asserted"]
fn eval(p: &mut Polar, query_str: &str) -> bool {
    let q = p.new_query(query_str, false).unwrap();
    !query_results!(q).is_empty()
}

#[track_caller]
fn qeval(p: &mut Polar, query_str: &str) {
    assert!(eval(p, query_str));
}

#[track_caller]
#[must_use = "test results need to be asserted"]
fn null(p: &mut Polar, query_str: &str) -> bool {
    let q = p.new_query(query_str, false).unwrap();
    query_results!(q).is_empty()
}

#[track_caller]
fn qnull(p: &mut Polar, query_str: &str) {
    assert!(null(p, query_str));
}

#[track_caller]
fn qext(p: &mut Polar, query_str: &str, external_results: Vec<Value>, expected_len: usize) {
    let mut external_results: Vec<Term> = external_results
        .into_iter()
        .map(Term::new_from_test)
        .rev()
        .collect();
    let q = p.new_query(query_str, false).unwrap();
    assert_eq!(
        query_results!(q, |_, _, _, _, _| external_results.pop()).len(),
        expected_len
    );
}

#[track_caller]
#[must_use = "test results need to be asserted"]
fn var(p: &mut Polar, query_str: &str, var: &str) -> Vec<Value> {
    let q = p.new_query(query_str, false).unwrap();
    query_results!(q)
        .iter()
        .map(|(r, _)| &r[&sym!(var)])
        .cloned()
        .collect()
}

#[track_caller]
fn qvar(p: &mut Polar, query_str: &str, variable: &str, expected: Vec<Value>) {
    assert_eq!(var(p, query_str, variable), expected);
}

#[track_caller]
#[must_use = "test results need to be asserted"]
fn vars(p: &mut Polar, query_str: &str, vars: &[&str]) -> Vec<Vec<Value>> {
    let q = p.new_query(query_str, false).unwrap();
    query_results!(q)
        .iter()
        .map(|bindings| {
            vars.iter()
                .map(|&var| bindings.0.get(&Symbol(var.to_string())).unwrap().clone())
                .collect()
        })
        .collect()
}

#[track_caller]
fn qvars(p: &mut Polar, query_str: &str, variables: &[&str], expected: Vec<Vec<Value>>) {
    assert_eq!(vars(p, query_str, variables), expected);
}

#[track_caller]
fn _qruntime(p: &mut Polar, query_str: &str) -> ErrorKind {
    p.new_query(query_str, false)
        .unwrap()
        .next_event()
        .unwrap_err()
        .kind
}

macro_rules! qruntime {
    ($query:tt, $err:pat $(, $cond:expr)?) => {
        assert!(matches!(_qruntime(&mut Polar::new(), $query), ErrorKind::Runtime($err) $(if $cond)?));
    };

    ($polar:expr, $query:tt, $err:pat $(, $cond:expr)?) => {
        assert!(matches!(_qruntime($polar, $query), ErrorKind::Runtime($err) $(if $cond)?));
    };
}

macro_rules! qparse {
    ($query:expr, $err:pat) => {
        assert!(matches!(
            Polar::new().load_str($query).unwrap_err().kind,
            ErrorKind::Parse($err)
        ));
    };
}

type TestResult = Result<(), PolarError>;

/// Adapted from <http://web.cse.ohio-state.edu/~stiff.4/cse3521/prolog-resolution.html>
#[test]
fn test_functions() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f(1);
           f(2);
           g(1);
           g(2);
           h(2);
           k(x) if f(x) and h(x) and g(x);"#,
    )?;
    qnull(&mut p, "k(1)");
    qeval(&mut p, "k(2)");
    qnull(&mut p, "k(3)");
    qvar(&mut p, "k(a)", "a", values![2]);
    Ok(())
}

/// Adapted from <http://web.cse.ohio-state.edu/~stiff.4/cse3521/prolog-resolution.html>
#[test]
fn test_jealous() -> TestResult {
    let p = Polar::new();
    p.load_str(
        r#"loves("vincent", "mia");
           loves("marcellus", "mia");
           jealous(a, b) if loves(a, c) and loves(b, c);"#,
    )?;
    let q = p.new_query("jealous(who, of)", false)?;
    let results = query_results!(q);
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
    Ok(())
}

#[test]
fn test_trace() -> TestResult {
    let p = Polar::new();
    p.load_str(
        r#"f(x) if x = 1 and x = 1;
           f(y) if y = 1;"#,
    )?;
    let q = p.new_query("f(1)", true)?;
    let results = query_results!(q);
    let trace = results[0].1.as_ref().unwrap();
    let expected = indoc!(
        r#"
        f(1) [
          f(x) if x = 1 and x = 1; [
              x = 1 []
              x = 1 []
          ]
        ]
        "#
    );
    assert_eq!(trace.formatted, expected);
    let trace = results[1].1.as_ref().unwrap();
    let expected = indoc!(
        r#"
        f(1) [
          f(y) if y = 1; [
              y = 1 []
          ]
        ]
        "#
    );
    assert_eq!(trace.formatted, expected);
    Ok(())
}

#[test]
fn test_nested_rule() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f(x) if g(x);
           g(x) if h(x);
           h(2);
           g(x) if j(x);
           j(4);"#,
    )?;
    qeval(&mut p, "f(2)");
    qnull(&mut p, "f(3)");
    qeval(&mut p, "f(4)");
    qeval(&mut p, "j(4)");
    Ok(())
}

/// A functions permutation that is known to fail.
#[test]
fn test_bad_functions() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f(2);
           f(1);
           g(1);
           g(2);
           h(2);
           k(x) if f(x) and h(x) and g(x);"#,
    )?;
    qvar(&mut p, "k(a)", "a", values![2]);
    Ok(())
}

#[test]
fn test_functions_reorder() -> TestResult {
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
        let mut p = Polar::new();

        let mut joined = permutation.join(";");
        joined.push(';');
        p.load_str(&joined)?;

        assert!(
            null(&mut p, "k(1)"),
            "k(1) was true for permutation {:?}",
            &permutation
        );
        assert!(
            eval(&mut p, "k(2)"),
            "k(2) failed for permutation {:?}",
            &permutation
        );
        assert_eq!(
            var(&mut p, "k(a)", "a"),
            values![2],
            "k(a) failed for permutation {:?}",
            &permutation
        );

        println!("permute: {}", i);
    }
    Ok(())
}

#[test]
fn test_results() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"foo(1);
           foo(2);
           foo(3);"#,
    )?;
    qvar(&mut p, "foo(a)", "a", values![1, 2, 3]);
    Ok(())
}

#[test]
fn test_result_permutations() -> TestResult {
    let parts = vec![
        (1, "foo(1)"),
        (2, "foo(2)"),
        (3, "foo(3)"),
        (4, "foo(4)"),
        (5, "foo(5)"),
    ];
    for permutation in permute(parts).into_iter() {
        eprintln!("{:?}", permutation);
        let mut p = Polar::new();
        let (results, rules): (Vec<_>, Vec<_>) = permutation.into_iter().unzip();
        p.load_str(&format!("{};", rules.join(";")))?;
        qvar(
            &mut p,
            "foo(a)",
            "a",
            results.into_iter().map(|v| value!(v)).collect::<Vec<_>>(),
        );
    }
    Ok(())
}

#[test]
fn test_multi_arg_method_ordering() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"bar(2, 1);
           bar(1, 1);
           bar(1, 2);
           bar(2, 2);"#,
    )?;
    qvars(
        &mut p,
        "bar(a, b)",
        &["a", "b"],
        values![[2, 1], [1, 1], [1, 2], [2, 2]],
    );
    Ok(())
}

#[test]
fn test_no_applicable_rules() -> TestResult {
    let mut p = Polar::new();
    qnull(&mut p, "f()");
    p.load_str("f(_);")?;
    qnull(&mut p, "f()");
    Ok(())
}

/// From Aït-Kaci's WAM tutorial (1999), page 34.
#[test]
fn test_ait_kaci_34() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"a() if b(x) and c(x);
           b(x) if e(x);
           c(1);
           e(x) if f(x);
           e(x) if g(x);
           f(2);
           g(1);"#,
    )?;
    qeval(&mut p, "a()");
    Ok(())
}

#[test]
fn test_constants() -> TestResult {
    let mut p = Polar::new();
    {
        let mut kb = p.kb.write().unwrap();
        kb.constant(sym!("one"), term!(1));
        kb.constant(sym!("two"), term!(2));
        kb.constant(sym!("three"), term!(3));
    }
    p.load_str(
        r#"one(x) if one = one and one = x and x < two;
           two(x) if one < x and two = two and two = x and two < three;
           three(x) if three = three and three = x;"#,
    )?;
    qeval(&mut p, "one(1)");
    qnull(&mut p, "two(1)");
    qeval(&mut p, "two(2)");
    qnull(&mut p, "three(2)");
    qeval(&mut p, "three(3)");
    Ok(())
}

#[test]
fn test_not() -> TestResult {
    let mut p = Polar::new();
    p.load_str("odd(1); even(2);")?;
    qeval(&mut p, "odd(1)");
    qnull(&mut p, "not odd(1)");
    qnull(&mut p, "even(1)");
    qeval(&mut p, "not even(1)");
    qnull(&mut p, "odd(2)");
    qeval(&mut p, "not odd(2)");
    qeval(&mut p, "even(2)");
    qnull(&mut p, "not even(2)");
    qnull(&mut p, "even(3)");
    qeval(&mut p, "not even(3)");

    p.load_str(
        r#"f(x) if not a(x);
           a(1);
           b(2);
           g(x) if not (a(x) or b(x));"#,
    )?;

    qnull(&mut p, "f(1)");
    qeval(&mut p, "f(2)");

    qnull(&mut p, "g(1)");
    qnull(&mut p, "g(2)");
    qeval(&mut p, "g(3)");
    qnull(&mut p, "g(x) and x=3"); // this should fail because unbound x means g(x) always fails
    qeval(&mut p, "x=3 and g(x)");

    p.load_str("h(x) if not (not (x = 1 or x = 3) or x = 3);")?;
    qeval(&mut p, "h(1)");
    qnull(&mut p, "h(2)");
    qnull(&mut p, "h(3)");

    qeval(&mut p, "d = {x: 1} and not d.x = 2");

    // Negate And with unbound variable.
    p.load_str("i(x,y) if not (y = 2 and x = 1);")?;
    qvar(&mut p, "i(2,y)", "y", values![sym!("_y_44")]);

    // Negate Or with unbound variable.
    p.load_str("j(x,y) if not (y = 2 or x = 1);")?;
    qnull(&mut p, "j(2, y)");
    Ok(())
}

#[test]
fn test_and() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f(1);
           f(2);"#,
    )?;
    qeval(&mut p, "f(1) and f(2)");
    qnull(&mut p, "f(1) and f(2) and f(3)");
    Ok(())
}

#[test]
fn test_equality() {
    let mut p = Polar::new();
    qeval(&mut p, "1 = 1");
    qnull(&mut p, "1 = 2");
}

#[test]
fn test_lookup() {
    qeval(&mut Polar::new(), "{x: 1}.x = 1");
}

#[test]
fn test_instance_lookup() {
    // Q: Not sure if this should be allowed? I can't get (new a{x: 1}).x to parse, but that might
    // be the only thing we should permit
    qext(&mut Polar::new(), "new a(x: 1).x = 1", values![1], 1);
}

/// Adapted from <http://web.cse.ohio-state.edu/~stiff.4/cse3521/prolog-resolution.html>
#[test]
fn test_retries() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f(1);
           f(2);
           g(1);
           g(2);
           h(2);
           k(x) if f(x) and h(x) and g(x);
           k(3);"#,
    )?;
    qnull(&mut p, "k(1)");
    qeval(&mut p, "k(2)");
    qvar(&mut p, "k(a)", "a", values![2, 3]);
    qeval(&mut p, "k(3)");
    Ok(())
}

#[test]
fn test_two_rule_bodies_not_nested() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f(x) if a(x);
           f(1);"#,
    )?;
    qvar(&mut p, "f(x)", "x", values![1]);
    Ok(())
}

#[test]
fn test_two_rule_bodies_nested() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f(x) if a(x);
           f(1);
           a(x) if g(x);"#,
    )?;
    qvar(&mut p, "f(x)", "x", values![1]);
    Ok(())
}

#[test]
fn test_unify_and() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f(x, y) if a(x) and y = 2;
           a(1);
           a(3);"#,
    )?;
    qvar(&mut p, "f(x, y)", "x", values![1, 3]);
    qvar(&mut p, "f(x, y)", "y", values![2, 2]);
    Ok(())
}

#[test]
fn test_symbol_lookup() {
    let mut p = Polar::new();
    qvar(&mut p, "{x: 1}.x = res", "res", values![1]);
    qvar(&mut p, "{x: 1} = d and d.x = res", "res", values![1]);
}

#[test]
fn test_or() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f(x) if a(x) or b(x);
           a(1);
           b(3);"#,
    )?;
    qvar(&mut p, "f(x)", "x", values![1, 3]);
    qeval(&mut p, "f(1)");
    qnull(&mut p, "f(2)");
    qeval(&mut p, "f(3)");

    p.load_str(
        r#"g(x) if a(x) or b(x) or c(x);
           c(5);"#,
    )?;
    qvar(&mut p, "g(x)", "x", values![1, 3, 5]);
    qeval(&mut p, "g(1)");
    qnull(&mut p, "g(2)");
    qeval(&mut p, "g(3)");
    qeval(&mut p, "g(5)");
    Ok(())
}

#[test]
fn test_dict_specializers() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f({x: 1});
           g(_: {x: 1});"#,
    )?;
    // Test unifying dicts against our rules.
    qeval(&mut p, "f({x: 1})");
    qnull(&mut p, "f({x: 1, y: 2})");
    qnull(&mut p, "f(1)");
    qnull(&mut p, "f({})");
    qnull(&mut p, "f({x: 2})");
    qnull(&mut p, "f({y: 1})");

    qeval(&mut p, "g({x: 1})");
    qeval(&mut p, "g({x: 1, y: 2})");
    qnull(&mut p, "g(1)");
    qnull(&mut p, "g({})");
    qnull(&mut p, "g({x: 2})");
    qnull(&mut p, "g({y: 1})");

    // Test unifying & isa-ing instances against our rules.
    qnull(&mut p, "f(new a(x: 1))");
    qext(&mut p, "g(new a(x: 1))", values![1, 1], 1);
    qnull(&mut p, "f(new a())");
    qnull(&mut p, "f(new a(x: {}))");
    qext(&mut p, "g(new a(x: 2))", values![2, 2], 0);
    qext(&mut p, "g(new a(y: 2, x: 1))", values![1, 1], 1);
    Ok(())
}

#[test]
fn test_non_instance_specializers() -> TestResult {
    let mut p = Polar::new();
    p.load_str("f(x: 1) if x = 1;")?;
    qeval(&mut p, "f(1)");
    qnull(&mut p, "f(2)");

    p.load_str("g(x: 1, y: [x]) if y = [1];")?;
    qeval(&mut p, "g(1, [1])");
    qnull(&mut p, "g(1, [2])");

    p.load_str("h(x: {y: y}, x.y) if y = 1;")?;
    qeval(&mut p, "h({y: 1}, 1)");
    qnull(&mut p, "h({y: 1}, 2)");
    Ok(())
}

#[test]
fn test_bindings() -> TestResult {
    let mut p = Polar::new();
    qvar(&mut p, "x=1", "x", values![1]);
    qvar(&mut p, "x=x", "x", values![sym!("x")]);
    qvar(&mut p, "x=y and y=x", "x", values![sym!("y")]);

    p.load_str(
        r#"f(x) if x = y and g(y);
           g(y) if y = 1;"#,
    )?;
    qvar(&mut p, "f(x)", "x", values![1]);
    Ok(())
}

#[test]
fn test_lookup_derefs() -> TestResult {
    let p = Polar::new();
    p.load_str(
        r#"f(x) if x = y and g(y);
           g(y) if new Foo().get(y) = y;"#,
    )?;
    let q = p.new_query("f(1)", false)?;
    let mut foo_lookups = vec![term!(1)];
    let mock_foo = |_, _, _, args: Option<Vec<Term>>, _| {
        // check the argument is bound to an integer
        assert!(matches!(args.unwrap()[0].value(), Value::Number(_)));
        foo_lookups.pop()
    };

    let results = query_results!(q, mock_foo);
    assert!(foo_lookups.is_empty());
    assert_eq!(results.len(), 1);

    let mut foo_lookups = vec![term!(1)];
    let mock_foo = |_, _, _, args: Option<Vec<Term>>, _| {
        assert!(matches!(args.unwrap()[0].value(), Value::Number(_)));
        foo_lookups.pop()
    };
    let q = p.new_query("f(2)", false)?;
    let results = query_results!(q, mock_foo);
    assert!(results.is_empty());
    Ok(())
}

#[test]
fn unify_predicates() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f(g(_x));
           k(x) if h(g(x), g(x));
           h(g(1), g(1));"#,
    )?;
    qeval(&mut p, "f(g(1))");
    qnull(&mut p, "f(1)");
    qeval(&mut p, "k(1)");
    Ok(())
}

/// Test that rules are executed in the correct order.
#[test]
fn test_rule_order() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"a("foo");
           a("bar");
           a("baz");"#,
    )?;
    qvar(&mut p, "a(x)", "x", values!["foo", "bar", "baz"]);
    Ok(())
}

#[test]
fn test_load_str_with_query() -> TestResult {
    let p = Polar::new();
    p.load_str(
        r#"f(1);
           f(2);
           ?= f(1);
           ?= not f(3);"#,
    )?;
    while let Some(q) = p.next_inline_query(false) {
        assert_eq!(query_results!(q).len(), 1);
    }
    Ok(())
}

/// Test using a constructor with positional + kwargs.
#[test]
fn test_make_external() -> TestResult {
    let q = Polar::new().new_query("x = new Bar(1, a: 2, b: 3)", false)?;
    let mock_make_bar = |_, constructor: Term| match constructor.value() {
        Value::Call(Call {
            name,
            args,
            kwargs: Some(kwargs),
        }) if name == &sym!("Bar")
            && args == &vec![term!(1)]
            && kwargs == &btreemap! {sym!("a") => term!(2), sym!("b") => term!(3)} => {}
        _ => panic!("Expected call with args and kwargs"),
    };
    let results = query_results!(q, no_results, mock_make_bar, no_debug);
    assert_eq!(results.len(), 1);
    Ok(())
}

/// Test external call with positional + kwargs.
#[test]
fn test_external_call() -> TestResult {
    let p = Polar::new();
    p.register_constant(sym!("Foo"), term!(true));
    let mut foo_lookups = vec![term!(1)];

    let q = p.new_query("(new Foo()).bar(1, a: 2, b: 3) = 1", false)?;

    let mock_foo_lookup =
        |_, _, _, args: Option<Vec<Term>>, kwargs: Option<BTreeMap<Symbol, Term>>| {
            assert_eq!(args.unwrap()[0], term!(1));
            assert_eq!(
                kwargs.unwrap(),
                btreemap! {sym!("a") => term!(2), sym!("b") => term!(3)}
            );
            foo_lookups.pop()
        };
    let results = query_results!(q, mock_foo_lookup);
    assert_eq!(results.len(), 1);
    assert!(foo_lookups.is_empty());
    Ok(())
}
#[test]
#[ignore] // ignore because this take a LONG time (could consider lowering the goal limit)
#[should_panic(expected = "Goal count exceeded! MAX_EXECUTED_GOALS = 10000")]
fn test_infinite_loop() {
    let mut p = Polar::new();
    p.load_str("f(x) if f(x);").unwrap();
    qeval(&mut p, "f(1)");
}

#[test]
fn test_comparisons() -> TestResult {
    let mut p = Polar::new();

    // <
    p.load_str("lt(x, y) if x < y;")?;
    qnull(&mut p, "lt(1,1)");
    qeval(&mut p, "lt(1,2)");
    qnull(&mut p, "lt(2,1)");
    qnull(&mut p, "lt(+1,-1)");
    qeval(&mut p, "lt(-1,+1)");
    qnull(&mut p, "lt(-1,-1)");
    qeval(&mut p, "lt(-2,-1)");
    qeval(&mut p, "lt(1019,1e19)");
    qnull(&mut p, "lt(1e19,1019)");
    qnull(&mut p, "lt(9007199254740992,9007199254740992)"); // identical
    qnull(&mut p, "lt(9007199254740992,9007199254740992.0)"); // equal
    qnull(&mut p, "lt(9007199254740992,9007199254740993.0)"); // indistinguishable
    qeval(&mut p, "lt(9007199254740992,9007199254740994.0)"); // distinguishable
    qeval(&mut p, "lt(\"aa\",\"ab\")");
    qnull(&mut p, "lt(\"aa\",\"aa\")");

    // <=
    p.load_str("leq(x, y) if x <= y;")?;
    qeval(&mut p, "leq(1,1)");
    qeval(&mut p, "leq(1,2)");
    qnull(&mut p, "leq(2,1)");
    qnull(&mut p, "leq(+1,-1)");
    qeval(&mut p, "leq(-1,+1)");
    qeval(&mut p, "leq(-1,-1)");
    qeval(&mut p, "leq(-2,-1)");
    qeval(&mut p, "leq(\"aa\",\"aa\")");
    qeval(&mut p, "leq(\"aa\",\"ab\")");
    qnull(&mut p, "leq(\"ab\",\"aa\")");

    // >
    p.load_str("gt(x, y) if x > y;")?;
    qnull(&mut p, "gt(1,1)");
    qnull(&mut p, "gt(1,2)");
    qeval(&mut p, "gt(2,1)");
    qeval(&mut p, "gt(+1,-1)");
    qnull(&mut p, "gt(-1,+1)");
    qnull(&mut p, "gt(-1,-1)");
    qeval(&mut p, "gt(-1,-2)");
    qeval(&mut p, "gt(\"ab\",\"aa\")");
    qnull(&mut p, "gt(\"aa\",\"aa\")");

    // >=
    p.load_str("geq(x, y) if x >= y;")?;
    qeval(&mut p, "geq(1,1)");
    qnull(&mut p, "geq(1,2)");
    qeval(&mut p, "geq(2,1)");
    qeval(&mut p, "geq(2,1)");
    qeval(&mut p, "geq(+1,-1)");
    qnull(&mut p, "geq(-1,+1)");
    qeval(&mut p, "geq(-1,-1)");
    qeval(&mut p, "geq(-1,-1.0)");
    qeval(&mut p, "geq(\"ab\",\"aa\")");
    qeval(&mut p, "geq(\"aa\",\"aa\")");

    // ==
    p.load_str("eq(x, y) if x == y;")?;
    qeval(&mut p, "eq(1,1)");
    qnull(&mut p, "eq(1,2)");
    qnull(&mut p, "eq(2,1)");
    qnull(&mut p, "eq(-1,+1)");
    qeval(&mut p, "eq(-1,-1)");
    qeval(&mut p, "eq(-1,-1.0)");
    qnull(&mut p, "eq(1019,1e19)");
    qnull(&mut p, "eq(1e19,1019)");
    qeval(&mut p, "eq(9007199254740992,9007199254740992)"); // identical
    qeval(&mut p, "eq(9007199254740992,9007199254740992.0)"); // equal
    qeval(&mut p, "eq(9007199254740992,9007199254740993.0)"); // indistinguishable
    qnull(&mut p, "eq(9007199254740992,9007199254740994.0)"); // distinguishable
    qeval(&mut p, "eq(\"aa\", \"aa\")");
    qnull(&mut p, "eq(\"ab\", \"aa\")");

    // !=
    p.load_str("neq(x, y) if x != y;")?;
    qnull(&mut p, "neq(1,1)");
    qeval(&mut p, "neq(1,2)");
    qeval(&mut p, "neq(2,1)");
    qeval(&mut p, "neq(-1,+1)");
    qnull(&mut p, "neq(-1,-1)");
    qnull(&mut p, "neq(-1,-1.0)");
    qnull(&mut p, "neq(\"aa\", \"aa\")");
    qeval(&mut p, "neq(\"ab\", \"aa\")");

    let mut q = p.new_query("eq(bob, bob)", false)?;
    q.next_event().expect_err("can't compare unbound variables");

    qeval(&mut p, "1.0 == 1");
    qeval(&mut p, "0.99 < 1");
    qeval(&mut p, "1.0 <= 1");
    qeval(&mut p, "1 == 1");
    qeval(&mut p, "0.0 == 0");
    Ok(())
}

#[test]
fn test_modulo_and_remainder() {
    let mut p = Polar::new();
    qeval(&mut p, "1 mod 1 == 0");
    qeval(&mut p, "1 rem 1 == 0");
    qeval(&mut p, "1 mod -1 == 0");
    qeval(&mut p, "1 rem -1 == 0");
    qeval(&mut p, "0 mod 1 == 0");
    qeval(&mut p, "0 rem 1 == 0");
    qeval(&mut p, "0 mod -1 == 0");
    qeval(&mut p, "0 rem -1 == 0");
    qruntime!("1 mod 0 = x", RuntimeError::ArithmeticError { .. });
    qruntime!("1 rem 0 = x", RuntimeError::ArithmeticError { .. });
    let res = var(&mut p, "1 mod 0.0 = x", "x")[0].clone();
    if let Value::Number(Numeric::Float(x)) = res {
        assert!(x.is_nan());
    } else {
        panic!();
    }
    let res = var(&mut p, "1 rem 0.0 = x", "x")[0].clone();
    if let Value::Number(Numeric::Float(x)) = res {
        assert!(x.is_nan());
    } else {
        panic!();
    }

    // From http://www.lispworks.com/documentation/lw50/CLHS/Body/f_mod_r.htm.
    qeval(&mut p, "-1 rem 5 == -1");
    qeval(&mut p, "-1 mod 5 == 4");
    qeval(&mut p, "13 mod 4 == 1");
    qeval(&mut p, "13 rem 4 == 1");
    qeval(&mut p, "-13 mod 4 == 3");
    qeval(&mut p, "-13 rem 4 == -1");
    qeval(&mut p, "13 mod -4 == -3");
    qeval(&mut p, "13 rem -4 == 1");
    qeval(&mut p, "-13 mod -4 == -1");
    qeval(&mut p, "-13 rem -4 == -1");
    qeval(&mut p, "13.4 mod 1 == 0.40000000000000036");
    qeval(&mut p, "13.4 rem 1 == 0.40000000000000036");
    qeval(&mut p, "-13.4 mod 1 == 0.5999999999999996");
    qeval(&mut p, "-13.4 rem 1 == -0.40000000000000036");
}

#[test]
fn test_arithmetic() -> TestResult {
    let mut p = Polar::new();
    qeval(&mut p, "1 + 1 == 2");
    qeval(&mut p, "1 + 1 < 3 and 1 + 1 > 1");
    qeval(&mut p, "2 - 1 == 1");
    qeval(&mut p, "1 - 2 == -1");
    qeval(&mut p, "1.23 - 3.21 == -1.98");
    qeval(&mut p, "2 * 3 == 6");
    qeval(&mut p, "6 / 2 == 3");
    qeval(&mut p, "2 / 6 == 0.3333333333333333");

    p.load_str(
        r#"even(0) if cut;
           even(x) if x > 0 and odd(x - 1);
           odd(1) if cut;
           odd(x) if x > 0 and even(x - 1);"#,
    )?;

    qeval(&mut p, "even(0)");
    qnull(&mut p, "even(1)");
    qeval(&mut p, "even(2)");
    qnull(&mut p, "even(3)");
    qeval(&mut p, "even(4)");

    qnull(&mut p, "odd(0)");
    qeval(&mut p, "odd(1)");
    qnull(&mut p, "odd(2)");
    qeval(&mut p, "odd(3)");
    qnull(&mut p, "odd(4)");

    qruntime!("9223372036854775807 + 1 > 0", RuntimeError::ArithmeticError { .. });
    qruntime!("-9223372036854775807 - 2 < 0", RuntimeError::ArithmeticError { .. });

    // x / 0 = ∞
    qvar(&mut p, "x=1/0", "x", values![f64::INFINITY]);
    qeval(&mut p, "1/0 = 2/0");
    qnull(&mut p, "1/0 < 0");
    qeval(&mut p, "1/0 > 0");
    qeval(&mut p, "1/0 > 1e100");
    Ok(())
}

#[test]
fn test_debug() -> TestResult {
    let p = Polar::new();
    p.load_str(indoc!(
        r#"a() if debug("a") and b() and c() and d();
           b();
           c() if debug("c");
           d();"#
    ))?;

    let mut call_num = 0;
    let debug_handler = |s: &str| {
        let rt = match call_num {
            0 => {
                let expected = indoc!(
                    r#"
                    QUERY: debug(), BINDINGS: {}

                    001: a() if debug("a") and b() and c() and d();
                                ^
                    002: b();
                    003: c() if debug("c");
                    004: d();
                    "#
                );
                assert_eq!(s, expected);
                "over"
            }
            1 => {
                let expected = indoc!(
                    r#"
                    QUERY: b(), BINDINGS: {}

                    001: a() if debug("a") and b() and c() and d();
                                               ^
                    002: b();
                    003: c() if debug("c");
                    004: d();
                    "#
                );
                assert_eq!(s, expected);
                "over"
            }
            2 => {
                let expected = indoc!(
                    r#"
                    QUERY: c(), BINDINGS: {}

                    001: a() if debug("a") and b() and c() and d();
                                                       ^
                    002: b();
                    003: c() if debug("c");
                    004: d();
                    "#
                );
                assert_eq!(s, expected);
                "over"
            }
            3 => {
                let expected = indoc!(
                    r#"
                    QUERY: debug(), BINDINGS: {}

                    001: a() if debug("a") and b() and c() and d();
                    002: b();
                    003: c() if debug("c");
                                ^
                    004: d();
                    "#
                );
                assert_eq!(s, expected);
                "over"
            }
            4 => {
                let expected = indoc!(
                    r#"
                    QUERY: d(), BINDINGS: {}

                    001: a() if debug("a") and b() and c() and d();
                                                               ^
                    002: b();
                    003: c() if debug("c");
                    004: d();
                    "#
                );
                assert_eq!(s, expected);
                "over"
            }
            _ => panic!("Too many calls!"),
        };
        call_num += 1;
        rt.to_string()
    };

    let q = p.new_query("a()", false)?;
    let _results = query_results!(q, no_results, no_externals, debug_handler);

    let p = Polar::new();
    p.load_str(indoc!(
        r#"a() if debug() and b() and c() and d();
           a() if 5 = 5;
           b() if 1 = 1 and 2 = 2;
           c() if 3 = 3 and 4 = 4;
           d();"#
    ))?;

    let mut call_num = 0;
    let debug_handler = |s: &str| {
        let rt = match call_num {
            0 => {
                assert_eq!(s.lines().next().unwrap(), "QUERY: debug(), BINDINGS: {}");
                "step"
            }
            1 => {
                assert_eq!(s.lines().next().unwrap(), "QUERY: b(), BINDINGS: {}");
                "step"
            }
            2 => {
                assert_eq!(
                    s.lines().next().unwrap(),
                    "QUERY: 1 = 1 and 2 = 2, BINDINGS: {}"
                );
                "out"
            }
            3 => {
                assert_eq!(s.lines().next().unwrap(), "QUERY: c(), BINDINGS: {}");
                "step"
            }
            4 => {
                assert_eq!(
                    s.lines().next().unwrap(),
                    "QUERY: 3 = 3 and 4 = 4, BINDINGS: {}"
                );
                "step"
            }
            5 => {
                assert_eq!(s.lines().next().unwrap(), "QUERY: 3 = 3, BINDINGS: {}");
                "out"
            }
            6 => {
                assert_eq!(s.lines().next().unwrap(), "QUERY: d(), BINDINGS: {}");
                "over"
            }
            7 => {
                assert_eq!(s.lines().next().unwrap(), "QUERY: 5 = 5, BINDINGS: {}");
                "c"
            }
            _ => panic!("Too many calls: {}", s),
        };
        call_num += 1;
        rt.to_string()
    };
    let q = p.new_query("a()", false)?;
    let _results = query_results!(q, no_results, no_externals, debug_handler);
    Ok(())
}

#[test]
fn test_debug_in_inverter() {
    let polar = Polar::new();
    polar.load_str("a() if not debug();").unwrap();
    let mut call_num = 0;
    let debug_handler = |s: &str| {
        let rt = match call_num {
            0 => {
                let expected = indoc!(
                    r#"
                    QUERY: debug(), BINDINGS: {}

                    001: a() if not debug();
                                    ^
                    "#
                );
                assert_eq!(s, expected);
                "over"
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
    let mut p = Polar::new();
    qeval(&mut p, "[1,2,3] = [_,_,_]");
    qnull(&mut p, "[1,2,3] = [__,__,__]");
}

#[test]
fn test_singleton_vars() -> TestResult {
    let p = Polar::new();
    p.register_constant(sym!("X"), term!(true));
    p.register_constant(sym!("Y"), term!(true));
    p.load_str("f(x:X,y:Y,z:Z) if z = z;")?;
    let output = p.next_message().unwrap();
    assert!(matches!(&output.kind, MessageKind::Warning));
    assert_eq!(
        &output.msg,
        "Singleton variable x is unused or undefined, see <https://docs.osohq.com/using/polar-syntax.html#variables>\n001: f(x:X,y:Y,z:Z) if z = z;\n       ^"
    );
    let output = p.next_message().unwrap();
    assert!(matches!(&output.kind, MessageKind::Warning));
    assert_eq!(
        &output.msg,
        "Singleton variable y is unused or undefined, see <https://docs.osohq.com/using/polar-syntax.html#variables>\n001: f(x:X,y:Y,z:Z) if z = z;\n           ^"
    );
    let output = p.next_message().unwrap();
    assert!(matches!(&output.kind, MessageKind::Warning));
    assert_eq!(
        &output.msg,
        "Unknown specializer Z\n001: f(x:X,y:Y,z:Z) if z = z;\n                 ^"
    );
    Ok(())
}

#[test]
fn test_print() -> TestResult {
    // TODO: If POLAR_LOG is on this test will fail.
    let p = Polar::new();
    p.load_str("f(x,y,z) if print(x, y, z);")?;
    let message_handler = |output: &Message| {
        assert!(matches!(&output.kind, MessageKind::Print));
        assert_eq!(&output.msg, "1, 2, 3");
    };
    let q = p.new_query("f(1, 2, 3)", false)?;
    let _results = query_results!(q, @msgs message_handler);
    Ok(())
}

#[test]
fn test_unknown_specializer_suggestions() -> TestResult {
    let p = Polar::new();
    p.load_str("f(s: string) if s;")?;
    let msg = p.next_message().unwrap();
    assert!(matches!(&msg.kind, MessageKind::Warning));
    assert_eq!(
        &msg.msg,
        "Unknown specializer string, did you mean String?\n001: f(s: string) if s;\n          ^"
    );
    Ok(())
}

#[test]
fn test_rest_vars() -> TestResult {
    let mut p = Polar::new();
    qvar(&mut p, "[1,2,3] = [*rest]", "rest", vec![value!([1, 2, 3])]);
    qvar(&mut p, "[1,2,3] = [1,*rest]", "rest", vec![value!([2, 3])]);
    qvar(&mut p, "[1,2,3] = [1,2,*rest]", "rest", vec![value!([3])]);
    qvar(&mut p, "[1,2,3] = [1,2,3,*rest]", "rest", vec![value!([])]);
    qnull(&mut p, "[1,2,3] = [1,2,3,4,*_rest]");

    p.load_str(
        r#"member(x, [x, *_rest]);
           member(x, [_first, *rest]) if member(x, rest);"#,
    )?;
    qeval(&mut p, "member(1, [1,2,3])");
    qeval(&mut p, "member(3, [1,2,3])");
    qeval(&mut p, "not member(4, [1,2,3])");
    qvar(&mut p, "member(x, [1,2,3])", "x", values![1, 2, 3]);

    p.load_str(
        r#"append([], x, x);
           append([first, *rest], x, [first, *tail]) if append(rest, x, tail);"#,
    )?;
    qeval(&mut p, "append([], [], [])");
    qeval(&mut p, "append([], [1,2,3], [1,2,3])");
    qeval(&mut p, "append([1], [2,3], [1,2,3])");
    qeval(&mut p, "append([1,2], [3], [1,2,3])");
    qeval(&mut p, "append([1,2,3], [], [1,2,3])");
    qeval(&mut p, "not append([1,2,3], [4], [1,2,3])");
    Ok(())
}

#[test]
fn test_in_op() -> TestResult {
    let mut p = Polar::new();
    p.load_str("f(x, y) if x in y;")?;
    qeval(&mut p, "f(1, [1,2,3])");
    qvar(&mut p, "f(x, [1,2,3])", "x", values![1, 2, 3]);

    // Failure.
    qnull(&mut p, "4 in [1,2,3]");
    qeval(&mut p, "4 in [1,2,3] or 1 in [1,2,3]");

    // Make sure we scan the whole list.
    let q = p.new_query("1 in [1, 2, x, 1]", false)?;
    let results = query_results!(q);
    assert_eq!(results.len(), 3);
    assert!(results[0].0.is_empty());
    assert_eq!(
        results[1].0.get(&Symbol("x".to_string())).unwrap().clone(),
        value!(1)
    );
    assert!(results[2].0.is_empty());

    // This returns 3 results, with 1 binding each.
    let q = p.new_query("f(1, [x,y,z])", false)?;
    let results = query_results!(q);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0[&sym!("x")], value!(1));
    assert_eq!(results[1].0[&sym!("y")], value!(1));
    assert_eq!(results[2].0[&sym!("z")], value!(1));

    qeval(&mut p, "f({a:1}, [{a:1}, b, c])");

    // Negation.
    qeval(&mut p, "not (4 in [1,2,3])");
    qnull(&mut p, "not (1 in [1,2,3])");
    qnull(&mut p, "not (2 in [1,2,3])");
    qnull(&mut p, "not (3 in [1,2,3])");

    // Nothing is in an empty list.
    qnull(&mut p, "x in []");
    qnull(&mut p, "1 in []");
    qnull(&mut p, "\"foo\" in []");
    qnull(&mut p, "[] in []");
    qeval(&mut p, "not x in []");
    qeval(&mut p, "not 1 in []");
    qeval(&mut p, "not \"foo\" in []");
    qeval(&mut p, "not [] in []");
    Ok(())
}

#[test]
fn test_matches() {
    let mut p = Polar::new();
    qnull(&mut p, "1 matches 2");
    qeval(&mut p, "1 matches 1");
    // This doesn't fail because `y` is parsed as an unknown specializer
    // qnull(&mut p, "x = 1 and y = 2 and x matches y");
    qeval(&mut p, "x = {foo: 1} and x matches {foo: 1}");
    qeval(&mut p, "x = {foo: 1, bar: 2} and x matches {foo: 1}");
    qnull(&mut p, "x = {foo: 1} and x matches {foo: 1, bar: 2}");
    qnull(&mut p, "x = {foo: 1} and x matches {foo: 2}");
}

#[test]
fn test_keyword_bug() {
    qparse!("g(a) if a.new(b);", ParseError::ReservedWord { .. });
    qparse!("f(a) if a.in(b);", ParseError::ReservedWord { .. });
    qparse!("cut(a) if a;", ParseError::ReservedWord { .. });
    qparse!("debug(a) if a;", ParseError::ReservedWord { .. });
}

/// Test that rule heads work correctly when unification or specializers are used.
#[test]
fn test_unify_rule_head() -> TestResult {
    qparse!("f(Foo{a: 1});", ParseError::UnrecognizedToken { .. });
    qparse!("f(new Foo(a: Foo{a: 1}));", ParseError::UnrecognizedToken { .. });
    qparse!("f(x: new Foo(a: 1));", ParseError::ReservedWord { .. });
    qparse!("f(x: Foo{a: new Foo(a: 1)});", ParseError::ReservedWord { .. });

    let p = Polar::new();
    p.register_constant(sym!("Foo"), term!(true));
    p.load_str(
        r#"f(_: Foo{a: 1}, x) if x = 1;
           g(_: Foo{a: Foo{a: 1}}, x) if x = 1;"#,
    )?;

    let q = p.new_query("f(new Foo(a: 1), x)", false)?;
    let (results, _externals) = query_results_with_externals(q);
    assert_eq!(results[0].0[&sym!("x")], value!(1));

    let q = p.new_query("g(new Foo(a: new Foo(a: 1)), x)", false)?;
    let (results, _externals) = query_results_with_externals(q);
    assert_eq!(results[0].0[&sym!("x")], value!(1));
    Ok(())
}

/// Test that cut commits to all choice points before the cut, not just the last.
#[test]
fn test_cut() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"a(x) if x = 1 or x = 2;
           b(x) if x = 3 or x = 4;
           bcut(x) if x = 3 or x = 4 and cut;
           c(a, b) if a(a) and b(b) and cut;
           c_no_cut(a, b) if a(a) and b(b);
           c_partial_cut(a, b) if a(a) and bcut(b);
           c_another_partial_cut(a, b) if a(a) and cut and b(b);"#,
    )?;

    // Ensure we return multiple results without a cut.
    qvars(
        &mut p,
        "c_no_cut(a, b)",
        &["a", "b"],
        values![[1, 3], [1, 4], [2, 3], [2, 4]],
    );

    // Ensure that only one result is returned when cut is at the end.
    qvars(&mut p, "c(a, b)", &["a", "b"], values![[1, 3]]);

    // Make sure that cut in `bcut` does not affect `c_partial_cut`.
    // If it did, only one result would be returned, [1, 3].
    qvars(
        &mut p,
        "c_partial_cut(a, b)",
        &["a", "b"],
        values![[1, 3], [2, 3]],
    );

    // Make sure cut only affects choice points before it.
    qvars(
        &mut p,
        "c_another_partial_cut(a, b)",
        &["a", "b"],
        values![[1, 3], [1, 4]],
    );

    p.load_str("f(x) if (x = 1 and cut) or x = 2;")?;
    qvar(&mut p, "f(x)", "x", values![1]);
    qeval(&mut p, "f(1)");
    qeval(&mut p, "f(2)");
    Ok(())
}

#[test]
fn test_forall() -> TestResult {
    let mut p = Polar::new();
    p.load_str("all_ones(l) if forall(item in l, item = 1);")?;

    qeval(&mut p, "all_ones([1])");
    qeval(&mut p, "all_ones([1, 1, 1])");
    qnull(&mut p, "all_ones([1, 2, 1])");

    p.load_str("not_ones(l) if forall(item in l, item != 1);")?;
    qnull(&mut p, "not_ones([1])");
    qeval(&mut p, "not_ones([2, 3, 4])");

    qnull(&mut p, "forall(x = 2 or x = 3, x != 2)");
    qnull(&mut p, "forall(x = 2 or x = 3, x != 3)");
    qeval(&mut p, "forall(x = 2 or x = 3, x = 2 or x = 3)");
    qeval(&mut p, "forall(x = 1, x = 1)");
    qeval(&mut p, "forall(x in [2, 3, 4], x > 1)");

    p.load_str(
        r#"g(1);
           g(2);
           g(3);"#,
    )?;
    qeval(&mut p, "forall(g(x), x in [1, 2, 3])");

    p.load_str(
        r#"allow(_: {x: 1}, y) if y = 1;
           allow(_: {y: 1}, y) if y = 2;
           allow(_: {z: 1}, y) if y = 3;"#,
    )?;
    qeval(
        &mut p,
        "forall(allow({x: 1, y: 1, z: 1}, y), y in [1, 2, 3])",
    );
    Ok(())
}

#[test]
fn test_emoji_policy() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"
                    👩‍🔧("👩‍🦰");
                    allow(👩, "🛠", "🚙") if 👩‍🔧(👩);
                "#,
    )?;
    qeval(&mut p, r#"allow("👩‍🦰","🛠","🚙")"#);
    qnull(&mut p, r#"allow("🧟","🛠","🚙")"#);
    Ok(())
}

#[test]
/// Check that boolean expressions evaluate without requiring "= true".
fn test_boolean_expression() {
    let mut p = Polar::new();
    qeval(&mut p, "a = {t: true, f: false} and a.t"); // Succeeds because t is true.
    qnull(&mut p, "a = {t: true, f: false} and a.f"); // Fails because `f` is not true.
    qnull(&mut p, "a = {t: true, f: false} and a.f and a.t"); // Fails because `f` is not true.
    qeval(&mut p, "a = {t: true, f: false} and (a.f or a.t)"); // Succeeds because `t` is true.

    qeval(&mut p, "true");
    qnull(&mut p, "false");
    qeval(&mut p, "a = true and a");
    qnull(&mut p, "a = false and a");
}

#[test]
fn test_float_parsing() {
    let mut p = Polar::new();
    qvar(&mut p, "x=1+1", "x", values![2]);
    qvar(&mut p, "x=1+1.5", "x", values![2.5]);
    qvar(&mut p, "x=1.e+5", "x", values![1e5]);
    qvar(&mut p, "x=1e+5", "x", values![1e5]);
    qvar(&mut p, "x=1e5", "x", values![1e5]);
    qvar(&mut p, "x=1e-5", "x", values![1e-5]);
    qvar(&mut p, "x=1.e-5", "x", values![1e-5]);
    qvar(&mut p, "x=1.0e+15", "x", values![1e15]);
    qvar(&mut p, "x=1.0E+15", "x", values![1e15]);
    qvar(&mut p, "x=1.0e-15", "x", values![1e-15]);
}

#[test]
fn test_assignment() {
    let mut p = Polar::new();
    qeval(&mut p, "x := 5 and x == 5");
    qruntime!("x := 5 and x := 6", RuntimeError::TypeError { msg: s, .. },
        s == "Can only assign to unbound variables, x is bound to value 5.");
    qnull(&mut p, "x := 5 and x > 6");
    qeval(&mut p, "x := y and y = 6 and x = 6");

    // confirm old syntax -> parse error
    qparse!("f(x) := g(x);", ParseError::UnrecognizedToken { .. });
}

#[test]
fn test_rule_index() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f(1, 1, "x");
           f(1, 1, "y");
           f(1, x, "y") if x = 2;
           f(1, 2, {b: "y"});
           f(1, 3, {c: "z"});"#,
    )?;
    // Exercise the index.
    qeval(&mut p, r#"f(1, 1, "x")"#);
    qeval(&mut p, r#"f(1, 1, "y")"#);
    qvar(&mut p, r#"f(1, x, "y")"#, "x", values![1, 2]);
    qnull(&mut p, r#"f(1, 1, "z")"#);
    qnull(&mut p, r#"f(1, 2, "x")"#);
    qeval(&mut p, r#"f(1, 2, {b: "y"})"#);
    qeval(&mut p, r#"f(1, 3, {c: "z"})"#);
    Ok(())
}

#[test]
fn test_fib() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"fib(0, 1) if cut;
           fib(1, 1) if cut;
           fib(n, a+b) if fib(n-1, a) and fib(n-2, b);"#,
    )?;
    qvar(&mut p, r#"fib(0, x)"#, "x", values![1]);
    qvar(&mut p, r#"fib(1, x)"#, "x", values![1]);
    qvar(&mut p, r#"fib(2, x)"#, "x", values![2]);
    qvar(&mut p, r#"fib(3, x)"#, "x", values![3]);
    qvar(&mut p, r#"fib(4, x)"#, "x", values![5]);
    qvar(&mut p, r#"fib(5, x)"#, "x", values![8]);
    Ok(())
}

#[test]
fn test_duplicated_rule() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"f(1);
           f(1);"#,
    )?;
    qvar(&mut p, "f(x)", "x", values![1, 1]);
    Ok(())
}

#[test]
fn test_numeric_applicability() -> TestResult {
    let mut p = Polar::new();
    let eps = f64::EPSILON;
    let nan1 = f64::NAN;
    let nan2 = f64::from_bits(f64::NAN.to_bits() | 1);
    assert!(eps.is_normal() && nan1.is_nan() && nan2.is_nan());
    p.register_constant(sym!("eps"), term!(eps));
    p.register_constant(sym!("nan1"), term!(nan1));
    p.register_constant(sym!("nan2"), term!(nan2));
    p.load_str(
        r#"f(0);
           f(1);
           f(9007199254740991); # (1 << 53) - 1
           f(9007199254740992); # (1 << 53)
           f(9223372036854775807); # i64::MAX
           f(-9223372036854775807); # i64::MIN + 1
           f(9223372036854776000.0); # i64::MAX as f64
           f(nan1); # NaN"#,
    )?;
    qeval(&mut p, "f(0)");
    qeval(&mut p, "f(0.0)");
    qnull(&mut p, "f(eps)");
    qeval(&mut p, "f(1)");
    qeval(&mut p, "f(1.0)");
    qnull(&mut p, "f(1.0000000000000002)");
    qnull(&mut p, "f(9007199254740990)");
    qnull(&mut p, "f(9007199254740990.0)");
    qeval(&mut p, "f(9007199254740991)");
    qeval(&mut p, "f(9007199254740991.0)");
    qeval(&mut p, "f(9007199254740992)");
    qeval(&mut p, "f(9007199254740992.0)");
    qeval(&mut p, "f(9223372036854775807)");
    qeval(&mut p, "f(-9223372036854775807)");
    qeval(&mut p, "f(9223372036854776000.0)");
    qnull(&mut p, "f(nan1)");
    qnull(&mut p, "f(nan2)");
    Ok(())
}

#[test]
fn test_external_unify() -> TestResult {
    let p = Polar::new();
    p.load_str(
        r#"selfEq(x) if eq(x, x);
           eq(x, x);"#,
    )?;

    let q = p.new_query("selfEq(new Foo())", false)?;
    let (results, _externals) = query_results_with_externals(q);
    assert_eq!(results.len(), 1);

    let q = p.new_query("eq(new Foo(), new Foo())", false)?;
    let (results, _externals) = query_results_with_externals(q);
    assert!(results.is_empty());
    Ok(())
}

#[test]
fn test_list_results() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"delete([x, *xs], x, ys) if delete(xs, x, ys);
           delete([x, *xs], z, [x, *ys]) if
               x != z and delete(xs, z, ys);
           delete([], _, []);"#,
    )?;
    qeval(&mut p, "delete([1,2,3,2,1],2,[1,3,1])");
    qvar(
        &mut p,
        "delete([1,2,3,2,1],2,result)",
        "result",
        vec![value!([1, 3, 1])],
    );

    qvar(&mut p, "[1,2] = [1,*ys]", "ys", vec![value!([2])]);
    qvar(
        &mut p,
        "[1,2,*xs] = [1,*ys] and [1,2,3] = [1,*ys]",
        "xs",
        vec![value!([3])],
    );
    qvar(
        &mut p,
        "[1,2,*xs] = [1,*ys] and [1,2,3] = [1,*ys]",
        "ys",
        vec![value!([2, 3])],
    );
    qvar(
        &mut p,
        "[1,2,*xs] = [1,*ys] and [1,2,3] = [1,*ys]",
        "ys",
        vec![value!([2, 3])],
    );
    qeval(&mut p, "xs = [2] and [1,2] = [1, *xs]");
    qnull(&mut p, "[1, 2] = [2, *ys]");
    Ok(())
}

#[test]
fn test_expressions_in_lists() -> TestResult {
    let mut p = Polar::new();
    p.load_str(
        r#"scope(actor: Dictionary, "read", "Person", filters) if
               filters = ["id", "=", actor.id];"#,
    )?;
    qeval(
        &mut p,
        r#"scope({id: 1}, "read", "Person", ["id", "=", 1])"#,
    );
    qnull(
        &mut p,
        r#"scope({id: 2}, "read", "Person", ["id", "=", 1])"#,
    );
    qnull(
        &mut p,
        r#"scope({id: 1}, "read", "Person", ["not_id", "=", 1])"#,
    );
    qeval(&mut p, r#"d = {x: 1} and [d.x, 1+1] = [1, 2]"#);
    qvar(
        &mut p,
        r#"d = {x: 1} and [d.x, 1+1] = [1, *rest]"#,
        "rest",
        vec![value!([2])],
    );
    Ok(())
}

#[test]
fn test_list_matches() {
    let mut p = Polar::new();
    qeval(&mut p, "[] matches []");
    qnull(&mut p, "[1] matches []");
    qnull(&mut p, "[] matches [1]");
    qnull(&mut p, "[1, 2] matches [1, 2, 3]");
    qnull(&mut p, "[2, 1] matches [1, 2]");
    qeval(&mut p, "[1, 2, 3] matches [1, 2, 3]");
    qnull(&mut p, "[1, 2, 3] matches [1, 2]");

    qnull(&mut p, "[x] matches []");
    qnull(&mut p, "[] matches [x]");
    qnull(&mut p, "[1, 2, x] matches [1, 2]");
    qnull(&mut p, "[1, x] matches [1, 2, 3]");
    qnull(&mut p, "[2, x] matches [1, 2]");
    qvar(&mut p, "[1, 2, x] matches [1, 2, 3]", "x", values![3]);
    qnull(&mut p, "[1, 2, 3] matches [1, x]");

    qvar(&mut p, "[] matches [*ys]", "ys", vec![value!([])]);
    qvar(&mut p, "[*xs] matches []", "xs", vec![value!([])]);
    qvar(&mut p, "[*xs] matches [1]", "xs", vec![value!([1])]);
    qvar(&mut p, "[1] matches [*ys]", "ys", vec![value!([1])]);
    qeval(&mut p, "[*xs] matches [*ys]");
    qvar(&mut p, "[1,2,3] matches [1,2,*xs]", "xs", vec![value!([3])]);
    qvar(
        &mut p,
        "[1,2,*xs] matches [1,2,3,*ys]",
        "xs",
        vec![value!([3, Value::RestVariable(Symbol::new("ys"))])],
    );
}

#[test]
fn error_on_binding_expressions_and_patterns_to_variables() -> TestResult {
    qruntime!("x matches y", RuntimeError::TypeError { msg: m, .. }, m == "cannot bind pattern 'y' to 'x'");
    let mut p = Polar::new();
    p.load_str(
        r#"f(x: y) if x = 1;
           g(x: {}) if x = 1;"#,
    )?;
    qruntime!(&mut p, "f(x)", RuntimeError::TypeError { msg: m, .. }, m == "cannot bind pattern 'y' to '_x_1'");
    qruntime!(&mut p, "g(x)", RuntimeError::TypeError { msg: m, .. }, m == "cannot bind pattern '{}' to '_x_2'");
    Ok(())
}

#[test]
fn test_builtin_iterables() {
    let mut p = Polar::new();

    qnull(&mut p, r#"x in """#);
    qvar(
        &mut p,
        "x in \"abc\"",
        "x",
        vec![value!("a"), value!("b"), value!("c")],
    );
    qnull(&mut p, "x in {}");
    qvar(
        &mut p,
        "x in {a: 1, b: 2}",
        "x",
        vec![value!(["a", 1]), value!(["b", 2])],
    );
    qeval(&mut p, r#"["a", 1] in {a: 1, b: 2}"#);
    qvar(
        &mut p,
        "[x, _] in {a: 1, b: 2}",
        "x",
        vec![value!("a"), value!("b")],
    );
    qeval(&mut p, r#"["a", 1] in {a: 1, b: 2}"#);
    qvar(
        &mut p,
        "[_, x] in {a: 1, b: 2}",
        "x",
        vec![value!(1), value!(2)],
    );

    qeval(&mut p, r#""b" in "abc""#);
    qnull(&mut p, r#""d" in "abc""#);
    qeval(&mut p, r#"forall(x in "abc", x in "abacus")"#);
    qnull(&mut p, r#"forall(x in "abcd", x in "abacus")"#);
}
