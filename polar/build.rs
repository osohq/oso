extern crate cbindgen;
extern crate lalrpop;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    lalrpop::process_root().unwrap();

    // cbindgen::Builder::new()
    //     .with_crate(crate_dir)
    //     .with_language(cbindgen::Language::C)
    //     .with_no_includes()
    //     .with_item_prefix("polar_")
    //     .generate()
    //     .expect("Unable to generate bindings")
    //     .write_to_file("polar.h");
}
