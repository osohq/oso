use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::host::{Host, Instance, PolarResultIter};
use crate::FromPolar;

use polar_core::events::*;
use polar_core::terms::*;

impl Iterator for Query {
    type Item = crate::Result<ResultSet>;
    fn next(&mut self) -> Option<Self::Item> {
        Query::next_result(self)
    }
}

pub struct Query {
    inner: polar_core::polar::Query,
    calls: HashMap<u64, PolarResultIter>,
    host: Host,
}

impl Query {
    pub fn new(inner: polar_core::polar::Query, host: Host) -> Self {
        Self {
            calls: HashMap::new(),
            inner,
            host,
        }
    }

    pub fn next_result(&mut self) -> Option<crate::Result<ResultSet>> {
        loop {
            let event = self.inner.next()?;
            check_messages!(self.inner);
            if let Err(e) = event {
                return Some(Err(e.into()));
            }
            let event = event.unwrap();
            tracing::debug!(event=?event);
            let result = match event {
                QueryEvent::None => Ok(()),
                QueryEvent::Done => return None,
                QueryEvent::Result { bindings, .. } => {
                    return Some(Ok(ResultSet {
                        bindings,
                        host: self.host.clone(),
                    }));
                }
                QueryEvent::MakeExternal {
                    instance_id,
                    constructor,
                } => self.handle_make_external(instance_id, constructor),
                QueryEvent::ExternalCall {
                    call_id,
                    instance,
                    attribute,
                    args,
                    kwargs,
                } => self.handle_external_call(call_id, instance, attribute, args, kwargs),
                QueryEvent::ExternalOp {
                    call_id,
                    operator,
                    args,
                } => self.handle_external_op(call_id, operator, args),
                QueryEvent::ExternalIsa {
                    call_id,
                    instance,
                    class_tag,
                } => self.handle_external_isa(call_id, instance, class_tag),
                QueryEvent::ExternalUnify {
                    call_id,
                    left_instance_id,
                    right_instance_id,
                } => self.handle_external_unify(call_id, left_instance_id, right_instance_id),
                QueryEvent::ExternalIsSubSpecializer {
                    call_id,
                    instance_id,
                    left_class_tag,
                    right_class_tag,
                } => self.handle_external_is_subspecializer(
                    call_id,
                    instance_id,
                    left_class_tag,
                    right_class_tag,
                ),
                QueryEvent::Debug { message } => self.handle_debug(message),
            };
            if let Err(e) = result {
                // TODO (dhatch): These seem to be getting swallowed
                tracing::error!("application error {}", e);
                self.application_error(e);
            }
        }
    }

    fn question_result(&mut self, call_id: u64, result: bool) {
        self.inner.question_result(call_id, result);
    }

    fn call_result(&mut self, call_id: u64, result: Term) -> crate::Result<()> {
        Ok(self.inner.call_result(call_id, Some(result))?)
    }

    fn call_result_none(&mut self, call_id: u64) -> crate::Result<()> {
        Ok(self.inner.call_result(call_id, None)?)
    }

    fn application_error(&mut self, error: crate::OsoError) {
        self.inner.application_error(error.to_string())
    }

    fn handle_make_external(&mut self, instance_id: u64, constructor: Term) -> crate::Result<()> {
        match constructor.value() {
            Value::Call(Call { name, args, .. }) => {
                self.host.make_instance(name, args.clone(), instance_id)
            }
            _ => lazy_error!("invalid type for constructing an instance -- internal error"),
        }
    }

    fn register_call(
        &mut self,
        call_id: u64,
        instance: Instance,
        name: Symbol,
        args: Option<Vec<Term>>,
    ) -> crate::Result<()> {
        if self.calls.get(&call_id).is_none() {
            tracing::trace!(call_id, name = %name, args = ?args, "register_call");
            let results = if let Some(args) = args {
                instance.call(&name.0, args, &mut self.host)?
            } else {
                Box::new(std::iter::once(instance.get_attr(&name.0, &mut self.host)))
            };
            self.calls.insert(call_id, results);
        }
        Ok(())
    }

    fn next_call_result(&mut self, call_id: u64) -> Option<Result<Term, crate::OsoError>> {
        self.calls.get_mut(&call_id).and_then(|c| c.next())
    }

