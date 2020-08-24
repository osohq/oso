use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::host::{FromPolar, Instance, ToPolar};
use polar_core::types::Symbol as Name;
use polar_core::types::*;

impl Iterator for Query {
    type Item = anyhow::Result<ResultSet>;
    fn next(&mut self) -> Option<Self::Item> {
        Query::next_result(self)
    }
}

pub struct Query {
    inner: polar_core::polar::Query,
    calls: HashMap<u64, Box<dyn Iterator<Item = Arc<dyn crate::host::ToPolar>>>>,
    host: Arc<Mutex<crate::host::Host>>,
}

impl Query {
    pub fn new(inner: polar_core::polar::Query, host: Arc<Mutex<crate::host::Host>>) -> Self {
        Self {
            calls: HashMap::new(),
            inner,
            host,
        }
    }

    pub fn next_result(&mut self) -> Option<anyhow::Result<ResultSet>> {
        loop {
            let event = self.inner.next()?;
            self.check_messages();
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
                } => self.handle_external_call(call_id, instance, attribute, args),
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
                self.application_error(e);
            }
        }
    }

    fn check_messages(&mut self) {
        while let Some(msg) = self.inner.next_message() {
            match msg.kind {
                polar_core::types::MessageKind::Print => tracing::trace!("{}", msg.msg),
                polar_core::types::MessageKind::Warning => tracing::warn!("{}", msg.msg),
            }
        }
    }

    fn question_result(&mut self, call_id: u64, result: bool) {
        self.inner.question_result(call_id, result);
    }

    fn call_result(&mut self, call_id: u64, result: Arc<dyn ToPolar>) -> anyhow::Result<()> {
        let mut host = self.host.lock().unwrap();
        let value = host.value_to_polar(result.as_ref());
        Ok(self.inner.call_result(call_id, Some(value))?)
    }

    fn call_result_none(&mut self, call_id: u64) -> anyhow::Result<()> {
        Ok(self.inner.call_result(call_id, None)?)
    }

    fn application_error(&mut self, error: anyhow::Error) {
        self.inner.application_error(error.to_string())
    }

    fn handle_make_external(&mut self, instance_id: u64, constructor: Term) -> anyhow::Result<()> {
        let mut host = self.host.lock().unwrap();
        match constructor.value() {
            Value::InstanceLiteral(InstanceLiteral { .. }) => todo!("instantiate from literal"),
            Value::Call(Predicate { name, args }) => {
                let _instance = host.make_instance(name, args.clone(), instance_id);
            }
            _ => panic!("not valid"),
        }
        Ok(())
    }

    fn register_call(
        &mut self,
        call_id: u64,
        instance: Instance,
        name: Name,
        args: Option<Vec<Term>>,
    ) -> anyhow::Result<()> {
        if self.calls.get(&call_id).is_none() {
            let (f, args) = if let Some(args) = args {
                if let Some(m) = instance.methods.get(&name) {
                    (m, args)
                } else {
                    return Err(anyhow::anyhow!("instance method not found"));
                }
            } else if let Some(attr) = instance.attributes.get(&name) {
                (attr, vec![])
            } else {
                return Err(anyhow::anyhow!("attribute lookup not found"));
            };
            tracing::trace!(call_id, name = %name, args = ?args, "register_call");
            let host = &mut self.host.lock().unwrap();
            let result = f.invoke(instance.instance.as_ref(), args, host);
            self.calls
                .insert(call_id, Box::new(std::iter::once(result)));
        }
        Ok(())
    }

    fn next_call_result(&mut self, call_id: u64) -> Option<Arc<dyn ToPolar>> {
        self.calls.get_mut(&call_id).and_then(|c| c.next())
    }

    fn handle_external_call(
        &mut self,
        call_id: u64,
        instance: Term,
        name: Name,
        args: Option<Vec<Term>>,
    ) -> anyhow::Result<()> {
        let instance = Instance::from_polar(&instance, &mut self.host.lock().unwrap()).unwrap();
        self.register_call(call_id, instance, name, args)?;

        if let Some(result) = self.next_call_result(call_id) {
            self.call_result(call_id, result)?;
        } else {
            self.call_result_none(call_id)?;
        }

        Ok(())
    }

    fn handle_external_op(
        &mut self,
        call_id: u64,
        operator: Operator,
        args: Vec<Term>,
    ) -> anyhow::Result<()> {
        assert_eq!(args.len(), 2);
        let res = {
            let mut host = self.host.lock().unwrap();
            let args = [
                Instance::from_polar(&args[0], &mut host).unwrap(),
                Instance::from_polar(&args[1], &mut host).unwrap(),
            ];
            host.operator(operator, args)
        };
        self.question_result(call_id, res);
        Ok(())
    }

    fn handle_external_isa(
        &mut self,
        call_id: u64,
        instance: Term,
        class_tag: Name,
    ) -> anyhow::Result<()> {
        tracing::debug!(instance = ?instance, class = %class_tag, "isa");
        let res = self.host.lock().unwrap().isa(instance, &class_tag);
        self.question_result(call_id, res);
        Ok(())
    }

    fn handle_external_unify(
        &mut self,
        call_id: u64,
        left_instance_id: u64,
        right_instance_id: u64,
    ) -> anyhow::Result<()> {
        let res = self
            .host
            .lock()
            .unwrap()
            .unify(left_instance_id, right_instance_id);
        self.question_result(call_id, res);
        Ok(())
    }

    fn handle_external_is_subspecializer(
        &mut self,
        call_id: u64,
        instance_id: u64,
        left_class_tag: Name,
        right_class_tag: Name,
    ) -> anyhow::Result<()> {
        let res = self.host.lock().unwrap().is_subspecializer(
            instance_id,
            &left_class_tag,
            &right_class_tag,
        );
        self.question_result(call_id, res);
        Ok(())
    }

    fn handle_debug(&mut self, message: String) -> anyhow::Result<()> {
        eprintln!("TODO: {}", message);
        self.check_messages();
        Ok(())
    }
}

#[derive(Clone)]
pub struct ResultSet {
    bindings: polar_core::types::Bindings,
    host: Arc<Mutex<crate::host::Host>>,
}

impl ResultSet {
    pub fn get<T: crate::host::FromPolar>(&self, name: &str) -> Option<T> {
        self.bindings
            .get(&Name(name.to_string()))
            .and_then(|term| T::from_polar(term, &mut self.host.lock().unwrap()))
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
            self.get::<T>(k.as_ref())
                .map(|binding| &binding == v)
                .unwrap_or(false)
        })
    }
}
