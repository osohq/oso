//! Inspiration:
//! - https://github.com/rust-unofficial/patterns/blob/607fcb00c4ecb9c6317e4e101e16dc15717758bd/patterns/fold.md
//! - https://docs.rs/rustc-ap-syntax/71.0.0/src/syntax/fold.rs.html
//!
//! Paraphrasing the above, a Folder represents an AST->AST fold; it consumes an AST and returns an
//! AST of the same type.

// TODO(gj): Consider transitioning to a MutVisitor like
//           https://docs.rs/rustc-ap-syntax/645.0.0/src/rustc_ap_syntax/mut_visit.rs.html

use std::collections::BTreeMap;

use crate::rules::*;
use crate::terms::*;

/// Paraphrasing from https://docs.rs/rustc-ap-syntax/71.0.0/src/syntax/fold.rs.html:
///
/// Any additions to this trait should happen in form of a call to a public `fold_*` function that
/// only calls out to the folder again, not other `fold_*` functions. This is a necessary API
/// workaround to the problem of not being able to call out to the super default method in an
/// overridden default method.
///
/// Paraphrasing from https://docs.rs/rustc-ap-syntax/645.0.0/src/rustc_ap_syntax/visit.rs.html:
///
/// Each method of the Folder trait is a hook to be potentially overridden. Each method's default
/// implementation recursively visits the substructure of the input via the corresponding `fold_*`
/// method; e.g., the `fold_rule` method by default calls `folder::fold_rule`.
pub trait Folder: Sized {
    fn fold_number(&mut self, n: Numeric) -> Numeric {
        fold_number(n, self)
    }
    fn fold_string(&mut self, s: String) -> String {
        fold_string(s, self)
    }
    fn fold_boolean(&mut self, b: bool) -> bool {
        fold_boolean(b, self)
    }
    fn fold_instance_id(&mut self, i: u64) -> u64 {
        fold_instance_id(i, self)
    }
    // Class, key, and rule names.
    fn fold_name(&mut self, n: Symbol) -> Symbol {
        fold_name(n, self)
    }
    fn fold_variable(&mut self, v: Symbol) -> Symbol {
        fold_variable(v, self)
    }
    fn fold_rest_variable(&mut self, v: Symbol) -> Symbol {
        fold_variable(v, self)
    }
    fn fold_operator(&mut self, o: Operator) -> Operator {
        fold_operator(o, self)
    }
    fn fold_rule(&mut self, r: Rule) -> Rule {
        fold_rule(r, self)
    }
    fn fold_term(&mut self, t: Term) -> Term {
        fold_term(t, self)
    }
    fn fold_value(&mut self, v: Value) -> Value {
        fold_value(v, self)
    }
    fn fold_external_instance(&mut self, e: ExternalInstance) -> ExternalInstance {
        fold_external_instance(e, self)
    }
    fn fold_instance_literal(&mut self, i: InstanceLiteral) -> InstanceLiteral {
        fold_instance_literal(i, self)
    }
    fn fold_dictionary(&mut self, d: Dictionary) -> Dictionary {
        fold_dictionary(d, self)
    }
    fn fold_pattern(&mut self, p: Pattern) -> Pattern {
        fold_pattern(p, self)
    }
    fn fold_call(&mut self, c: Call) -> Call {
        fold_call(c, self)
    }
    fn fold_list(&mut self, l: TermList) -> TermList {
        fold_list(l, self)
    }
    fn fold_operation(&mut self, o: Operation) -> Operation {
        fold_operation(o, self)
    }
    fn fold_param(&mut self, p: Parameter) -> Parameter {
        fold_param(p, self)
    }
}

pub fn fold_rule<T: Folder>(
    Rule {
        name,
        params,
        body,
        source_info,
        required,
    }: Rule,
    fld: &mut T,
) -> Rule {
    Rule {
        name: fld.fold_name(name),
        params: params.into_iter().map(|p| fld.fold_param(p)).collect(),
        body: fld.fold_term(body),
        source_info,
        required,
    }
}

pub fn fold_term<T: Folder>(t: Term, fld: &mut T) -> Term {
    t.clone_with_value(fld.fold_value(t.value().clone()))
}

pub fn fold_value<T: Folder>(v: Value, fld: &mut T) -> Value {
    match v {
        Value::Number(n) => Value::Number(fld.fold_number(n)),
        Value::String(s) => Value::String(fld.fold_string(s)),
        Value::Boolean(b) => Value::Boolean(fld.fold_boolean(b)),
        Value::ExternalInstance(e) => Value::ExternalInstance(fld.fold_external_instance(e)),
        Value::Dictionary(d) => Value::Dictionary(fld.fold_dictionary(d)),
        Value::Pattern(p) => Value::Pattern(fld.fold_pattern(p)),
        Value::Call(c) => Value::Call(fld.fold_call(c)),
        Value::List(l) => Value::List(fld.fold_list(l)),
        Value::Variable(v) => Value::Variable(fld.fold_variable(v)),
        Value::RestVariable(r) => Value::RestVariable(fld.fold_rest_variable(r)),
        Value::Expression(o) => Value::Expression(fld.fold_operation(o)),
    }
}

pub fn fold_number<T: Folder>(n: Numeric, _fld: &mut T) -> Numeric {
    n
}

pub fn fold_string<T: Folder>(s: String, _fld: &mut T) -> String {
    s
}

