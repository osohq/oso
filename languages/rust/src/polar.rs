/// Communicate with the Polar virtual machine: load rules, make queries, etc/
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::host::Host;

pub struct Polar {
    inner: Rc<crate::PolarCore>,
    host: Arc<Mutex<Host>>,
    load_queue: Vec<String>,
}

impl Polar {
    pub fn new() -> Self {
        let inner = Rc::new(crate::PolarCore::new(None));
        Self {
            host: Arc::new(Mutex::new(Host::new(Rc::downgrade(&inner)))),
            inner,
            load_queue: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn load_file(&mut self, file: &str) -> anyhow::Result<()> {
        let mut f = File::open(&file)?;
        let mut policy = String::new();
        f.read_to_string(&mut policy)?;
        self.inner.load_file(&policy, Some(file.to_string()))?;
        Ok(())
    }

    pub fn load_str(&mut self, s: &str) -> anyhow::Result<()> {
        Ok(self.inner.load(s)?)
    }

    pub fn query(&mut self, s: &str) -> anyhow::Result<crate::query::Query> {
        let query = self.inner.new_query(s, false)?;
        let query = crate::query::Query::new(query, self.host.clone());
        Ok(query)
    }

    pub fn register_class(
        &mut self,
        class: crate::host::Class,
        name: Option<String>,
    ) -> anyhow::Result<()> {
        let mut host = self.host.lock().unwrap();
        let _name = host.cache_class(class, name.map(polar_core::types::Symbol));
        Ok(())
    }
    // pub fn query_rule<'a>(
    //     &mut self,
    //     name: &str,
    //     args: impl IntoIterator<Item = &'a dyn crate::host::ToPolar>,
    // ) -> anyhow::Result<crate::query::Query> {
    //     let args = args.into_iter().map(|arg| arg.to_polar(&mut self.host.lock().unwrap())).collect();
    //     let query = polar_core::types::Term::new_from_ffi(value)

    // }
}

// class Polar:
//     """Polar API"""

//     def query_rule(self, name, *args):
//         """Query for rule with name ``name`` and arguments ``args``.

//         :param name: The name of the predicate to query.
//         :param args: Arguments for the predicate.

//         :return: The result of the query.
//         """
//         return self.query(Predicate(name=name, args=args))

//     def register_class(self, cls, *, name=None, from_polar=None):
//         """Register `cls` as a class accessible by Polar. `from_polar` can
//         either be a method or a string. In the case of a string, Polar will
//         look for the method using `getattr(cls, from_polar)`."""
//         cls_name = self.host.cache_class(cls, name, from_polar)
//         self.register_constant(cls_name, cls)

//     def register_constant(self, name, value):
//         """Register `value` as a Polar constant variable called `name`."""
//         name = to_c_str(name)
//         value = ffi_serialize(self.host.to_polar_term(value))
//         lib.polar_register_constant(self.ffi_polar, name, value)

//     def _load_queued_files(self):
//         """Load queued policy files into the knowledge base."""
//         while self.load_queue:
//             filename = self.load_queue.pop(0)
//             with open(filename) as file:
//                 load_str(self.ffi_polar, file.read(), filename)

// def polar_class(_cls=None, *, name=None, from_polar=None):
//     """Decorator to register a Python class with Polar.
//     An alternative to ``register_class()``.

//     :param str from_polar: Name of class function to create a new instance from ``fields``.
//                            Defaults to class constructor.
//     """

//     def wrap(cls):
//         cls_name = cls.__name__ if name is None else name
//         CLASSES[cls_name] = cls
//         CONSTRUCTORS[cls_name] = from_polar or cls
//         return cls

//     if _cls is None:
//         return wrap

//     return wrap(_cls)
