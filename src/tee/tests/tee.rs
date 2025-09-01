use assert_cmd::Command;
use assert_fs::prelude::*;

#[test]
fn writes_to_multiple_files() -> Result<(), Box<dyn std::error::Error>> {
    let file1 = assert_fs::NamedTempFile::new("f1.txt")?;
    let file2 = assert_fs::NamedTempFile::new("f2.txt")?;

    let mut cmd = Command::cargo_bin("tee")?;
    cmd.arg(file1.path())
        .arg(file2.path())
        .write_stdin("hello\n");

    cmd.assert().success().stdout("hello\n");

    file1.assert("hello\n");
    file2.assert("hello\n");
    Ok(())
}

#[test]
fn appends_with_a_option() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("a.txt")?;
    file.write_str("init\n")?;

    let mut cmd = Command::cargo_bin("tee")?;
    cmd.arg("-a")
        .arg(file.path())
        .write_stdin("more\n");

    cmd.assert().success().stdout("more\n");

    file.assert("init\nmore\n");
    Ok(())
}
