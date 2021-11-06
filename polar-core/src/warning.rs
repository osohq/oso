use std::fmt;

use indoc::indoc;

use super::terms::{InstanceLiteral, Pattern, Term, Value};

#[derive(Debug)]
pub enum Warning {
    // validation | general
    AmbiguousPrecedence { term: Term },
    // validation | enforcement
    MissingAllowRule,
    // validation | resource blocks
    MissingHasPermissionRule,
    // validation | general
    UnknownSpecializer { term: Term },
}

const AMBIGUOUS_PRECEDENCE_MSG: &str = indoc! {"
    Expression without parentheses could be ambiguous.
    Prior to 0.20, `x and y or z` would parse as `x and (y or z)`.
    As of 0.20, it parses as `(x and y) or z`, matching other languages.
"};

const MISSING_ALLOW_RULE_MSG: &str = indoc! {"
    Your policy does not contain an allow rule, which usually means
    that no actions are allowed. Did you mean to add an allow rule to
    the top of your policy?

      allow(actor, action, resource) if ...

    You can also suppress this warning by adding an allow_field or allow_request
    rule. For more information about allow rules, see:

      https://docs.osohq.com/reference/polar/builtin_rule_types.html#allow
"};

const MISSING_HAS_PERMISSION_RULE_MSG: &str = indoc! {"
    Warning: your policy uses resource blocks but does not call the
    has_permission rule. This means that permissions you define in a
    resource block will not have any effect. Did you mean to include a
    call to has_permission in a top-level allow rule?

      allow(actor, action, resource) if
          has_permission(actor, action, resource);

    For more information about resource blocks, see https://docs.osohq.com/any/reference/polar/polar-syntax.html#actor-and-resource-blocks
"};

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Warning::*;
        match self {
            AmbiguousPrecedence { .. } => write!(f, "{}", AMBIGUOUS_PRECEDENCE_MSG)?,
            MissingAllowRule => write!(f, "{}", MISSING_ALLOW_RULE_MSG)?,
            MissingHasPermissionRule => write!(f, "{}", MISSING_HAS_PERMISSION_RULE_MSG)?,
            UnknownSpecializer { term } => {
                write!(f, "Unknown specializer {}", term)?;
                if let Some(suggestion) = common_specializer_misspellings(term) {
                    write!(f, ", did you mean {}?", suggestion)?;
                }
            }
        }
        Ok(())
    }
}

fn common_specializer_misspellings(term: &Term) -> Option<&str> {
    if let Value::Pattern(Pattern::Instance(InstanceLiteral { tag, .. })) = term.value() {
        let misspelled_type = match tag.0.as_ref() {
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
        return Some(misspelled_type);
    }
    None
}
