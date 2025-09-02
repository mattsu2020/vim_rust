fn main() {
    for pkg in ["gtk+-3.0", "gdk-x11-3.0", "gdk-wayland-3.0"] {
        if let Ok(lib) = pkg_config::probe_library(pkg) {
            for path in lib.link_paths {
                println!("cargo:rustc-link-search=native={}", path.display());
            }
            for lib in lib.libs {
                println!("cargo:rustc-link-lib={}", lib);
            }
        }
    }
}
