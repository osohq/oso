//! Communicate with the Polar virtual machine: load rules, make queries, etc/

use polar_core::terms::{Call, Symbol, Term, Value};

use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use crate::host::Host;
use crate::query::Query;
use crate::{ToPolar, ToPolarList};

/// Oso is the main struct you interact with. It is an instance of the Oso authorization library
/// and contains the polar language knowledge base and query engine.
#[derive(Clone)]
pub struct Oso {
    inner: Arc<polar_core::polar::Polar>,
    host: Host,
}

impl Default for Oso {
    fn default() -> Self {
        Self::new()
    }
}

impl Oso {
    /// Create a new instance of Oso. Each instance is separate and can have different rules and classes loaded into it.
    pub fn new() -> Self {
        let inner = Arc::new(polar_core::polar::Polar::new());
        let host = Host::new(inner.clone());

        let mut oso = Self { host, inner };

        for class in crate::builtins::classes() {
            oso.register_class(class)
                .expect("failed to register builtin class");
        }
        oso
    }

    /// High level interface for authorization decisions. Makes an allow query with the given actor, action and resource and returns true or false.
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
        let mut query = self.query_rule("allow", (actor, action, resource)).unwrap();
        match query.next() {
            Some(Ok(_)) => Ok(true),
            Some(Err(e)) => Err(e),
            None => Ok(false),
        }
    }

    /// Clear out all files and rules that have been loaded.
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

    /// Load a file containing polar rules. All polar files must end in `.polar`
    pub fn load_file<P: AsRef<std::path::Path>>(&mut self, file: P) -> crate::Result<()> {
        let file = file.as_ref();
        if !file.extension().map(|ext| ext == "polar").unwrap_or(false) {
            return Err(crate::OsoError::IncorrectFileType {
                filename: file.to_string_lossy().into_owned(),
            });
        }
        let mut f = File::open(&file)?;
        let mut policy = String::new();
        f.read_to_string(&mut policy)?;
        self.inner
            .load(&policy, Some(file.to_string_lossy().into_owned()))?;
        self.check_inline_queries()
    }

    /// Load a string of polar source directly.
    /// # Examples
    /// ```ignore
    /// oso.load_str("allow(a, b, c) if true;");
    /// ```
    pub fn load_str(&mut self, s: &str) -> crate::Result<()> {
        self.inner.load(s, None)?;
        self.check_inline_queries()
    }

    /// Query the knowledge base. This can be an allow query or any other polar expression.
    /// # Examples
    /// ```ignore
    /// oso.query("x = 1 or x = 2");
    /// ```
    pub fn query(&mut self, s: &str) -> crate::Result<Query> {
        let query = self.inner.new_query(s, false)?;
        check_messages!(self.inner);
        let query = Query::new(query, self.host.clone());
        Ok(query)
    }

    /// Query the knowledge base but with a rule name and argument list.
    /// This allows you to pass in rust values.
    /// # Examples
    /// ```ignore
    /// oso.query_rule("is_admin", vec![User{name: "steve"}]);
    /// ```
    pub fn query_rule(&mut self, name: &str, args: impl ToPolarList) -> crate::Result<Query> {
        let mut query_host = self.host.clone();
        let args = args.to_polar_list(&mut query_host);
        let query_value = Value::Call(Call {
            name: Symbol(name.to_string()),
            args,
            kwargs: None,
        });
        let query_term = Term::new_from_ffi(query_value);
        let query = self.inner.new_query_from_term(query_term, false);
        check_messages!(self.inner);
        let query = Query::new(query, query_host);
        Ok(query)
    }

    /// Register a rust type as a Polar class.
    /// See [`oso::Class`] docs.
    pub fn register_class(&mut self, class: crate::host::Class) -> crate::Result<()> {
        let name = class.name.clone();
        let name = Symbol(name);
        let class_name = self.host.cache_class(class.clone(), name)?;
        self.register_constant(&class_name, class)
    }

    /// Register a rust type as a Polar constant.
    /// See [`oso::Class`] docs.
    pub fn register_constant<V: crate::host::ToPolar + Send + Sync>(
        &mut self,
        name: &str,
        value: V,
    ) -> crate::Result<()> {
        self.inner
            .register_constant(Symbol(name.to_string()), value.to_polar(&mut self.host));
        Ok(())
    }
}

// @TODO: This is very unsafe.
// Temporary workaround. We need to differentiate between instances which
// _do_ need to be `Send` (e.g. registered as constants on the base `Oso` objects)
// and instances which don't need to be Send (e.g. created/accessed on a single thread for
// just one query).
unsafe impl Send for Oso {}
unsafe impl Sync for Oso {}

// Make sure the `Oso` object is threadsafe
#[cfg(test)]
static_assertions::assert_impl_all!(Oso: Send, Sync);
