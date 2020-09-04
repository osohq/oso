use super::formatting::source_lines;
use super::kb::*;
use super::rules::*;
use super::terms::*;

use std::collections::{hash_map::Entry, HashMap};

fn common_misspellings(t: &str) -> Option<String> {
    let misspelled_type = match t {
        "integer" => "Integer",
        "int" => "Integer",
        "i32" => "Integer",
        "i64" => "Integer",
        "u32" => "Integer",
        "u64" => "Integer",
        "usize" => "Integer",
        "size_t" => "Integer",
        "float" => "Float",
        "f32" => "Float",
        "f64" => "Float",
        "double" => "Float",
        "char" => "String",
        "str" => "String",
        "string" => "String",
        "list" => "List",
        "array" => "List",
        "Array" => "List",
        "dict" => "Dictionary",
        "Dict" => "Dictionary",
        "dictionary" => "Dictionary",
        "hash" => "Dictionary",
        "Hash" => "Dictionary",
        "map" => "Dictionary",
        "Map" => "Dictionary",
        "HashMap" => "Dictionary",
        "hashmap" => "Dictionary",
        "hash_map" => "Dictionary",
        _ => return None,
    };
    Some(misspelled_type.to_owned())
}

/// Warn about singleton variables and unknown specializers in a rule,
/// except those whose names start with `_`.
pub fn check_singletons(rule: &Rule, kb: &KnowledgeBase) -> Vec<String> {
    let mut warnings = vec![];
    let mut singletons = HashMap::<Symbol, Option<Term>>::new();
    let mut check_term = |term: &Term| {
        if let Value::Variable(sym)
        | Value::RestVariable(sym)
        | Value::Pattern(Pattern::Instance(InstanceLiteral { tag: sym, .. })) = term.value()
        {
            if !sym.0.starts_with('_') && !kb.is_constant(sym) {
                match singletons.entry(sym.clone()) {
                    Entry::Occupied(mut o) => {
                        o.insert(None);
                    }
                    Entry::Vacant(v) => {
                        v.insert(Some(term.clone()));
                    }
                }
            }
        }
        term.clone()
    };

    for param in &rule.params {
        param.parameter.clone().map_replace(&mut check_term);
        if let Some(mut spec) = param.specializer.clone() {
            spec.map_replace(&mut check_term);
        }
    }
    rule.body.clone().map_replace(&mut check_term);

    let mut singletons = singletons
        .into_iter()
        .collect::<Vec<(Symbol, Option<Term>)>>();
    singletons.sort_by_key(|(_sym, term)| term.as_ref().map_or(0, |term| term.offset()));
    for (sym, singleton) in singletons {
        if let Some(term) = singleton {
            let mut msg = if let Value::Pattern(..) = term.value() {
                let mut msg = format!("Unknown specializer {}", sym);
                if let Some(t) = common_misspellings(&sym.0) {
                    msg.push_str(&format!(", did you mean {}?", t));
                }
                msg
            } else {
                format!("Singleton variable {} is unused or undefined, see <https://docs.oso.dev/using/polar-syntax.html#variables>", sym)
            };
            if let Some(ref source) = term
                .get_source_id()
                .and_then(|id| kb.sources.get_source(id))
            {
                msg = format!("{}\n{}", msg, source_lines(source, term.offset(), 0));
            }
            warnings.push(msg)
        }
    }
    warnings
}
