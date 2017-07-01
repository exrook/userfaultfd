extern crate bindgen;

use std::env;
use std::path::PathBuf;

use bindgen::callbacks::{ParseCallbacks,IntKind};

#[derive(Debug)]
struct Callback();

impl ParseCallbacks for Callback {
    fn parsed_macro(&self, name: &str) {
        println!("SDS: {}", name);
    }
    fn int_macro(&self, name: &str, value: i64) -> Option<IntKind> {
        println!("{}", name);
        None
    }
}

fn main() {
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .unstable_rust(false)
        .parse_callbacks(Box::from(Callback()))
        .derive_default(true)
        .header("wrapper.h")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

