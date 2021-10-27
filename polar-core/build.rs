fn main() {
    #[cfg(feature = "build-parser")]
    lalrpop::Configuration::new()
        .emit_rerun_directives(true)
        .generate_in_source_tree()
        .process()
        .unwrap()
}
