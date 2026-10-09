fn main() {
    let sdk_path = std::env::var("R3DSDK_DIR").unwrap_or_else(|_| "sdk".to_string());

    let include_path = std::path::PathBuf::from(format!("{sdk_path}/Include"));

    let mut config = cpp_build::Config::new();

    config
        .flag_if_supported("-std=c++14")
        .flag_if_supported("-fno-rtti")
        .flag_if_supported("-xobjective-c++")
        .include(&include_path)
        .include("headers/")
        .build("src/lib.rs");

    println!("cargo:rerun-if-env-changed=R3DSDK_DIR");
    println!("cargo:rerun-if-changed={sdk_path}/Include");
    // Every module may hold `cpp!` blocks the build compiles, so any source change reruns it.
    println!("cargo:rerun-if-changed=src");

    println!("cargo:rustc-include-search={sdk_path}/Include");
    println!("cargo:rustc-include-search=headers/");

    // A build script runs on the host, so the target comes from Cargo's
    // `CARGO_CFG_*` variables, not from `cfg!`.
    if std::env::var_os("CARGO_FEATURE_LINK").is_some() {
        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
        let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
        match (target_os.as_str(), target_arch.as_str()) {
            ("windows", "aarch64") => {
                // The ARM64 SDK ships only Visual Studio 2022 builds.
                println!("cargo:rustc-link-search={sdk_path}/Lib/winarm64");
                println!("cargo:rustc-link-lib=R3DSDK-2022MD");
            }
            ("windows", _) => {
                println!("cargo:rustc-link-search={sdk_path}/Lib/win64");
                println!("cargo:rustc-link-lib=R3DSDK-2017MD");
            }
            ("macos", _) => {
                println!("cargo:rustc-link-search={sdk_path}/Lib/mac64");
                println!("cargo:rustc-link-lib=R3DSDK-libcpp");
                println!("cargo:rustc-link-lib=framework=Metal");
            }
            ("linux", _) => {
                println!("cargo:rustc-link-search={sdk_path}/Lib/linux64");
                println!("cargo:rustc-link-lib=R3DSDKPIC-cpp11");
            }
            _ => {}
        }
    }
}
