use super::types::*;

//pub fn rewrite(term: Term) -> Term {
// if you see a . expression, replace it with a temp symbol?.
// insert the . expression, with a third argument (the temp symbol) in the nearest enclosing AND/OR/NOT expression.
// if the expression isn't in an AND/OR/NOT we have to wrap it in an empty AND.

// this is only for rules
// foo(a.b) => foo(?temp) := .(a,b,?temp);
// foo(a.b) := hi;
// => foo(?temp) := .(a,b,?temp), hi;

// for queries
// foo(a.b) => .(a,b,x), foo(?x)
//

// the two cases
// rewrite the head of a rule,
// rewrite (a query or the body of a rule)
//}

pub fn rewrite_term(term: Term) -> Term {
    term
}

pub fn rewrite_rule(rule: Rule) -> Rule {
    //let Rule { name, params, body } = rule;

    //let body = rewrite_term(body);
    //Rule { name, params, body }
    rule
}