pub fn fold_boolean<T: Folder>(b: bool, _fld: &mut T) -> bool {
    b
}

pub fn fold_instance_id<T: Folder>(id: u64, _fld: &mut T) -> u64 {
    id
}

pub fn fold_external_instance<T: Folder>(
    ExternalInstance {
        instance_id,
        constructor,
        repr,
        class_repr,
        class_id,
    }: ExternalInstance,
    fld: &mut T,
) -> ExternalInstance {
    ExternalInstance {
        instance_id: fld.fold_instance_id(instance_id),
        constructor: constructor.map(|t| fld.fold_term(t)),
        repr: repr.map(|r| fld.fold_string(r)),
        class_repr: class_repr.map(|r| fld.fold_string(r)),
        class_id: class_id.map(|id| fld.fold_instance_id(id)),
    }
}

pub fn fold_instance_literal<T: Folder>(
    InstanceLiteral { tag, fields }: InstanceLiteral,
    fld: &mut T,
) -> InstanceLiteral {
    InstanceLiteral {
        tag: fld.fold_name(tag),
        fields: fld.fold_dictionary(fields),
    }
}

pub fn fold_dictionary<T: Folder>(Dictionary { fields }: Dictionary, fld: &mut T) -> Dictionary {
    Dictionary {
        fields: fields
            .into_iter()
            .map(|(k, v)| (fld.fold_name(k), fld.fold_term(v)))
            .collect::<BTreeMap<Symbol, Term>>(),
    }
}

pub fn fold_pattern<T: Folder>(p: Pattern, fld: &mut T) -> Pattern {
    match p {
        Pattern::Dictionary(d) => Pattern::Dictionary(fld.fold_dictionary(d)),
        Pattern::Instance(i) => Pattern::Instance(fld.fold_instance_literal(i)),
    }
}

pub fn fold_call<T: Folder>(Call { name, args, kwargs }: Call, fld: &mut T) -> Call {
    Call {
        name: fld.fold_name(name),
        args: fld.fold_list(args),
        kwargs: kwargs.map(|kwargs| {
            kwargs
                .into_iter()
                .map(|(k, v)| (fld.fold_name(k), fld.fold_term(v)))
                .collect::<BTreeMap<Symbol, Term>>()
        }),
    }
}

pub fn fold_variable<T: Folder>(v: Symbol, _fld: &mut T) -> Symbol {
    v
}

pub fn fold_list<T: Folder>(l: TermList, fld: &mut T) -> TermList {
    l.into_iter()
        .map(|t| fld.fold_term(t))
        .collect::<TermList>()
}

pub fn fold_operator<T: Folder>(o: Operator, _fld: &mut T) -> Operator {
    o
}

pub fn fold_operation<T: Folder>(
    Operation { operator, args }: Operation,
    fld: &mut T,
) -> Operation {
    Operation {
        operator: fld.fold_operator(operator),
        args: fld.fold_list(args),
    }
}

pub fn fold_name<T: Folder>(n: Symbol, _fld: &mut T) -> Symbol {
    n
}

pub fn fold_param<T: Folder>(
    Parameter {
        parameter,
        specializer,
    }: Parameter,
    fld: &mut T,
) -> Parameter {
    Parameter {
        parameter: fld.fold_term(parameter),
        specializer: specializer.map(|t| fld.fold_term(t)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TrivialFolder {}
    impl Folder for TrivialFolder {}

    #[test]
    fn test_fold_term_atomics() {
        let number = value!(1);
        let string = value!("Hi there!");
        let boolean = value!(true);
        let variable = value!(sym!("x"));
        let rest_var = Value::RestVariable(sym!("rest"));
        let list = Value::List(vec![
            term!(number),
            term!(string),
            term!(boolean),
            term!(variable),
            term!(rest_var),
        ]);
        let term = term!(list);
        let mut fld = TrivialFolder {};
        assert_eq!(fld.fold_term(term.clone()), term);
    }

    #[test]
    fn test_fold_term_compounds() {
        let external_instance = term!(Value::ExternalInstance(ExternalInstance {
            instance_id: 1,
            constructor: None,
            repr: None,
            class_repr: None,
            class_id: None,
        }));
        let instance_pattern = term!(value!(Pattern::Instance(InstanceLiteral {
            tag: sym!("d"),
            fields: Dictionary {
                fields: btreemap! {
                    sym!("e") => term!(call!("f", [2])),
                    sym!("g") => term!(op!(Add, term!(3), term!(4))),
                }
            }
        })));
        let dict_pattern = term!(Value::Pattern(Pattern::Dictionary(Dictionary {
            fields: btreemap! {
                sym!("i") => term!("j"),
                sym!("k") => term!("l"),
            },
        })));
        let term = term!(btreemap! {
            sym!("a") => term!(btreemap!{
                sym!("b") => external_instance,
                sym!("c") => instance_pattern,
            }),
            sym!("h") => dict_pattern,
        });
        let mut fld = TrivialFolder {};
        assert_eq!(fld.fold_term(term.clone()), term);
    }

    #[test]
    fn test_fold_rule() {
        let rule = rule!("a", ["b"; instance!("c"), value!("d")] => call!("e", [value!("f")]));
        let mut fld = TrivialFolder {};
        assert_eq!(fld.fold_rule(rule.clone()), rule);
    }
}
