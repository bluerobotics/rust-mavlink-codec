fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // The C reference shim is only compiled for the opt-in benchmark arm so that normal
    // builds need neither a C compiler nor the c_library_v2 submodule.
    if std::env::var_os("CARGO_FEATURE_BENCH_C_REFERENCE").is_none() {
        return;
    }

    let lib_root = std::path::Path::new("third_party/c_library_v2");
    assert!(
        lib_root.join("ardupilotmega/mavlink.h").exists(),
        "the `bench-c-reference` feature requires the c_library_v2 submodule; run \
         `git submodule update --init third_party/c_library_v2`"
    );

    println!("cargo:rerun-if-changed=benches/c_shim/shim.c");

    cc::Build::new()
        .file("benches/c_shim/shim.c")
        .include(lib_root)
        .warnings(false)
        .flag_if_supported("-Wno-address-of-packed-member")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-function")
        .compile("mavlink_c_shim");
}
