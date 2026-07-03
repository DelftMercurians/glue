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
            // These enums are decoded directly from wire bytes. Emitting them as
            // newtypes (see newtype_enum below) means every byte pattern is a
            // representable value, so transmuting a garbled frame into a struct
            // that contains them can never be undefined behaviour. They keep
            // AsBytes (needed by the containing message structs) but drop the
            // num_derive traits, which only apply to real `enum`s.
            "HG_Status" | "Radio_MessageType" | "Radio_RobotCommand" =>
                vec![
                    "AsBytes".into(),
                ],
            "HG_Pose" | "CAN_VARIABLE" | "Radio_ConfigMessage" | "Radio_Command" | "Radio_GenericCommand" | "Radio_Reply" | "Radio_GlobalCommand" |
            "Radio_PrimaryStatusHF" | "Radio_PrimaryStatusLF" | "Radio_SerialMessage" |
            "Radio_ImuReadings" | "MessageType" | "Radio_Message" |
            "Radio_Message__bindgen_ty_1" | "Radio_Message__bindgen_ty_1__bindgen_ty_1" | "Radio_Message__bindgen_ty_1__bindgen_ty_2" |
            "Radio_MultiConfigMessage" | "HG_ConfigOperation" | "HG_VariableType" | "Radio_PrimaryStatusHF__bindgen_ty_1" | "Radio_PrimaryStatusHF__bindgen_ty_2" |
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
        .raw_line("use zerocopy_derive::AsBytes;")
        .raw_line("use strum_macros::EnumIter;")
        // Add these — cover [u8; 1] through whatever sizes your structs use:
        .raw_line("unsafe impl zerocopy::AsBytes for __BindgenBitfieldUnit<[u8; 1]> { fn only_derive_is_allowed_to_implement_this_trait() {} }")
        .raw_line("unsafe impl zerocopy::AsBytes for __BindgenBitfieldUnit<[u8; 2]> { fn only_derive_is_allowed_to_implement_this_trait() {} }")
        .raw_line("unsafe impl zerocopy::AsBytes for __BindgenBitfieldUnit<[u8; 4]> { fn only_derive_is_allowed_to_implement_this_trait() {} }")
        .raw_line("unsafe impl zerocopy::AsBytes for __BindgenBitfieldUnit<[u8; 8]> { fn only_derive_is_allowed_to_implement_this_trait() {} }")
        .header("wrapper.hpp")
        .derive_debug(true)
        // Wire-inbound enums: newtypes so no byte pattern is ever an invalid
        // value (removes the transmute-UB surface when parsing garbled frames).
        .newtype_enum("HG::Status")
        .newtype_enum("Radio::MessageType")
        .newtype_enum("Radio::RobotCommand")
        .rustified_enum("CAN::DEVICE_ID")
        .rustified_enum("CAN::MESSAGE_ID")
        .rustified_enum("CAN::ACCESS")
        .rustified_enum("CAN::VARIABLE")
        .rustified_enum("Radio::PrimaryStatusHF")
        .rustified_enum("Radio::PrimaryStatusLF")
        .rustified_enum("HG::ConfigOperation")
        .rustified_enum("HG::VariableType")
        .rustified_enum("HG::Variable")
        .rustified_enum("Radio::Access")
        .rustified_enum("HG::ReflexState")
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