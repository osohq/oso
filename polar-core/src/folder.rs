use std::collections::BTreeMap;

//use super::partial::Constraints;
use super::rules::*;
use super::terms::*;

// The interface here is that each `fold_*` method should call
// the corresponding `noop_` function that only calls the folder
// again, not the other `noop_* functions. This interface was
// taken from the rustc `Folder` implementation, which says:
//   This is a necessary API workaround to the problem of not
//   being able to call out to the super default method
//   in an overridden default method.
pub trait Folder: Sized {
    fn fold_rule(&mut self, r: Rule) -> Rule {
        noop_fold_rule(r, self)
    }
    fn fold_term(&mut self, t: Term) -> Term {
        noop_fold_term(t, self)
    }
    fn fold_value(&mut self, v: Value) -> Value {
        noop_fold_value(v, self)
    }
    fn fold_number(&mut self, n: Numeric) -> Numeric {
        noop_fold_number(n, self)
    }
    fn fold_string(&mut self, s: String) -> String {
        noop_fold_string(s, self)
    }
    fn fold_boolean(&mut self, b: bool) -> bool {
        noop_fold_boolean(b, self)
    }
    fn fold_id(&mut self, i: u64) -> u64 {
        noop_fold_id(i, self)
    }
    fn fold_external_instance(&mut self, e: ExternalInstance) -> ExternalInstance {
        noop_fold_external_instance(e, self)
    }
    fn fold_instance_literal(&mut self, i: InstanceLiteral) -> InstanceLiteral {
        noop_fold_instance_literal(i, self)
    }
    fn fold_dictionary(&mut self, d: Dictionary) -> Dictionary {
        noop_fold_dictionary(d, self)
    }
    fn fold_pattern(&mut self, p: Pattern) -> Pattern {
        noop_fold_pattern(p, self)
    }
    fn fold_call(&mut self, c: Call) -> Call {
        noop_fold_call(c, self)
    }
    fn fold_list(&mut self, l: TermList) -> TermList {
        noop_fold_list(l, self)
    }
    fn fold_variable(&mut self, v: Symbol) -> Symbol {
        noop_fold_variable(v, self)
    }
    fn fold_rest_variable(&mut self, v: Symbol) -> Symbol {
        noop_fold_variable(v, self)
    }
    fn fold_operator(&mut self, o: Operator) -> Operator {
        noop_fold_operator(o, self)
    }
    fn fold_operation(&mut self, o: Operation) -> Operation {
        noop_fold_operation(o, self)
    }
    fn fold_name(&mut self, name: Symbol) -> Symbol {
        noop_fold_name(name, self)
    }
    fn fold_param(&mut self, param: Parameter) -> Parameter {
        noop_fold_param(param, self)
    }
    fn fold_params(&mut self, params: Vec<Parameter>) -> Vec<Parameter> {
        noop_fold_params(params, self)
    }
}

pub fn noop_fold_rule<T: Folder>(Rule { name, params, body }: Rule, fld: &mut T) -> Rule {
    Rule {
        name: fld.fold_name(name),
        params: fld.fold_params(params),
        body: fld.fold_term(body),
    }
}

pub fn noop_fold_term<T: Folder>(t: Term, fld: &mut T) -> Term {
    t.clone_with_value(fld.fold_value(t.value().clone()))
}

pub fn noop_fold_value<T: Folder>(v: Value, fld: &mut T) -> Value {
    match v {
        Value::Number(n) => Value::Number(fld.fold_number(n)),
        Value::String(s) => Value::String(fld.fold_string(s)),
        Value::Boolean(b) => Value::Boolean(fld.fold_boolean(b)),
        Value::ExternalInstance(e) => Value::ExternalInstance(fld.fold_external_instance(e)),
        Value::InstanceLiteral(i) => Value::InstanceLiteral(fld.fold_instance_literal(i)),
        Value::Dictionary(d) => Value::Dictionary(fld.fold_dictionary(d)),
        Value::Pattern(p) => Value::Pattern(fld.fold_pattern(p)),
        Value::Call(c) => Value::Call(fld.fold_call(c)),
        Value::List(l) => Value::List(fld.fold_list(l)),
        Value::Variable(v) => Value::Variable(fld.fold_variable(v)),
        Value::RestVariable(r) => Value::RestVariable(fld.fold_rest_variable(r)),
        Value::Expression(o) => Value::Expression(fld.fold_operation(o)),
        // Value::Partial(_) => {}
        _ => todo!("Fold {:?}", v),
    }
}

