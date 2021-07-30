use std::collections::HashMap;

use serde::{Serialize,Deserialize};

use crate::error::{PolarError, PolarResult};
use crate::kb::Bindings;
use crate::terms::*;
use crate::events::ResultEvent;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Type {
    Base {
        class_tag: String,
    },
    Relationship {
        kind: String,
        other_class_tag: String,
        my_field: String,
        other_field: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct FetchResult {
    id: i32
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Attrib {
    field: String,
    of: FetchResult
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Constraint {
    Eq {
        field: String,
        // @NOTE:(steve) I don't really want this to be Term. I want to make sure it's not a constraint
        // or a variable but just a ground value. Wish we had a type for that.
        value: Term
    },
    In {
        field: String,
    }
}

// The list of constraints passed to a fetching function.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct FetchRequest {
    class_tag: String,
    constraints: Vec<Constraint>

}

// A Set of fetch requests that may depend on the results of other fetches.
// resolve_order is the order to resolve the fetches in.
// result_id says which result to return.
// @Q(steve): Is it always the last one in the resolve_order?
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ResultSet {
    requests: HashMap<i32, FetchRequest>,
    resolve_order: Vec<i32>,
    result_id: i32
}

// @TODO(steve): There is probably more structure than just a union of ResultSets
// I think when we add OR constraints that this will be more of a tree.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct FilterPlan {
    result_sets: Vec<ResultSet>
}

pub type Types = HashMap<String, Type>;
pub type PartialResults = Vec<ResultEvent>;

pub fn build_filter_plan(
    types: HashMap<String, Type>,
    partial_results: Vec<ResultEvent>,
    variable: &str,
    class_tag: &str,
) -> PolarResult<FilterPlan> {
    Ok((FilterPlan {result_sets: vec![]}))
}

