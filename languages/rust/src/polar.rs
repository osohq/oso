/// Communicate with the Polar virtual machine: load rules, make queries, etc/
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::host::Host;

pub struct Polar {
    inner: Rc<crate::PolarCore>,
    host: Arc<Mutex<Host>>,
}

impl Polar {
    pub fn new() -> Self {
        let inner = Rc::new(crate::PolarCore::new());
        let mut host = Host::new(Rc::downgrade(&inner));

        // register all builtin constants
        for (name, class) in crate::host::builtins::constants() {
            let value = host.cache_class(class, name.clone());
            inner.register_constant(name, value);
        }

        Self {
            host: Arc::new(Mutex::new(host)),
            inner,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    fn check_messages(&mut self) {
        while let Some(msg) = self.inner.next_message() {
            match msg.kind {
                polar_core::types::MessageKind::Print => tracing::trace!("{}", msg.msg),
                polar_core::types::MessageKind::Warning => tracing::warn!("{}", msg.msg),
            }
        }
    }

    fn check_inline_queries(&mut self) -> anyhow::Result<()> {
        while let Some(q) = self.inner.next_inline_query(false) {
            let query = crate::query::Query::new(q, self.host.clone());
            match query.collect::<anyhow::Result<Vec<_>>>() {
                Ok(v) if !v.is_empty() => continue,
                Ok(_) => anyhow::bail!("inline query result was false"),
                Err(e) => {
                    anyhow::bail!("error in inline query: {}", e);
                }
            }
        }
        self.check_messages();
        Ok(())
    }

    pub fn load_file(&mut self, file: &str) -> anyhow::Result<()> {
        let mut f = File::open(&file)?;
        let mut policy = String::new();
        f.read_to_string(&mut policy)?;
        self.inner.load(&policy, Some(file.to_string()))?;
        self.check_inline_queries()
    }

    pub fn load_str(&mut self, s: &str) -> anyhow::Result<()> {
        self.inner.load(s, None)?;
        self.check_inline_queries()
    }

    pub fn query(&mut self, s: &str) -> anyhow::Result<crate::query::Query> {
        let query = self.inner.new_query(s, false)?;
        self.check_messages();
        let query = crate::query::Query::new(query, self.host.clone());
        Ok(query)
    }

    pub fn query_rule<'a>(
        &mut self,
        name: &str,
        args: impl IntoIterator<Item = &'a dyn crate::host::ToPolar>,
    ) -> anyhow::Result<crate::query::Query> {
        let args = args
            .into_iter()
            .map(|arg| arg.to_polar(&mut self.host.lock().unwrap()))
            .collect();
        let query_value = polar_core::types::Value::Call(polar_core::types::Predicate {
            name: polar_core::types::Symbol(name.to_string()),
            args,
        });
        let query_term = polar_core::types::Term::new_from_ffi(query_value);
        let query = self.inner.new_query_from_term(query_term, false);
        self.check_messages();
        let query = crate::query::Query::new(query, self.host.clone());
        Ok(query)
    }

    pub fn register_class(&mut self, class: crate::host::Class) -> anyhow::Result<()> {
        let mut host = self.host.lock().unwrap();
        let name = class.name.clone();
        let name = polar_core::types::Symbol(name);
        let class_term = host.cache_class(class, name.clone());
        self.inner.register_constant(name, class_term);
        Ok(())
    }

    pub fn register_constant<V: crate::host::ToPolar>(
        &mut self,
        name: &str,
        value: &V,
    ) -> anyhow::Result<()> {
        let mut host = self.host.lock().unwrap();
        self.inner.register_constant(
            polar_core::types::Symbol(name.to_string()),
            host.to_polar(value),
        );
        Ok(())
    }
}
