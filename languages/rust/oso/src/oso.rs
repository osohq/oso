//! Communicate with the Polar virtual machine: load rules, make queries, etc/
use polar_core::sources::Source;
use polar_core::terms::{Call, Symbol, Term, Value};

use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::sync::Arc;

use crate::host::Host;
use crate::query::Query;
use crate::{FromPolar, OsoError, PolarValue, ToPolar, ToPolarList};

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

/// Represents an `action` used in an `allow` rule.
/// When the action is bound to a concrete value (e.g. a string)
/// this returns an `Action::Typed(action)`.
/// If _any_ actions are allowed, then the `Action::Any` variant is returned.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Action<T = String> {
    Any,
    Typed(T),
}

impl<T: FromPolar> FromPolar for Action<T> {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if matches!(val, PolarValue::Variable(_)) {
            Ok(Action::Any)
        } else {
            T::from_polar(val).map(Action::Typed)
        }
    }
}

impl Oso {
    /// Create a new instance of Oso. Each instance is separate and can have different rules and classes loaded into it.
    pub fn new() -> Self {
        let inner = Arc::new(polar_core::polar::Polar::new());
        let host = Host::new(inner.clone());

        let mut oso = Self { inner, host };

        for class in crate::builtins::classes() {
            oso.register_class(class)
                .expect("failed to register builtin class");
        }
        oso.register_constant(Option::<crate::PolarValue>::None, "nil")
            .expect("failed to register the constant None");
        oso
    }

    /// High level interface for authorization decisions. Makes an allow query with the given actor, action and resource and returns true or false.
    pub fn is_allowed<Actor, Action, Resource>(
        &self,
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

    /// Get the actions actor is allowed to take on resource.
    /// Returns a [std::collections::HashSet] of actions, typed according the return value.
    /// # Examples
    /// ```ignore
    /// oso.load_str(r#"allow(actor: Actor{name: "sally"}, action, resource: Widget{id: 1}) if
    ///               action in ["CREATE", "READ"];"#);
    ///
    /// // get a HashSet of oso::Actions
    /// let actions: HashSet<Action> = oso.get_allowed_actions(actor, resource)?;
    ///
    /// // or Strings
    /// let actions: HashSet<String> = oso.get_allowed_actions(actor, resource)?;
    /// ```
    pub fn get_allowed_actions<Actor, Resource, T>(
        &self,
        actor: Actor,
        resource: Resource,
    ) -> crate::Result<HashSet<T>>
    where
        Actor: ToPolar,
        Resource: ToPolar,
        T: FromPolar + Eq + Hash,
    {
        let mut query = self
            .query_rule(
                "allow",
                (actor, PolarValue::Variable("action".to_owned()), resource),
            )
            .unwrap();

        let mut set = HashSet::new();
        loop {
            match query.next() {
                Some(Ok(result)) => {
                    if let Some(action) = result.get("action") {
                        set.insert(T::from_polar(action)?);
                    }
                }
                Some(Err(e)) => return Err(e),
                None => break,
            };
        }

        Ok(set)
    }

    /// Clear out all files and rules that have been loaded.
    pub fn clear_rules(&mut self) -> crate::Result<()> {
        self.inner.clear_rules();
        check_messages!(self.inner);
        Ok(())
    }

    fn check_inline_queries(&self) -> crate::Result<()> {
        while let Some(q) = self.inner.next_inline_query(false) {
            let location = q.source_info();
            let query = Query::new(q, self.host.clone());
            match query.collect::<crate::Result<Vec<_>>>() {
                Ok(v) if !v.is_empty() => continue,
                Ok(_) => return Err(OsoError::InlineQueryFailedError { location }),
                Err(e) => return lazy_error!("error in inline query: {}", e),
            }
        }
        check_messages!(self.inner);
        Ok(())
    }

    // Register MROs, load Polar code, and check inline queries.
    fn load_sources(&mut self, sources: Vec<Source>) -> crate::Result<()> {
        self.host.register_mros()?;
        self.inner.load(sources)?;
        self.check_inline_queries()
    }

    /// Load a file containing Polar rules. All Polar files must end in `.polar`.
    #[deprecated(
        since = "0.20.1",
        note = "`Oso::load_file` has been deprecated in favor of `Oso::load_files` as of the 0.20 release.\n\nPlease see changelog for migration instructions: https://docs.osohq.com/project/changelogs/2021-09-15.html"
    )]
    pub fn load_file<P: AsRef<std::path::Path>>(&mut self, filename: P) -> crate::Result<()> {
        self.load_files(vec![filename])
    }

    /// Load files containing Polar rules. All Polar files must end in `.polar`.
    pub fn load_files<P: AsRef<std::path::Path>>(
        &mut self,
        filenames: Vec<P>,
    ) -> crate::Result<()> {
        if filenames.is_empty() {
            return Ok(());
        }

        let mut sources = Vec::with_capacity(filenames.len());

        for file in filenames {
            let file = file.as_ref();
            let filename = file.to_string_lossy().into_owned();
            if !file.extension().map_or(false, |ext| ext == "polar") {
                return Err(crate::OsoError::IncorrectFileType { filename });
            }
            let mut f = File::open(&file)?;
            let mut src = String::new();
            f.read_to_string(&mut src)?;
            sources.push(Source {
                src,
                filename: Some(filename),
            });
        }

        self.load_sources(sources)
    }

    /// Load a string of polar source directly.
    /// # Examples
    /// ```ignore
    /// oso.load_str("allow(a, b, c) if true;");
    /// ```
    pub fn load_str(&mut self, src: &str) -> crate::Result<()> {
        // TODO(gj): emit... some sort of warning?
        self.load_sources(vec![Source {
            src: src.to_owned(),
            filename: None,
        }])
    }

    /// Query the knowledge base. This can be an allow query or any other polar expression.
    /// # Examples
    /// ```ignore
    /// oso.query("x = 1 or x = 2");
    /// ```
    pub fn query(&self, s: &str) -> crate::Result<Query> {
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
    #[must_use = "Query that is not consumed does nothing."]
    pub fn query_rule(&self, name: &str, args: impl ToPolarList) -> crate::Result<Query> {
        let mut query_host = self.host.clone();
        let args = args
            .to_polar_list()
            .iter()
            .map(|value| value.to_term(&mut query_host))
            .collect();
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
        let class_name = self.host.cache_class(class.clone(), name)?;

        for hook in &class.register_hooks {
            hook.call(self)?;
        }
        self.register_constant(class, &class_name)
    }

    /// Register a rust type as a Polar constant.
    /// See [`oso::Class`] docs.
    pub fn register_constant<V: crate::host::ToPolar + Send + Sync>(
        &mut self,
        value: V,
        name: &str,
    ) -> crate::Result<()> {
        self.inner.register_constant(
            Symbol(name.to_string()),
            value.to_polar().to_term(&mut self.host),
        )?;
        Ok(())
    }
}

// Make sure the `Oso` object is threadsafe
#[cfg(test)]
static_assertions::assert_impl_all!(Oso: Send, Sync);
