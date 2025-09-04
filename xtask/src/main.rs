use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    args.next();
    match args.next().as_deref() {
        Some("pathdef") => pathdef(),
        Some("which") => {
            if let Some(prog) = args.next() {
                if let Some(path) = which(&prog) {
                    println!("{}", path.display());
                    Ok(())
                } else {
                    Err(format!("{} not found", prog).into())
                }
            } else {
                Err("missing executable name".into())
            }
        }
        _ => {
            eprintln!("usage: xtask <pathdef|which> [args]");
            Ok(())
        }
    }
}

fn pathdef() -> Result<(), Box<dyn std::error::Error>> {
    let link_sed = Path::new("auto/link.sed");
    if link_sed.exists() && link_sed.metadata()?.len() > 0 {
        fs::copy("auto/pathdef.c", "auto/pathdef.tmp")?;
        let output = Command::new("sed")
            .args(["-f", "auto/link.sed", "auto/pathdef.tmp"])
            .output()?;
        fs::write("auto/pathdef.c", output.stdout)?;
        fs::remove_file("auto/pathdef.tmp")?;
    }
    Ok(())
}

fn which(prog: &str) -> Option<std::path::PathBuf> {
    let path_var = env::var_os("PATH")?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(prog);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
