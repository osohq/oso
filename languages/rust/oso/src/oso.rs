//! # Oso module
//!
//! Communicate with the Polar virtual machine: load rules, make queries, etc.
use crate::{
    host::Host, query::Query, Class, FromPolar, OsoError, PolarValue, Result, ToPolar, ToPolarList,
};
use polar_core::{
    polar::Polar,
    sources::Source,
    terms::{Call, Symbol, Term, Value},
};
use std::{collections::HashSet, fs::File, hash::Hash, io::Read, sync::Arc};

/// Instance of the Oso authorization library.
///
/// This is the main struct you interact with. Contains the polar language knowledge base and query
/// engine.
///
/// # Usage
///
/// Typically, you will create a new instance of Oso, load some definitions into it that you
/// can use to determine authorization and then use [`is_allowed()`](Oso::is_allowed) to check for
/// authorization.
///
/// ```rust
/// # use oso::Oso;
/// let mut oso = Oso::new();
///
/// // allow any actor to perform read on any resource
/// oso.load_str(r#"allow(_actor, "read", _resource);"#).unwrap();
///
/// assert!(oso.is_allowed("me", "read", "book").unwrap());
/// ```
///
/// To make Oso more useful to you, you can augment it with your own custom Rust types by using the
/// [`register_class()`](Oso::register_class) method.
///
/// Besides only checking if something is allowed, Oso can also tell you all of the actions that an
/// actor may take on a resource using the [`get_allowed_actions()`](Oso::get_allowed_actions)
/// method.
///
/// # Quickstart
///
/// You can check out the [Quickstart
/// guide](https://docs.osohq.com/rust/getting-started/quickstart.html) for more information on how
/// to get started using Oso.
#[derive(Clone)]
pub struct Oso {
    inner: Arc<Polar>,
    host: Host,
}

impl Default for Oso {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an `action` used in an `allow` rule.
///
/// When the action is bound to a concrete value (e.g. a string) this returns an
/// [`Action::Typed`].  If _any_ actions are allowed, then the [`Action::Any`] variant is
/// returned. By default, the type of the action is a [`String`].
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Action<T = String> {
    /// Any action is allowed.
    Any,
    /// This specific action is allowed.
    Typed(T),
}

impl<T: FromPolar> FromPolar for Action<T> {
    fn from_polar(val: PolarValue) -> Result<Self> {
        if matches!(val, PolarValue::Variable(_)) {
            Ok(Action::Any)
        } else {
            T::from_polar(val).map(Action::Typed)
        }
    }
}

impl Oso {
    /// Create a new instance of Oso.
    ///
    /// Each instance is separate and can have different rules and classes loaded into it.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// # use oso::Oso;
    /// let oso = Oso::new();
    /// ```
    pub fn new() -> Self {
        let inner = Arc::new(polar_core::polar::Polar::new());
        let host = Host::new(inner.clone());

        let mut oso = Self { inner, host };

        for class in crate::builtins::classes() {
            oso.register_class(class)
                .expect("failed to register builtin class");
        }
        oso.register_constant(Option::<PolarValue>::None, "nil")
            .expect("failed to register the constant None");
        oso
    }

