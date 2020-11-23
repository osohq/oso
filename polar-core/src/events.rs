use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::kb::*;
use super::runnable::Runnable;
use super::terms::*;
use super::traces::*;

#[allow(clippy::large_enum_variant)]
#[must_use]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum QueryEvent {
    None,

    /// This runnable is complete with `result`.
    Done {
        result: bool,
    },

    /// Run `runnable`, and report the result to its parent using `call_id`
    /// when it completes.
    #[serde(skip)]
    Run {
        call_id: u64,
        runnable: Box<dyn Runnable>,
    },

    Debug {
        message: String,
    },

    MakeExternal {
        instance_id: u64,
        constructor: Term,
    },

    ExternalCall {
        /// Persistent id across all requests for results from the same external call.
        call_id: u64,
        /// The external instance to make this call on.
        instance: Term,
        /// Field name to lookup or method name to call. A class name indicates a constructor
        /// should be called.
        attribute: Symbol,
        /// List of arguments to a method call.
        args: Option<Vec<Term>>,
        /// A map of keyword arguments to a method call.
        kwargs: Option<BTreeMap<Symbol, Term>>,
    },

    /// Checks if the instance is an instance of (a subclass of) the class_tag.
    ExternalIsa {
        call_id: u64,
        instance: Term,
        class_tag: Symbol,
    },

    /// Checks if the left is more specific than right with respect to instance.
    ExternalIsSubSpecializer {
        call_id: u64,
        instance_id: u64,
        left_class_tag: Symbol,
        right_class_tag: Symbol,
    },

    /// Checks if left class tag is a subclass or the same class as right.
    ExternalIsSubclass {
        call_id: u64,
        left_class_tag: Symbol,
        right_class_tag: Symbol,
    },

    /// Unifies two external instances.
    ExternalUnify {
        call_id: u64,
        left_instance_id: u64,
        right_instance_id: u64,
    },

    Result {
        bindings: Bindings,
        trace: Option<TraceResult>,
    },

    ExternalOp {
        call_id: u64,
        operator: Operator,
        args: TermList,
    },

    NextExternal {
        call_id: u64,
        iterable: Term,
    },
}
