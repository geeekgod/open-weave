use std::env;

/// Entry point for the build script that generates C bindings.
///
/// Reads the `CARGO_MANIFEST_DIR` environment variable, loads `cbindgen.toml`,
/// generates C header bindings for the crate, and writes them to `openweave.h`.
///
/// # Panics
///
/// Panics if `CARGO_MANIFEST_DIR` is not set, if `cbindgen.toml` cannot be read or
/// parsed, or if binding generation fails.
///
/// # Examples
///
/// ```no_run
/// use std::env;
///
/// env::set_var("CARGO_MANIFEST_DIR", "."); // ensure the crate directory is set
/// // Ensure `cbindgen.toml` exists in the crate root before running.
/// main();
/// ```
fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(cbindgen::Config::from_file("cbindgen.toml").unwrap())
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("openweave.h");
}