    /// Test if an `actor` is allowed to perform an `action` on a `resource`.
    ///
    /// High level interface for authorization decisions. Makes an allow query with the given
    /// actor, action and resource and returns `true` or `false`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// # use oso::Oso;
    /// let mut oso = Oso::new();
    ///
    /// // allow any actor to perform read on any resource
    /// oso.load_str(r#"allow(_actor, "read", _resource);"#).unwrap();
    ///
    /// assert_eq!(oso.is_allowed("me", "read", "book").unwrap(), true);
    /// assert_eq!(oso.is_allowed("me", "steal", "book").unwrap(), false);
    /// ```
    pub fn is_allowed<Actor, Action, Resource>(
        &self,
        actor: Actor,
        action: Action,
        resource: Resource,
    ) -> Result<bool>
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
    ///
    /// Returns a [`HashSet`] of actions, typed according the return value. It can return
    /// [`Action`] structs, which can encode that an actor can do anything with [`Action::Any`], or
    /// any type that implements [`FromPolar`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// # use oso::{Action, Oso};
    /// # use std::collections::HashSet;
    /// let mut oso = Oso::new();
    ///
    /// // anyone can read anything, and thomas can drive and sell a car.
    /// oso.load_str(r#"
    ///     allow(_actor, "read", "book");
    ///     allow("thomas", action, "car") if action in ["drive", "sell"];
    /// "#).unwrap();
    ///
    /// // anyone can read a book
    /// let actions: HashSet<Action> = oso.get_allowed_actions("thomas", "book").unwrap();
    /// assert_eq!(actions, [Action::Typed("read".into())].into());
    ///
    /// // only thomas can drive and sell his car
    /// let actions: HashSet<String> = oso.get_allowed_actions("thomas", "car").unwrap();
    /// assert_eq!(actions, [
    ///     "drive".into(),
    ///     "sell".into()
    /// ].into());
    /// ```
    ///
    /// If you prefer not to use "stringly-typed" actions, you can define your own action type.
    /// This method even works when using that action type. Here is an example, where an enum is
    /// used as an action type:
    ///
    /// ```rust
    /// # use oso::{Action, Oso, PolarClass};
    /// # use std::collections::HashSet;
    /// let mut oso = Oso::new();
    ///
    /// #[derive(PolarClass, PartialEq, Clone, Debug, Eq, Hash)]
    /// enum MyAction {
    ///     Read,
    ///     Write,
    /// }
    ///
    /// oso.register_class(
    ///     MyAction::get_polar_class_builder()
    ///     .with_equality_check()
    ///     .add_constant(MyAction::Read, "Read")
    ///     .add_constant(MyAction::Write, "Write")
    ///     .build()
    /// ).unwrap();
    ///
    /// oso.load_str(r#"
    ///     allow("backend", MyAction::Read, _resource);
    ///     allow("backend", MyAction::Write, "database");
    /// "#).unwrap();
    ///
    /// let actions: HashSet<MyAction> = oso.get_allowed_actions("backend", "database").unwrap();
    /// assert_eq!(actions, [
    ///     MyAction::Read,
    ///     MyAction::Write,
    /// ].into());
    /// ```
    pub fn get_allowed_actions<Actor, Resource, T>(
        &self,
        actor: Actor,
        resource: Resource,
    ) -> Result<HashSet<T>>
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
    ///
    /// This message may return an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use oso::Oso;
    /// let mut oso = Oso::new();
    ///
    /// // load some rules
    /// oso.load_str("allow(_actor, _action, _resource) if true;").unwrap();
    ///
    /// // clear rules
    /// oso.clear_rules().unwrap();
    /// ```
    pub fn clear_rules(&mut self) -> Result<()> {
        self.inner.clear_rules();
        check_messages!(self.inner);
        Ok(())
    }

    fn check_inline_queries(&self) -> Result<()> {
        while let Some(q) = self.inner.next_inline_query(false) {
            let location = q.source_info();
            let query = Query::new(q, self.host.clone());
            match query.collect::<Result<Vec<_>>>() {
                Ok(v) if !v.is_empty() => continue,
                Ok(_) => return Err(OsoError::InlineQueryFailedError { location }),
                Err(e) => return lazy_error!("error in inline query: {}", e),
            }
        }
        check_messages!(self.inner);
        Ok(())
    }

    // Register MROs, load Polar code, and check inline queries.
    fn load_sources(&mut self, sources: Vec<Source>) -> Result<()> {
        self.host.register_mros()?;
        self.inner.load(sources)?;
        self.check_inline_queries()
    }

