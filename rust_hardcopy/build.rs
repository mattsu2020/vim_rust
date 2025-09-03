fn main() {
    // Generate bindings for the small C helper.
    let bindings = bindgen::Builder::default()
        .header("src/c_printer.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Compile the C helper.
    cc::Build::new()
        .file("src/c_printer.c")
        .compile("c_printer");
}
