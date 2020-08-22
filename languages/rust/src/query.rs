use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use polar_core::types::Symbol as Name;
use polar_core::types::*;

pub struct Query {
    calls: HashMap<u64, Box<dyn Iterator<Item = Arc<dyn crate::host::ToPolar>>>>,
    inner: polar_core::polar::Query,
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

    pub fn next(&mut self) -> Option<anyhow::Result<ResultSet>> {
        loop {
            let event = self.inner.next()?;
            if let Err(e) = event {
                return Some(Err(e.into()));
            }
            let result = match event.unwrap() {
                QueryEvent::None => Ok(()),
                QueryEvent::Done => return None,
                QueryEvent::Result { bindings, trace } => {
                    return Some(Ok(ResultSet {
                        bindings,
                        host: self.host.clone(),
                    }))
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
                return Some(Err(e.into()));
            }
        }
    }

    fn handle_make_external(&mut self, instance_id: u64, constructor: Term) -> anyhow::Result<()> {
        let mut host = self.host.lock().unwrap();
        match constructor.value() {
            Value::InstanceLiteral(InstanceLiteral { tag, fields }) => {
                todo!("instantiate from literal")
            }
            Value::Call(Predicate { name, args }) => {
                let _instance = host.make_instance(name, args.clone(), instance_id);
            }
            _ => panic!("not valid"),
        }
        Ok(())
    }

    fn handle_external_call(
        &mut self,
        call_id: u64,
        instance: Term,
        name: Name,
        args: Option<Vec<Term>>,
    ) -> anyhow::Result<()> {
        if self.calls.get(&call_id).is_none() {
            let instance = match instance.value() {
                Value::ExternalInstance(ExternalInstance { instance_id, .. }) => self
                    .host
                    .lock()
                    .unwrap()
                    .get_instance(*instance_id)
                    .expect("instance not found")
                    .clone(),
                _ => {
                    self.inner.call_result(call_id, None)?;
                    return Ok(());
                }
            };
            if let Some(args) = args {
                if let Some(m) = instance.methods.get(&name) {
                    // TODO: Make this handle multiple results with iters?
                    let result = m.invoke(
                        instance.instance.as_ref(),
                        args,
                        &mut self.host.lock().unwrap(),
                    );
                    self.calls
                        .insert(call_id, Box::new(std::iter::once(result)));
                }
            } else {
                if let Some(attr) = instance.attributes.get(&name) {
                    // TODO: Make this handle multiple results with iters?
                    let result = attr(&instance, vec![]);
                    self.calls
                        .insert(call_id, Box::new(std::iter::once(result)));
                }
            }
        }

        if let Some(result) = self.calls.get_mut(&call_id).and_then(|c| c.next()) {
            self.inner.call_result(
                call_id,
                Some(result.to_polar(&mut self.host.lock().unwrap())),
            )?;
        } else {
            self.inner.call_result(call_id, None)?;
        }

        Ok(())
    }

    fn handle_external_op(
        &mut self,
        call_id: u64,
        operator: Operator,
        args: Vec<Term>,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn handle_external_isa(
        &mut self,
        call_id: u64,
        instance: Term,
        class_tag: Name,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn handle_external_unify(
        &mut self,
        call_id: u64,
        left_instance_id: u64,
        right_instance_id: u64,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn handle_external_is_subspecializer(
        &mut self,
        call_id: u64,
        instance_id: u64,
        left_class_tag: Name,
        right_class_tag: Name,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn handle_debug(&mut self, message: String) -> anyhow::Result<()> {
        todo!()
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
