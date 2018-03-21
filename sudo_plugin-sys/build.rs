extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::Builder::default()
        .header("include/bindings.h")
        // we provide a default sudo_plugin.h in case it's not available
        // on the system
        .clang_arg("--include-directory-after=include")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