    fn handle_external_call(
        &mut self,
        call_id: u64,
        instance: Term,
        name: Symbol,
        args: Option<Vec<Term>>,
        kwargs: Option<BTreeMap<Symbol, Term>>,
    ) -> crate::Result<()> {
        if kwargs.is_some() {
            return lazy_error!("Invalid call error: kwargs not supported in Rust.");
        }
        let instance = Instance::from_polar(&instance, &self.host).unwrap();
        if let Err(e) = self.register_call(call_id, instance, name, args) {
            self.application_error(e);
            return self.call_result_none(call_id);
        }

        if let Some(result) = self.next_call_result(call_id) {
            match result {
                Ok(r) => self.call_result(call_id, r),
                Err(e) => {
                    self.application_error(e);
                    self.call_result_none(call_id)
                }
            }
        } else {
            self.call_result_none(call_id)
        }
    }

    fn handle_external_op(
        &mut self,
        call_id: u64,
        operator: Operator,
        args: Vec<Term>,
    ) -> crate::Result<()> {
        assert_eq!(args.len(), 2);
        let res = {
            let args = [
                Instance::from_polar(&args[0], &self.host).unwrap(),
                Instance::from_polar(&args[1], &self.host).unwrap(),
            ];
            self.host.operator(operator, args)?
        };
        self.question_result(call_id, res);
        Ok(())
    }

    fn handle_external_isa(
        &mut self,
        call_id: u64,
        instance: Term,
        class_tag: Symbol,
    ) -> crate::Result<()> {
        tracing::debug!(instance = ?instance, class = %class_tag, "isa");
        let res = self.host.isa(instance, &class_tag)?;
        self.question_result(call_id, res);
        Ok(())
    }

    fn handle_external_unify(
        &mut self,
        call_id: u64,
        left_instance_id: u64,
        right_instance_id: u64,
    ) -> crate::Result<()> {
        let res = self.host.unify(left_instance_id, right_instance_id)?;
        self.question_result(call_id, res);
        Ok(())
    }

    fn handle_external_is_subspecializer(
        &mut self,
        call_id: u64,
        instance_id: u64,
        left_class_tag: Symbol,
        right_class_tag: Symbol,
    ) -> crate::Result<()> {
        let res = self
            .host
            .is_subspecializer(instance_id, &left_class_tag, &right_class_tag);
        self.question_result(call_id, res);
        Ok(())
    }

    fn handle_debug(&mut self, message: String) -> crate::Result<()> {
        eprintln!("TODO: {}", message);
        check_messages!(self.inner);
        Ok(())
    }

    /// Covert `term` into type `T`.
    pub fn from_polar<T: FromPolar>(&self, term: &Term) -> crate::Result<T> {
        Ok(T::from_polar(term, &self.host)?)
    }

    // TODO (dhatch): Get rid of this when implementing value type for the library.
    /// Convert `value` into type `T`.
    pub fn from_polar_value<T: FromPolar>(&self, value: Value) -> crate::Result<T> {
        Ok(T::from_polar(&Term::new_temporary(value), &self.host)?)
    }
}

#[derive(Clone)]
pub struct ResultSet {
    bindings: polar_core::kb::Bindings,
    host: crate::host::Host,
}

impl ResultSet {
    /// Return the keys in bindings.
    pub fn keys(&self) -> Box<dyn std::iter::Iterator<Item = &str> + '_> {
        Box::new(self.bindings.keys().map(|sym| sym.0.as_ref()))
    }

    pub fn iter_bindings(&self) -> Box<dyn std::iter::Iterator<Item = (&str, &Value)> + '_> {
        Box::new(self.bindings.iter().map(|(k, v)| (k.0.as_ref(), v.value())))
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<crate::Value> {
        self.bindings
            .get(&Symbol(name.to_string()))
            .map(|t| t.value().clone())
    }

    pub fn get_typed<T: crate::host::FromPolar>(&self, name: &str) -> crate::Result<T> {
        // TODO (dhatch): Type error
        self.bindings
            .get(&Symbol(name.to_string()))
            .ok_or_else(|| crate::OsoError::FromPolar)
            .and_then(|term| T::from_polar(term, &self.host))
    }
}

impl std::fmt::Debug for ResultSet {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:#?}", self.bindings)
    }
}

impl<S: AsRef<str>, T: crate::host::FromPolar + PartialEq<T>> PartialEq<HashMap<S, T>>
    for ResultSet
{
    fn eq(&self, other: &HashMap<S, T>) -> bool {
        other.iter().all(|(k, v)| {
            self.get_typed::<T>(k.as_ref())
                .map(|binding| &binding == v)
                .unwrap_or(false)
        })
    }
}

// Make sure the `Query` object is _not_ threadsafe
#[cfg(test)]
static_assertions::assert_not_impl_any!(Query: Send, Sync);
