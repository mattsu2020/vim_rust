fn main() {
    cc::Build::new()
        .file("src/nv_cmds_wrapper.c")
        .include("../src")
        .compile("nv_cmds_wrapper");
}
