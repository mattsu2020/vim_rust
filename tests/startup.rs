use std::process::Command;

#[test]
fn prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_vim_channel"))
        .arg("--version")
        .output()
        .expect("failed to run binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("VIM - Vi IMproved"));
    assert!(output.status.success());
}

#[test]
fn prints_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_vim_channel"))
        .arg("--help")
        .output()
        .expect("failed to run binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: vim"));
    assert!(output.status.success());
}
