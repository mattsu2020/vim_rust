fn main() {
    println!("cargo:rerun-if-changed=src/nv_cmds.h");
    create_nvcmdidxs::generate_file("src/nv_cmdidxs.h").expect("failed to generate nv_cmdidxs.h");

    // Generate a sample Vim script via Rust at build time.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let out_path = std::path::Path::new(&manifest_dir)
        .join("target")
        .join("generated")
        .join("vim")
        .join("rust_hello.vim");
    if let Err(err) = create_vimfile::generate_file(&out_path) {
        panic!("failed to generate {}: {}", out_path.display(), err);
    }
    println!("cargo:warning=generated_vim_file={}", out_path.display());

    // Also place it under runtime/plugin so stock Vim picks it up from runtimepath.
    let runtime_plugin = std::path::Path::new(&manifest_dir)
        .join("runtime")
        .join("plugin")
        .join("rust_hello.vim");
    if let Err(err) = create_vimfile::generate_file(&runtime_plugin) {
        panic!("failed to write runtime plugin {}: {}", runtime_plugin.display(), err);
    }
    println!("cargo:warning=installed_runtime_plugin={}", runtime_plugin.display());
}
