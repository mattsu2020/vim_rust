fn main() {
    #[cfg(target_os = "windows")]
    {
        use windows_resource::WindowsResource;
        let mut res = WindowsResource::new();
        res.set_resource_file("../gvimext.rc");
        res.compile().expect("failed to compile resources");
        println!("cargo:rustc-link-arg=/DEF:../gvimext.def");
    }
}
