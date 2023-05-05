use crate::{
    errors::OsoError,
    host::{Host, Instance, PolarIterator},
    FromPolar, PolarValue, Result,
};
use polar_core::{events::*, kb::Bindings, query::Query as PolarQuery, terms::*};
use std::collections::{BTreeMap, HashMap};
use tracing::{debug, error, trace};

impl Iterator for Query {
    type Item = Result<ResultSet>;
    fn next(&mut self) -> Option<Self::Item> {
        Query::next_result(self)
    }
}

/// Query that can be run against the rules loaded into Oso.
///
/// This is usually not used directly, but rather through [`Oso::query`](crate::Oso::query) or
/// [`Oso::query_rule`](crate::Oso::query_rule).
pub struct Query {
    inner: PolarQuery,
    /// Stores a map from call_id to the iterator the call iterates through
    iterators: HashMap<u64, PolarIterator>,
    host: Host,
}

impl Query {
    /// Create a new query.
    pub fn new(inner: PolarQuery, host: Host) -> Self {
        Self {
            iterators: HashMap::new(),
            inner,
            host,
        }
    }

    /// Source of the query.
    pub fn source(&self) -> String {
        self.inner.source_info()
    }

    /// Fetch the next result from this query.
    pub fn next_result(&mut self) -> Option<Result<ResultSet>> {
        loop {
            let event = self.inner.next()?;
            check_messages!(self.inner);
            if let Err(e) = event {
                return Some(Err(e.into()));
            }
            let event = event.unwrap();
            debug!(event=?event);
            let result = match event {
                QueryEvent::None => Ok(()),
                QueryEvent::Done { .. } => return None,
                QueryEvent::Result { bindings, .. } => {
                    return Some(ResultSet::from_bindings(bindings, self.host.clone()));
                }
                QueryEvent::MakeExternal {
                    instance_id,
                    constructor,
                } => self.handle_make_external(instance_id, constructor),
                QueryEvent::NextExternal { call_id, iterable } => {
                    self.handle_next_external(call_id, iterable)
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
                QueryEvent::ExternalIsSubclass {
                    call_id,
                    left_class_tag,
                    right_class_tag,
                } => self.handle_external_is_subclass(call_id, left_class_tag, right_class_tag),
                event => unimplemented!("Unhandled event {:?}", event),
            };

            match result {
                // Only call errors get passed back.
                Err(call_error @ OsoError::InvalidCallError { .. }) => {
                    error!("application invalid call error {}", call_error);
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

    fn question_result(&mut self, call_id: u64, result: bool) -> Result<()> {
        Ok(self.inner.question_result(call_id, result)?)
    }

    fn call_result(&mut self, call_id: u64, result: PolarValue) -> Result<()> {
        Ok(self
            .inner
            .call_result(call_id, Some(result.to_term(&mut self.host)))?)
    }

    fn call_result_none(&mut self, call_id: u64) -> Result<()> {
        Ok(self.inner.call_result(call_id, None)?)
    }

    /// Return an application error to Polar.
    ///
    /// NOTE: This should only be used for InvalidCallError.
    /// TODO (dhatch): Refactor Polar API so this is clear.
    ///
    /// All other errors must be returned directly from query.
    fn application_error(&mut self, error: OsoError) -> Result<()> {
        Ok(self.inner.application_error(error.to_string())?)
    }

    fn handle_make_external(&mut self, instance_id: u64, constructor: Term) -> Result<()> {
        match constructor.value() {
            Value::Call(Call { name, args, kwargs }) => {
                if !kwargs.is_none() {
                    lazy_error!("keyword args for constructor not supported.")
                } else {
                    let args = args
                        .iter()
                        .map(|term| PolarValue::from_term(term, &self.host))
                        .collect::<Result<Vec<PolarValue>>>()?;
                    self.host.make_instance(&name.0, args, instance_id)
                }
            }
            _ => lazy_error!("invalid type for constructing an instance -- internal error"),
        }
    }

    fn next_call_result(&mut self, call_id: u64) -> Option<Result<PolarValue>> {
        self.iterators.get_mut(&call_id).and_then(|c| c.next())
    }

    fn handle_next_external(&mut self, call_id: u64, iterable: Term) -> Result<()> {
        if self.iterators.get(&call_id).is_none() {
            let iterable_instance =
                Instance::from_polar(PolarValue::from_term(&iterable, &self.host)?)?;
            let iter = iterable_instance.as_iter(&self.host)?;
            self.iterators.insert(call_id, iter);
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
    ) -> Result<()> {
        if kwargs.is_some() {
            return lazy_error!("Invalid call error: kwargs not supported in Rust.");
        }
        trace!(call_id, name = %name, args = ?args, "call");
        let instance = Instance::from_polar(PolarValue::from_term(&instance, &self.host)?)?;
        let result = if let Some(args) = args {
            let args = args
                .iter()
                .map(|v| PolarValue::from_term(v, &self.host))
                .collect::<Result<Vec<PolarValue>>>()?;
            instance.call(&name.0, args, &mut self.host)
        } else {
            instance.get_attr(&name.0, &mut self.host)
        };
        match result {
            Ok(t) => self.call_result(call_id, t),
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
    ) -> Result<()> {
        assert_eq!(args.len(), 2);
        let res = {
            let args = [
                Instance::from_polar(PolarValue::from_term(&args[0], &self.host)?)?,
                Instance::from_polar(PolarValue::from_term(&args[1], &self.host)?)?,
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
    ) -> Result<()> {
        debug!(instance = ?instance, class = %class_tag, "isa");
        let res = self
            .host
            .isa(PolarValue::from_term(&instance, &self.host)?, &class_tag.0)?;
        self.question_result(call_id, res)?;
        Ok(())
    }

    fn handle_external_is_subspecializer(
        &mut self,
        call_id: u64,
        instance_id: u64,
        left_class_tag: Symbol,
        right_class_tag: Symbol,
    ) -> Result<()> {
        let res = self
            .host
            .is_subspecializer(instance_id, &left_class_tag.0, &right_class_tag.0);
        self.question_result(call_id, res)?;
        Ok(())
    }

    fn handle_external_is_subclass(
        &mut self,
        call_id: u64,
        left_class_tag: Symbol,
        right_class_tag: Symbol,
    ) -> Result<()> {
        let res = left_class_tag == right_class_tag;
        self.question_result(call_id, res)?;
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn handle_debug(&mut self, message: String) -> Result<()> {
        eprintln!("TODO: {}", message);
        check_messages!(self.inner);
        Ok(())
    }
}

/// Set of results from Oso query.
#[derive(Clone)]
pub struct ResultSet {
    bindings: Bindings,
    host: Host,
}

impl ResultSet {
    /// Create new ResultSet from bindings.
    pub fn from_bindings(bindings: Bindings, host: Host) -> Result<Self> {
        // Check for expression.
        for term in bindings.values() {
            if term.as_expression().is_ok() && !host.accept_expression {
                return Err(OsoError::Custom {
                    message: r#"
Received Expression from Polar VM. The Expression type is not yet supported in this language.

This may mean you performed an operation in your policy over an unbound variable.
                    "#
                    .to_owned(),
                });
            }
        }

        Ok(Self { bindings, host })
    }

    /// Return the keys in bindings.
    pub fn keys(&self) -> Box<dyn std::iter::Iterator<Item = &str> + '_> {
        Box::new(self.bindings.keys().map(|sym| sym.0.as_ref()))
    }

    /// Iterator over the bindings of this result.
    pub fn iter_bindings(&self) -> Box<dyn std::iter::Iterator<Item = (&str, &Value)> + '_> {
        Box::new(self.bindings.iter().map(|(k, v)| (k.0.as_ref(), v.value())))
    }

    /// Check if the bindings in this result are empty.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Get a value from the bindings.
    pub fn get(&self, name: &str) -> Option<crate::PolarValue> {
        self.bindings
            .get(&Symbol(name.to_string()))
            .map(|t| PolarValue::from_term(t, &self.host).unwrap())
    }

    /// Get a value from the bindings, and decode them into a Rust type.
    pub fn get_typed<T: FromPolar>(&self, name: &str) -> Result<T> {
        self.get(name)
            .ok_or(OsoError::FromPolar)
            .and_then(T::from_polar)
    }

    /// Turn self into an event.
    pub fn into_event(self) -> ResultEvent {
        ResultEvent::new(self.bindings)
    }
}

impl std::fmt::Debug for ResultSet {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:#?}", self.bindings)
    }
}

impl<S: AsRef<str>, T: FromPolar + PartialEq<T>> PartialEq<HashMap<S, T>> for ResultSet {
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
