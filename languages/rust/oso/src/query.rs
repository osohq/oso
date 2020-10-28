use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::errors::OsoError;
use crate::host::{Host, Instance, PolarIterator};
use crate::{FromPolar, PolarValue};

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
    calls: HashMap<u64, PolarIterator>,
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
                QueryEvent::Done { .. } => return None,
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
                QueryEvent::NextExternal { call_id, term } => {
                    self.handle_next_external(call_id, term)
                }
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
                event => unimplemented!("Unhandled event {:?}", event),
            };

            match result {
                // Only call errors get passed back.
                Err(call_error @ OsoError::InvalidCallError { .. }) => {
                    tracing::error!("application invalid call error {}", call_error);
                    if let Err(e) = self.application_error(call_error) {
                        return Some(Err(e));
                    }
                }
                // All others get returned.
                Err(err) => return Some(Err(err)),
                // Continue on ok
                Ok(_) => {}
            }
        }
    }

    fn question_result(&mut self, call_id: u64, result: bool) -> crate::Result<()> {
        Ok(self.inner.question_result(call_id, result)?)
    }

    fn call_result(&mut self, call_id: u64, result: Term) -> crate::Result<()> {
        Ok(self.inner.call_result(call_id, Some(result))?)
    }

    fn call_result_none(&mut self, call_id: u64) -> crate::Result<()> {
        Ok(self.inner.call_result(call_id, None)?)
    }

    /// Return an application error to Polar.
    ///
    /// NOTE: This should only be used for InvalidCallError.
    /// TODO (dhatch): Refactor Polar API so this is clear.
    ///
    /// All other errors must be returned directly from query.
    fn application_error(&mut self, error: crate::OsoError) -> crate::Result<()> {
        Ok(self.inner.application_error(error.to_string())?)
    }

    fn handle_make_external(&mut self, instance_id: u64, constructor: Term) -> crate::Result<()> {
        match constructor.value() {
            Value::Call(Call { name, args, kwargs }) => {
                if !kwargs.is_none() {
                    lazy_error!("keyword args for constructor not supported.")
                } else {
                    self.host.make_instance(name, args.clone(), instance_id)
                }
            }
            _ => lazy_error!("invalid type for constructing an instance -- internal error"),
        }
    }

    fn next_call_result(&mut self, call_id: u64) -> Option<crate::Result<Term>> {
        let Self {
            ref mut calls,
            ref mut host,
            ..
        } = self;
        calls.get_mut(&call_id).and_then(|c| c.next(host))
    }

    fn handle_next_external(&mut self, call_id: u64, term: Term) -> crate::Result<()> {
        if self.calls.get(&call_id).is_none() {
            let instance = Instance::from_polar(&term, &self.host)?;
            let iter = instance
                .downcast::<crate::host::PolarIterator>(Some(&self.host))
                .map(|i| Ok(i.clone()))
                .unwrap_or_else(|_| instance.as_iter(&self.host))?;
            self.calls.insert(call_id, iter);
        }

        match self.next_call_result(call_id) {
            Some(Ok(result)) => self.call_result(call_id, result),
            Some(Err(e)) => {
                self.call_result_none(call_id)?;
                Err(e)
            }
            None => self.call_result_none(call_id),
        }
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
        tracing::trace!(call_id, name = %name, args = ?args, "call");
        let instance = Instance::from_polar(&instance, &self.host).unwrap();
        let result = if let Some(args) = args {
            instance.call(&name.0, args, &mut self.host)
        } else {
            instance.get_attr(&name.0, &mut self.host)
        };
        match result {
            Ok(t) => {
                self.call_result(call_id, t)?;
                Ok(())
            }
            Err(e) => {
                self.call_result_none(call_id)?;
                Err(e)
            }
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
        self.question_result(call_id, res)?;
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
        self.question_result(call_id, res)?;
        Ok(())
    }

    fn handle_external_unify(
        &mut self,
        call_id: u64,
        left_instance_id: u64,
        right_instance_id: u64,
    ) -> crate::Result<()> {
        let res = self.host.unify(left_instance_id, right_instance_id)?;
        self.question_result(call_id, res)?;
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
        self.question_result(call_id, res)?;
        Ok(())
    }

    fn handle_debug(&mut self, message: String) -> crate::Result<()> {
        eprintln!("TODO: {}", message);
        check_messages!(self.inner);
        Ok(())
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

    pub fn get(&self, name: &str) -> Option<crate::PolarValue> {
        self.bindings
            .get(&Symbol(name.to_string()))
            .map(|t| PolarValue::from_term(t, &self.host).unwrap())
    }

    pub fn get_typed<T: crate::host::FromPolarValue>(&self, name: &str) -> crate::Result<T> {
        self.get(name)
            .ok_or_else(|| crate::OsoError::FromPolar)
            .and_then(T::from_polar_value)
    }
}

impl std::fmt::Debug for ResultSet {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:#?}", self.bindings)
    }
}

impl<S: AsRef<str>, T: crate::host::FromPolarValue + PartialEq<T>> PartialEq<HashMap<S, T>>
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
