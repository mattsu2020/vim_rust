fn main() {
    println!("cargo:rerun-if-changed=../src/rust_option.h");
    let bindings = bindgen::Builder::default()
        .header("../src/rust_option.h")
        .allowlist_type("rs_opt_t")
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}
