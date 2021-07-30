use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{PolarError, PolarResult};
use crate::events::ResultEvent;
use crate::kb::Bindings;
use crate::terms::*;

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
    id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Attrib {
    field: String,
    of: FetchResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Constraint {
    Eq {
        field: String,
        // @NOTE:(steve) I don't really want this to be Term. I want to make sure it's not a constraint
        // or a variable but just a ground value. Wish we had a type for that.
        value: Term,
    },
    In {
        field: String,
    },
}

// The list of constraints passed to a fetching function.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct FetchRequest {
    class_tag: String,
    constraints: Vec<Constraint>,
}

// A Set of fetch requests that may depend on the results of other fetches.
// resolve_order is the order to resolve the fetches in.
// result_id says which result to return.
// @Q(steve): Is it always the last one in the resolve_order?
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ResultSet {
    requests: HashMap<i32, FetchRequest>,
    resolve_order: Vec<i32>,
    result_id: i32,
}

// @TODO(steve): There is probably more structure than just a union of ResultSets
// I think when we add OR constraints that this will be more of a tree.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct FilterPlan {
    result_sets: Vec<ResultSet>,
}

pub type Types = HashMap<String, HashMap<String, Type>>;
pub type PartialResults = Vec<ResultEvent>;

pub fn build_filter_plan(
    types: Types,
    partial_results: PartialResults,
    variable: &str,
    class_tag: &str,
) -> PolarResult<FilterPlan> {
    

    Ok((FilterPlan {
        result_sets: vec![],
    }))
}

// [
// FilterPlan(
// data_sets={
// 0: Constraints(
// cls="Foo",
// constraints=[Constraint(kind="Eq", field="is_fooey", value=True)],
// )
// },
// resolve_order=[0],
// result_set=0,
// )
// ]

mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let mut types = HashMap::new();

        let mut foo_types = HashMap::new();
        foo_types.insert(
            "bar_name",
            Type::Base {
                class_tag: "String".to_owned(),
            },
        );
        foo_types.insert(
            "bar",
            Type::Relationship {
                kind: "parent".to_owned(),
                other_class_tag: "Bar".to_owned(),
                my_field: "bar_name".to_owned(),
                other_field: "name".to_owned(),
            },
        );
        types.insert("Foo", foo_types);

        println!("{}", serde_json::to_string(&types).unwrap())
    }
}
