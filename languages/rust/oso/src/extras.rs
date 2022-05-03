#[cfg(feature = "uuid-06")]
impl crate::PolarClass for uuid_06::Uuid {
    fn get_polar_class_builder() -> crate::host::ClassBuilder<uuid_06::Uuid> {
        crate::host::Class::builder()
            .name("Uuid")
            .with_equality_check()
    }
}

#[cfg(feature = "uuid-07")]
impl crate::PolarClass for uuid_07::Uuid {
    fn get_polar_class_builder() -> crate::host::ClassBuilder<uuid_07::Uuid> {
        crate::host::Class::builder()
            .name("Uuid")
            .with_equality_check()
    }
}

#[cfg(feature = "uuid-10")]
impl crate::PolarClass for uuid_10::Uuid {
    fn get_polar_class_builder() -> crate::host::ClassBuilder<uuid_10::Uuid> {
        crate::host::Class::builder()
            .name("Uuid")
            .with_equality_check()
    }
}
