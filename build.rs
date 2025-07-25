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
        match info.name {
            "HG_Status" | "Radio_MessageType" =>
                vec![
                    "FromPrimitive".into(),
                    "ToPrimitive".into(),
                    "AsBytes".into(),
                ],
            "HG_Pose" | "CAN_VARIABLE" | "Radio_ConfigMessage" | "Radio_Command" | "Radio_Reply" | "Radio_GlobalCommand" |
            "Radio_PrimaryStatusHF" | "Radio_PrimaryStatusLF" |
            "Radio_ImuReadings" | "MessageType" | "Radio_Message" | "Radio_RobotCommand" |
            "Radio_Message__bindgen_ty_1" | "Radio_Message__bindgen_ty_1__bindgen_ty_1" | "Radio_Message__bindgen_ty_1__bindgen_ty_2" |
            "Radio_MultiConfigMessage" | "HG_ConfigOperation" | "HG_VariableType" |
            "Radio_OdometryReading" | "Radio_OverrideOdometry" | "Radio_Access" | 
            "Radio_MessageWrapper" =>
                vec![
                    "AsBytes".into(),
                ],
            "HG_Variable" =>
                vec![
                    "EnumIter".into(),
                    "AsBytes".into(),
                ],
            _ => vec![],
        }
    }
}



fn main() {
    let macros = Arc::new(RwLock::new(HashSet::new()));

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    #[allow(unused_mut)]
    let mut bindgen_builder = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .raw_line("use num_derive::{ToPrimitive,FromPrimitive};")
        .raw_line("use zerocopy_derive::AsBytes;")
        .raw_line("use strum_macros::EnumIter;")
        .header("wrapper.hpp")
        .derive_debug(true)
        .rustified_enum("HG::Status")
        .rustified_enum("CAN::DEVICE_ID")
        .rustified_enum("CAN::MESSAGE_ID")
        .rustified_enum("CAN::ACCESS")
        .rustified_enum("CAN::VARIABLE")
        .rustified_enum("Radio::PrimaryStatusHF")
        .rustified_enum("Radio::PrimaryStatusLF")
        .rustified_enum("Radio::MessageType")
        .rustified_enum("Radio::RobotCommand")
        .rustified_enum("HG::ConfigOperation")
        .rustified_enum("HG::VariableType")
        .rustified_enum("HG::Variable")
        .rustified_enum("Radio::Access")
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
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    #[cfg(target_os = "macos")]
    {
        bindgen_builder = bindgen_builder.clang_arg("-D_LIBCPP_HAS_NO_THREADS");
    }

    let bindings = bindgen_builder
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    // println!("cargo:warning={}", env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}