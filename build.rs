fn main() {
    println!("cargo:rerun-if-changed=src/nv_cmds.h");
    println!("cargo:rerun-if-changed=src/normal.c");
    create_nvcmdidxs::generate_file("src/nv_cmdidxs.h").expect("failed to generate nv_cmdidxs.h");
}
