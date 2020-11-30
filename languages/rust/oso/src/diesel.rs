#[cfg(feature = "uuid_v06")]
impl crate::PolarClass for uuid_v06::Uuid {
    fn get_polar_class_builder() -> crate::host::ClassBuilder<uuid_v06::Uuid> {
        crate::host::Class::builder()
            .name("Uuid")
            .with_equality_check()
    }
}
