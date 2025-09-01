use std::env;
use std::fs;
use std::path::PathBuf;

fn maybe_rename(src_dir: &PathBuf, dst_dir: &PathBuf, old: &str, new: &str) {
    let src_path = src_dir.join(old);
    if src_path.exists() {
        let dst_path = dst_dir.join(new);
        if let Err(e) = fs::rename(&src_path, &dst_path) {
            eprintln!("Failed to rename {}: {}", src_path.display(), e);
        }
    }
}

fn rename_with_candidates(src_dir: &PathBuf, dst_dir: &PathBuf, candidates: &[&str], new: &str) {
    for cand in candidates {
        let src_path = src_dir.join(cand);
        if src_path.exists() {
            let dst_path = dst_dir.join(new);
            if let Err(e) = fs::rename(&src_path, &dst_path) {
                eprintln!("Failed to rename {}: {}", src_path.display(), e);
            }
            break;
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let src_dir = PathBuf::from(args.get(1).cloned().unwrap_or_else(|| "../src".to_string()));
    let dst_dir = PathBuf::from(
        args.get(2)
            .cloned()
            .unwrap_or_else(|| src_dir.to_string_lossy().to_string()),
    );

    maybe_rename(&src_dir, &dst_dir, "vim.exe", "vimw32.exe");
    maybe_rename(&src_dir, &dst_dir, "vim.pdb", "vimw32.pdb");
    maybe_rename(&src_dir, &dst_dir, "gvim.exe", "gvim_ole.exe");
    maybe_rename(&src_dir, &dst_dir, "gvim.pdb", "gvim_ole.pdb");
    maybe_rename(&src_dir, &dst_dir, "install.exe", "installw32.exe");
    maybe_rename(&src_dir, &dst_dir, "uninstall.exe", "uninstallw32.exe");
    rename_with_candidates(
        &src_dir,
        &dst_dir,
        &["tee/tee.exe", "tee.exe"],
        "teew32.exe",
    );
    rename_with_candidates(
        &src_dir,
        &dst_dir,
        &["xxd/xxd.exe", "xxd.exe"],
        "xxdw32.exe",
    );
}
