#[macro_export]
macro_rules! lazy_error {
    ($($input:tt)*) => {
        Err($crate::errors::OsoError::Custom {
            message: format!($($input)*),
        })
    };
}

macro_rules! check_messages {
    ($core_obj:expr) => {};
}
