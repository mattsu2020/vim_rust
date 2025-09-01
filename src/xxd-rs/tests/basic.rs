use std::fs::{self, File};
use std::io::Write;
use std::process::Command;

fn create_number_file(path: &std::path::Path) {
    let mut f = File::create(path).unwrap();
    for i in 1..=30 {
        writeln!(f, "{}", i).unwrap();
    }
}

#[test]
fn hexdump_and_reverse() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.txt");
    create_number_file(&input);

    let bin = env!("CARGO_BIN_EXE_xxd");
    let out = Command::new(bin).arg(&input).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let expected = [
        "00000000: 310a 320a 330a 340a 350a 360a 370a 380a  1.2.3.4.5.6.7.8.",
        "00000010: 390a 3130 0a31 310a 3132 0a31 330a 3134  9.10.11.12.13.14",
        "00000020: 0a31 350a 3136 0a31 370a 3138 0a31 390a  .15.16.17.18.19.",
        "00000030: 3230 0a32 310a 3232 0a32 330a 3234 0a32  20.21.22.23.24.2",
        "00000040: 350a 3236 0a32 370a 3238 0a32 390a 3330  5.26.27.28.29.30",
        "00000050: 0a                                       .",
    ].join("\n") + "\n";
    assert_eq!(expected, stdout);

    let hexfile = dir.path().join("dump.txt");
    fs::write(&hexfile, &stdout).unwrap();

    let rev = Command::new(bin).arg("-r").arg(&hexfile).output().unwrap();
    let original = fs::read(&input).unwrap();
    assert_eq!(original, rev.stdout);
}

#[test]
fn plain_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("plain.txt");
    fs::write(&input, b"hello").unwrap();
    let bin = env!("CARGO_BIN_EXE_xxd");

    let out = Command::new(bin).arg("-p").arg(&input).output().unwrap();
    let hexfile = dir.path().join("hex.txt");
    fs::write(&hexfile, &out.stdout).unwrap();

    let rev = Command::new(bin)
        .arg("-r")
        .arg("-p")
        .arg(&hexfile)
        .output()
        .unwrap();
    let original = fs::read(&input).unwrap();
    assert_eq!(original, rev.stdout);
}
