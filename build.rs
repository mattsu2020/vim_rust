fn main() {
    println!("cargo:rerun-if-changed=c_src/add.c");
    cc::Build::new()
        .file("c_src/add.c")
        .compile("c_add");
}