pub fn noop_fold_number<T: Folder>(n: Numeric, _fld: &mut T) -> Numeric {
    n
}

pub fn noop_fold_string<T: Folder>(s: String, _fld: &mut T) -> String {
    s
}

pub fn noop_fold_boolean<T: Folder>(b: bool, _fld: &mut T) -> bool {
    b
}

pub fn noop_fold_id<T: Folder>(id: u64, _fld: &mut T) -> u64 {
    id
}

pub fn noop_fold_external_instance<T: Folder>(
    ExternalInstance {
        instance_id,
        constructor,
        repr,
    }: ExternalInstance,
    fld: &mut T,
) -> ExternalInstance {
    ExternalInstance {
        instance_id: fld.fold_id(instance_id),
        constructor: constructor.map(|t| fld.fold_term(t)),
        repr: repr.map(|r| fld.fold_string(r)),
    }
}

pub fn noop_fold_instance_literal<T: Folder>(
    InstanceLiteral { tag, fields }: InstanceLiteral,
    fld: &mut T,
) -> InstanceLiteral {
    InstanceLiteral {
        tag: fld.fold_name(tag),
        fields: fld.fold_dictionary(fields),
    }
}

pub fn noop_fold_dictionary<T: Folder>(
    Dictionary { fields }: Dictionary,
    fld: &mut T,
) -> Dictionary {
    Dictionary {
        fields: fields
            .into_iter()
            .map(|(k, v)| (fld.fold_name(k), fld.fold_term(v)))
            .collect::<BTreeMap<Symbol, Term>>(),
    }
}

pub fn noop_fold_pattern<T: Folder>(p: Pattern, fld: &mut T) -> Pattern {
    match p {
        Pattern::Dictionary(d) => Pattern::Dictionary(fld.fold_dictionary(d)),
        Pattern::Instance(i) => Pattern::Instance(fld.fold_instance_literal(i)),
    }
}

pub fn noop_fold_call<T: Folder>(Call { name, args, kwargs }: Call, fld: &mut T) -> Call {
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

pub fn noop_fold_variable<T: Folder>(v: Symbol, _fld: &mut T) -> Symbol {
    v
}

pub fn noop_fold_list<T: Folder>(l: TermList, fld: &mut T) -> TermList {
    l.into_iter()
        .map(|t| fld.fold_term(t))
        .collect::<TermList>()
}

pub fn noop_fold_operator<T: Folder>(o: Operator, _fld: &mut T) -> Operator {
    o
}

pub fn noop_fold_operation<T: Folder>(
    Operation { operator, args }: Operation,
    fld: &mut T,
) -> Operation {
    Operation {
        operator: fld.fold_operator(operator),
        args: fld.fold_list(args),
    }
}

pub fn noop_fold_name<T: Folder>(name: Symbol, _fld: &mut T) -> Symbol {
    name
}

pub fn noop_fold_param<T: Folder>(
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

pub fn noop_fold_params<T: Folder>(params: Vec<Parameter>, fld: &mut T) -> Vec<Parameter> {
    params
        .into_iter()
        .map(|t| fld.fold_param(t))
        .collect::<Vec<Parameter>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TrivialFolder {}
    impl Folder for TrivialFolder {}

    #[test]
    fn test_fold_term() {
        let t = term!([value!(1), value!("Hi there!"), value!(true)]);
        let mut fld = TrivialFolder {};
        assert_eq!(fld.fold_term(t.clone()), t);
    }
}
