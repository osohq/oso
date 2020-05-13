extern crate cbindgen;
extern crate lalrpop;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    lalrpop::process_root().unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_no_includes()
        .with_item_prefix("polar_")
        .generate()
        .map(|res| Some(res))
        .or_else(|err| {
            // Continue on syntax errors
            if let cbindgen::Error::ParseSyntaxError { .. } = err {
                Ok(None)
            } else if let cbindgen::Error::CargoMetadata { .. } = err {
                Ok(None)
            } else {
                Err(err)
            }
        })
        .expect("Unable to generate bindings")
        .and_then(|res| Some(res.write_to_file("polar.h")));
}
