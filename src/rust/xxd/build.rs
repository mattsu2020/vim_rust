fn main() {
    cc::Build::new()
        .file("xxd.c")
        .define("main", Some("xxd_main"))
        .compile("libxxd.a");
}
