fn main() {
    let sdk_path = std::env::var("REDSDK_PATH").unwrap_or_else(|_| "sdk".to_string());

    let include_path = std::path::PathBuf::from(format!("{sdk_path}/Include"));

    let mut config = cpp_build::Config::new();

    config
        .flag_if_supported("-std=c++14")
        .flag_if_supported("-fno-rtti")
        .flag_if_supported("-xobjective-c++")
        .include(&include_path)
        .include("headers/")
        .build("src/lib.rs");

    println!("cargo:rerun-if-changed=src/asyncdecoder.rs");
    println!("cargo:rerun-if-changed=src/clip.rs");
    println!("cargo:rerun-if-changed=src/custom_io.rs");
    println!("cargo:rerun-if-changed=src/debayer/cuda.rs");
    println!("cargo:rerun-if-changed=src/debayer/metal.rs");
    println!("cargo:rerun-if-changed=src/debayer/opencl.rs");
    println!("cargo:rerun-if-changed=src/image_processing_settings.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/metadata.rs");
    println!("cargo:rerun-if-changed=src/r3ddecoder.rs");
    println!("cargo:rerun-if-changed=src/sdk.rs");

    println!("cargo:rustc-include-search={sdk_path}/Include");
    println!("cargo:rustc-include-search=headers/");

    if cfg!(feature = "link") {
        if cfg!(target_os = "windows") {
            println!("cargo:rustc-link-search={sdk_path}/Lib/win64");
            println!("cargo:rustc-link-lib=R3DSDK-2017MD");
        } else if cfg!(target_os = "macos") {
            println!("cargo:rustc-link-search={sdk_path}/Lib/mac64");
            println!("cargo:rustc-link-lib=R3DSDK-libcpp");
            println!("cargo:rustc-link-lib=framework=Metal");
        } else if cfg!(target_os = "linux") {
            println!("cargo:rustc-link-search={sdk_path}/Lib/linux64");
            println!("cargo:rustc-link-lib=R3DSDKPIC-cpp11");
        }
    }
}
