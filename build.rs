use bindgen;
use cc;

use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .default_enum_style(bindgen::EnumVariation::Rust)
        .raw_line("#![allow(dead_code, non_camel_case_types, non_snake_case)]")
        .raw_line("#![allow(clippy::unreadable_literal)]")
        .header("src/win.h")
        .header("src/ViGEmClient/include/ViGEm/Client.h")
        .clang_args(&["-I", "src/ViGEmClient/include"])
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from("src/vigem_api_gen.rs");
    bindings.write_to_file(out_path)
        .expect("Couldn't write bindings!");
    cc::Build::new()
        .cpp(true)
        .static_crt(true)
        .include("src/ViGEmClient/include")
        .file("src/ViGEmClient/src/ViGEmClient.cpp")
        .define("VIGEM_DYNAMIC", None)
        .define("VIGEM_EXPORTS", None)
        .define("NDEBUG", None)
        .define("_LIB", None).define("_WINDLL", None)
        .define("_UNICODE", None).define("UNICODE", None)
        .flag("/EHsc") //needed for C++ error unwinding
        .object("setupapi.lib")
//      standard MSVC flags
/*        .flag("/Gd").flag("/TP").flag("/FC")
        .flag("/Zi").flag("/W3").flag("/WX-")
        .flag("/sdl").flag("/Oi").flag("/GL")
        .flag("/Zc:wchar_t").flag("/Zc:forScope")
        .flag("/Zc:inline").flag("/Gm-")
        .flag("/GS").flag("/Gy")*/
        .compile("VigemClient");

}
