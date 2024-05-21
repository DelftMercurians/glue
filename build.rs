use std::env;
use std::path::PathBuf;
use bindgen::callbacks::{
    DeriveInfo, ParseCallbacks,
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex, RwLock};

#[allow(dead_code)]
#[derive(Debug)]
struct MacroCallback {
    macros: Arc<RwLock<HashSet<String>>>,
    seen_hellos: Mutex<u32>,
    seen_funcs: Mutex<u32>,
}

impl ParseCallbacks for MacroCallback {
    fn add_derives(&self, info: &DeriveInfo<'_>) -> Vec<String> {
        if info.name == "HG_Status" {
            vec![
                "FromPrimitive".into(),
                "ToPrimitive".into(),
            ]
        } else {
            vec![]
        }
    }
}



fn main() {

    let macros = Arc::new(RwLock::new(HashSet::new()));

    // Tell cargo to look for shared libraries in the specified directory
    // println!("cargo:rustc-link-search=C:\\Users\\thomas\\.platformio\\packages\\toolchain-gccarmnoneeabi\\arm-none-eabi\\include\\");

    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    // println!("cargo:rustc-link-lib=bz2");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .raw_line("use num_derive::{ToPrimitive,FromPrimitive};")
        .raw_line("use num_traits::FromPrimitive;")
        .header("wrapper.hpp")
        .rustified_enum("HG::Status")
        .rustified_enum("CAN::DEVICE_ID")
        .rustified_enum("CAN::MESSAGE_ID")
        .rustified_enum("CAN::ACCESS")
        .rustified_enum("CAN::VARIABLE")
        .rustified_enum("Radio::PrimaryStatusHF")
        .rustified_enum("Radio::PrimaryStatusLF")
        .clang_arg("--target=arm-none-eabi")
        .clang_arg("-DUSING_BINDGEN")
        .blocklist_file("^(.*can_id\\.h$)$")
        .parse_callbacks(Box::new(MacroCallback {
            macros: macros.clone(),
            seen_hellos: Mutex::new(0),
            seen_funcs: Mutex::new(0),
        }))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:warning={}", env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}