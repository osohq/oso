//! Communicate with the Polar virtual machine: load rules, make queries, etc/

use polar_core::roles_validation::ResultEvent;
use polar_core::terms::{Call, Symbol, Term, Value};

use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::sync::Arc;

use crate::host::Host;
use crate::query::Query;
use crate::{Class, FromPolar, OsoError, PolarValue, ToPolar, ToPolarList};

/// Oso is the main struct you interact with. It is an instance of the Oso authorization library
/// and contains the polar language knowledge base and query engine.
#[derive(Clone)]
pub struct Oso {
    inner: Arc<polar_core::polar::Polar>,
    host: Host,
    polar_roles_enabled: bool,
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

static OSO_INTERNAL_ROLES_HELPER: &str = "__oso_internal_roles_helpers__";

impl Oso {
    /// Create a new instance of Oso. Each instance is separate and can have different rules and classes loaded into it.
    pub fn new() -> Self {
        let inner = Arc::new(polar_core::polar::Polar::new());
        let host = Host::new(inner.clone());

        let mut oso = Self {
            inner,
            host,
            polar_roles_enabled: false,
        };

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
        self.reinitialize_roles()?;
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

    fn inner_load(&mut self, pol: &str, filename: Option<String>) -> crate::Result<()> {
        self.inner.load(pol, filename)?;
        self.check_inline_queries()?;
        self.reinitialize_roles()
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
        self.inner_load(&policy, Some(file.to_string_lossy().into_owned()))
    }

    fn reinitialize_roles(&mut self) -> crate::Result<()> {
        if !self.polar_roles_enabled {
            return Ok(());
        }
        self.polar_roles_enabled = false;
        self.enable_roles()
    }

    /// Load a string of polar source directly.
    /// # Examples
    /// ```ignore
    /// oso.load_str("allow(a, b, c) if true;");
    /// ```
    pub fn load_str(&mut self, s: &str) -> crate::Result<()> {
        self.inner_load(s, None)
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
        self.register_constant(class, &class_name)?;

        Ok(())
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
        );
        Ok(())
    }

    pub fn enable_roles(&mut self) -> crate::Result<()> {
        if self.polar_roles_enabled {
            return Ok(());
        }

        self.inner.enable_roles()?;

        if !self.host.has_class(OSO_INTERNAL_ROLES_HELPER) {
            self.register_class(
                Class::builder::<()>()
                    .name(OSO_INTERNAL_ROLES_HELPER)
                    .add_class_method("join", |sep: String, mut l: String, r: String| {
                        l.push_str(&sep as &str);
                        l.push_str(&r as &str);
                        l
                    })
                    .build(),
            )?;
        }

        let mut validation_results: Vec<Vec<ResultEvent>> = Vec::new();

        while let Some(q) = self.inner.next_inline_query(false) {
            let src = q.source_info();
            let mut host_ = self.host.clone();
            host_.accept_expression = true;
            let res = Query::new(q, host_).collect::<crate::Result<Vec<_>>>()?;
            if res.is_empty() {
                return Err(OsoError::InlineQueryFailedError { location: src });
            }
            validation_results.push(res.into_iter().map(|rs| rs.into_event()).collect());
        }

        self.inner.validate_roles_config(validation_results)?;

        check_messages!(self.inner);
        self.polar_roles_enabled = true;
        Ok(())
    }
}

// Make sure the `Oso` object is threadsafe
#[cfg(test)]
static_assertions::assert_impl_all!(Oso: Send, Sync);