    /// Load a file containing Polar rules. All Polar files must end in `.polar`.
    #[deprecated(
        since = "0.20.1",
        note = "`Oso::load_file` has been deprecated in favor of `Oso::load_files` as of the 0.20 release. Please see changelog for migration instructions: <https://docs.osohq.com/project/changelogs/2021-09-15.html>"
    )]
    pub fn load_file<P: AsRef<std::path::Path>>(&mut self, filename: P) -> Result<()> {
        self.load_files(vec![filename])
    }

    /// Load files containing Polar rules.
    ///
    /// All Polar files must have the `.polar` extension.
    ///
    /// ```rust
    /// # use oso::Oso;
    /// let mut oso = Oso::new();
    ///
    /// oso.load_files(vec!["../test.polar"]).unwrap();
    /// ```
    pub fn load_files<P: AsRef<std::path::Path>>(&mut self, filenames: Vec<P>) -> Result<()> {
        if filenames.is_empty() {
            return Ok(());
        }

        let mut sources = Vec::with_capacity(filenames.len());

        for file in filenames {
            let file = file.as_ref();
            let filename = file.to_string_lossy().into_owned();
            if !file.extension().map_or(false, |ext| ext == "polar") {
                return Err(OsoError::IncorrectFileType { filename });
            }
            let mut f = File::open(file)?;
            let mut src = String::new();
            f.read_to_string(&mut src)?;
            sources.push(Source::new_with_name(filename, src));
        }

        self.load_sources(sources)
    }

    /// Load a string of polar source directly.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// # use oso::Oso;
    /// let mut oso = Oso::new();
    ///
    /// oso.load_str("allow(_actor, _action, _resource) if true;").unwrap();
    /// ```
    ///
    /// Loading a source that is baked into the binary at compile-time:
    ///
    /// ```rust
    /// # use oso::Oso;
    /// let mut oso = Oso::new();
    ///
    /// oso.load_str(include_str!("../test.polar")).unwrap();
    /// ```
    pub fn load_str(&mut self, src: &str) -> Result<()> {
        // TODO(gj): emit... some sort of warning?
        self.load_sources(vec![Source::new(src)])
    }

    /// Query the knowledge base.
    ///
    /// This can be an allow query (like `allow(actor, "read", resource) or any other Polar expression.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// # use oso::Oso;
    /// let mut oso = Oso::new();
    ///
    /// oso.register_constant(2, "x").unwrap();
    ///
    /// oso.query("x = 1 or x = 2").unwrap();
    /// ```
    pub fn query(&self, s: &str) -> Result<Query> {
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
    pub fn query_rule(&self, name: &str, args: impl ToPolarList) -> Result<Query> {
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
    ///
    /// See also the [`Class`] docs.
    ///
    /// Typically, you can simply derive [`PolarClass`] for your custom types and then use the
    /// [`get_polar_class()`](crate::PolarClass::get_polar_class) method.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use oso::{PolarClass, Oso};
    /// #[derive(PolarClass)]
    /// struct MyClass;
    ///
    /// let mut oso = Oso::new();
    ///
    /// oso.register_class(MyClass::get_polar_class());
    /// ```
    ///
    /// In order to customize your class for Polar, you can use the methods of
    /// [`ClassBuilder`](crate::ClassBuilder).
    ///
    /// ```rust
    /// # use oso::{PolarClass, Oso};
    /// #[derive(PolarClass, PartialEq, Clone, Debug, Eq, Hash)]
    /// enum Service {
    ///     Database,
    ///     Frontend,
    ///     Backend,
    /// }
    ///
    /// let class = Service::get_polar_class_builder()
    ///     .with_equality_check()
    ///     .add_constant(Service::Frontend, "Frontend")
    ///     .add_constant(Service::Backend, "Backend")
    ///     .add_constant(Service::Database, "Database")
    ///     .build();
    ///
    /// let mut oso = Oso::new();
    ///
    /// oso.register_class(class).unwrap();
    /// ```
    pub fn register_class(&mut self, class: Class) -> Result<()> {
        let name = class.name.clone();
        let class_name = self.host.cache_class(class.clone(), name)?;

        for hook in &class.register_hooks {
            hook.call(self)?;
        }
        self.register_constant(class, &class_name)
    }

    /// Register a Rust value as a Polar constant.
    ///
    /// The Rust value must implement [`ToPolar`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// # use oso::Oso;
    /// let mut oso = Oso::new();
    ///
    /// oso.register_constant(std::f64::consts::PI, "PI");
    /// ```
    pub fn register_constant<V: ToPolar + Send + Sync>(
        &mut self,
        value: V,
        name: &str,
    ) -> Result<()> {
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
