use super::rules::*;
use super::terms::*;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Rule(Arc<Rule>),
    Term(Term),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    pub node: Node,
    pub children: Vec<Rc<Trace>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceResult {
    pub trace: Rc<Trace>,
    pub formatted: String,
}

// step out is break after the next trace pop
// step into is break after the next query (self.trace.push)
// break after the next query as long as the current trace stack is not longer than the one when you started stepping.
