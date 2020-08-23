use super::*;

#[derive(Clone, Default)]
struct Type;

pub fn type_class() -> Class {
    let class = Class::<Type>::with_default();

    // class.instance_check = Arc::new()

    class.erase_type()
}

fn boolean() -> Class<bool> {
    Class::<bool>::with_default().name("Boolean")
}

fn integer() -> Class<i64> {
    Class::<i64>::with_default().name("Integer")
}

fn float() -> Class<f64> {
    Class::<f64>::with_default().name("Float")
}

fn number() -> Class<Numeric> {
    Class::<Numeric>::with_constructor(|| Numeric::Integer(0)).name("Number")
}

fn list() -> Class<Vec<Term>> {
    Class::<Vec<Term>>::with_default().name("List")
}

fn dictionary() -> Class<HashMap<Name, Term>> {
    Class::<HashMap<Name, Term>>::with_default().name("Dictionary")
}

fn string() -> Class<String> {
    Class::<String>::with_default()
        .add_method("ends_with", |s: &String, pat: String| s.ends_with(&pat))
}

/// Returns the builtin types, the name, class, and instance
pub fn constants(host: &mut Host) -> Vec<(Name, Class, Term)> {
    let type_class = type_class();
    host.cache_class(type_class.clone(), None);

    let to_term = |class: Class, host: &mut Host| -> Term {
        let repr = format!("type<{}>", class.name);
        let instance = type_class.cast_to_instance(class);
        let instance = host.cache_instance(instance, None);
        Term::new_from_ffi(Value::ExternalInstance(ExternalInstance {
            constructor: None,
            repr: Some(repr),
            instance_id: instance,
        }))
    };

    vec![
        (Name("Boolean".to_string()), boolean().erase_type()),
        (Name("Number".to_string()), number().erase_type()),
        (Name("Integer".to_string()), integer().erase_type()),
        (Name("Float".to_string()), float().erase_type()),
        (Name("String".to_string()), string().erase_type()),
        (Name("List".to_string()), list().erase_type()),
        (Name("Dictionary".to_string()), dictionary().erase_type()),
    ]
    .into_iter()
    .map(|(name, class)| {
        let instance = to_term(class.clone(), host);
        (name, class, instance)
    })
    .collect()
}
