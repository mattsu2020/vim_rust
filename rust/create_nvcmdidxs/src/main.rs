use std::env;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let out_path = env::args().nth(1).unwrap_or_else(|| "nv_cmdidxs.h".to_string());
    create_nvcmdidxs::generate_file(Path::new(&out_path))
}
