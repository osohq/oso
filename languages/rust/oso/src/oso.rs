//! Communicate with the Polar virtual machine: load rules, make queries, etc/

use polar_core::terms::{Call, Symbol, Term, Value};

use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};

use crate::host::Host;
use crate::query::Query;
use crate::ToPolar;

#[derive(Clone)]
pub struct Oso {
    inner: Arc<polar_core::polar::Polar>,
    host: Arc<Mutex<Host>>,
}

impl Default for Oso {
    fn default() -> Self {
        Self::new()
    }
}

impl Oso {
    pub fn new() -> Self {
        let inner = Arc::new(polar_core::polar::Polar::new());
        let host = Host::new(inner.clone());

        let mut oso = Self {
            host: Arc::new(Mutex::new(host)),
            inner,
        };

        for class in crate::builtins::classes() {
            oso.register_class(class)
                .expect("failed to register builtin class");
        }
        oso
    }

    pub fn is_allowed<Actor, Action, Resource>(
        &mut self,
        actor: Actor,
        action: Action,
        resource: Resource,
    ) -> crate::Result<bool>
    where
        Actor: ToPolar,
        Action: ToPolar,
        Resource: ToPolar,
    {
        let args: Vec<&dyn ToPolar> = vec![&actor, &action, &resource];
        let mut query = self.query_rule("allow", args).unwrap();
        match query.next() {
            Some(Ok(_)) => Ok(true),
            Some(Err(e)) => Err(e),
            None => Ok(false),
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    fn check_inline_queries(&mut self) -> crate::Result<()> {
        while let Some(q) = self.inner.next_inline_query(false) {
            let query = Query::new(q, self.host.clone());
            match query.collect::<crate::Result<Vec<_>>>() {
                Ok(v) if !v.is_empty() => continue,
                Ok(_) => return lazy_error!("inline query result was false"),
                Err(e) => return lazy_error!("error in inline query: {}", e),
            }
        }
        check_messages!(self.inner);
        Ok(())
    }

    pub fn load_file<P: AsRef<std::path::Path>>(&mut self, file: P) -> crate::Result<()> {
        let file = file.as_ref();
        if !file.extension().map(|ext| ext == "polar").unwrap_or(false) {
            return Err(crate::OsoError::IncorrectFileType { filename: file.to_string_lossy().into_owned() });
        }
        let mut f = File::open(&file)?;
        let mut policy = String::new();
        f.read_to_string(&mut policy)?;
        self.inner.load(&policy, Some(file.to_string_lossy().into_owned()))?;
        self.check_inline_queries()
    }

    pub fn load_str(&mut self, s: &str) -> crate::Result<()> {
        self.inner.load(s, None)?;
        self.check_inline_queries()
    }

    pub fn query(&mut self, s: &str) -> crate::Result<Query> {
        let query = self.inner.new_query(s, false)?;
        check_messages!(self.inner);
        let query = Query::new(query, self.host.clone());
        Ok(query)
    }

    pub fn query_rule<'a>(
        &mut self,
        name: &str,
        args: impl IntoIterator<Item = &'a dyn crate::host::ToPolar>,
    ) -> crate::Result<Query> {
        let args = args
            .into_iter()
            .map(|arg| arg.to_polar(&mut self.host.lock().unwrap()))
            .collect();
        let query_value = Value::Call(Call {
            name: Symbol(name.to_string()),
            args,
            kwargs: None,
        });
        let query_term = Term::new_from_ffi(query_value);
        let query = self.inner.new_query_from_term(query_term, false);
        check_messages!(self.inner);
        let query = Query::new(query, self.host.clone());
        Ok(query)
    }

    pub fn register_class(&mut self, class: crate::host::Class) -> crate::Result<()> {
        let name = class.name.clone();
        let name = Symbol(name);
        let class_name = self.host.lock().unwrap().cache_class(class.clone(), name);
        self.register_constant(&class_name, &class)
    }

    pub fn register_constant<V: crate::host::ToPolar>(
        &mut self,
        name: &str,
        value: &V,
    ) -> crate::Result<()> {
        let mut host = self.host.lock().unwrap();
        self.inner
            .register_constant(Symbol(name.to_string()), value.to_polar(&mut host));
        Ok(())
    }
}
