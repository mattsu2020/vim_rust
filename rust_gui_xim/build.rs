use std::env;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(have_xim)");
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if matches!(target_os.as_str(), "linux" | "freebsd" | "netbsd" | "openbsd" | "dragonfly" | "macos") {
        println!("cargo:rustc-cfg=have_xim");
    }
}
