/// Create a custom [`OsoError`](crate::errors::OsoError), with a syntax similar to `format!()`.
#[macro_export]
macro_rules! lazy_error {
    ($($input:tt)*) => {
        Err($crate::errors::OsoError::Custom {
            message: format!($($input)*),
        })
    };
}

macro_rules! check_messages {
    ($core_obj:expr) => {
        while let Some(message) = $core_obj.next_message() {
            match message.kind {
                ::polar_core::messages::MessageKind::Print => ::std::println!("{}", &message.msg),
                ::polar_core::messages::MessageKind::Warning => {
                    ::std::eprintln!("[warning] {}", &message.msg)
                }
            }
        }
        true
    };
}
