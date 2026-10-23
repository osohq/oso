#[allow(unused)]
use crate::host::{Class, ClassBuilder};

#[cfg(feature = "uuid-06")]
impl crate::PolarClass for uuid_06::Uuid {
    fn get_polar_class_builder() -> ClassBuilder<uuid_06::Uuid> {
        Class::builder().name("Uuid").with_equality_check()
    }
}

#[cfg(feature = "uuid-07")]
impl crate::PolarClass for uuid_07::Uuid {
    fn get_polar_class_builder() -> ClassBuilder<uuid_07::Uuid> {
        Class::builder().name("Uuid").with_equality_check()
    }
}

#[cfg(feature = "uuid-10")]
impl crate::PolarClass for uuid_10::Uuid {
    fn get_polar_class_builder() -> ClassBuilder<uuid_10::Uuid> {
        Class::builder().name("Uuid").with_equality_check()
    }
}
